use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use tracing::warn;

use crate::config::{load_deskling_by_slug, LiveConfig, SettingsStore};
use crate::deskling::{DesklingPhysics, MonitorLayout, StateMachine};
use crate::error::{DesklingError, DesklingResult};
use crate::input::{CursorArbiter, DragTracker, MouseHook};
use crate::runtime::ticker::{spawn_ticker, SharedDeskling, TickerHandle};
use crate::util::lock;

pub const WINDOW_LABEL_PREFIX: &str = "deskling-";

pub type InstanceId = String;

pub type RuntimeMap = Arc<Mutex<HashMap<InstanceId, Runtime>>>;

#[derive(Debug)]
pub struct AppState {
    pub desklings_dir: PathBuf,
    pub settings: Arc<SettingsStore>,
    pub runtimes: RuntimeMap,
    pub mouse: Arc<MouseHook>,
    pub arbiter: Arc<CursorArbiter>,
}

#[derive(Debug)]
pub struct Runtime {
    pub instance_id: InstanceId,
    pub slug: String,
    pub window_label: String,
    pub live: Arc<LiveConfig>,
    #[allow(dead_code)]
    pub ticker: TickerHandle,
}

impl Runtime {
    pub fn live(&self) -> Arc<LiveConfig> {
        Arc::clone(&self.live)
    }
}

pub fn make_instance_id(slug: &str, index: u32) -> InstanceId {
    format!("{slug}#{index}")
}

pub fn slug_from_instance_id(instance_id: &str) -> &str {
    match instance_id.find('#') {
        Some(pos) => &instance_id[..pos],
        None => instance_id,
    }
}

pub fn window_label_for(instance_id: &str) -> String {
    format!("{WINDOW_LABEL_PREFIX}{}", sanitize_for_label(instance_id))
}

fn sanitize_for_label(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn start_runtime(
    app: &AppHandle,
    desklings_dir: &Path,
    instance_id: &str,
    mouse: Arc<MouseHook>,
    arbiter: Arc<CursorArbiter>,
) -> DesklingResult<Runtime> {
    let slug = slug_from_instance_id(instance_id);
    let loaded = load_deskling_by_slug(desklings_dir, slug)?;
    let live = LiveConfig::new(loaded.config);

    let label = window_label_for(instance_id);
    let window = ensure_overlay_window(app, &label)?;

    let cfg0 = live.load();
    let size = f64::from(cfg0.deskling.size);
    let _ = window.set_size(LogicalSize::new(size, size));

    let layout = MonitorLayout::from_window(&window);
    let physics = DesklingPhysics::new(&layout, size);
    let state_machine = StateMachine::new(&cfg0);
    let drag = DragTracker::new();

    let start_x = physics.x;
    let start_y = physics.y;

    let shared = Arc::new(Mutex::new(SharedDeskling {
        physics,
        state_machine,
        drag,
    }));

    let _ = window.set_position(LogicalPosition::new(start_x, start_y));
    let _ = window.set_ignore_cursor_events(true);

    if let Err(e) = window.set_visible_on_all_workspaces(true) {
        warn!(error = %e, "failed to pin window to all workspaces");
    }
    if let Err(e) = window.show() {
        warn!(error = %e, "failed to show mascot window");
    }

    #[cfg(target_os = "macos")]
    raise_overlay_window(&window);

    arbiter.push_top(instance_id);

    let sprites_dir = desklings_dir.join(slug).join("sprites");
    let ticker = spawn_ticker(
        app.clone(),
        label.clone(),
        instance_id.to_string(),
        Arc::clone(&shared),
        Arc::clone(&live),
        mouse,
        arbiter,
        layout,
        sprites_dir,
    );

    Ok(Runtime {
        instance_id: instance_id.to_string(),
        slug: slug.to_string(),
        window_label: label,
        live,
        ticker,
    })
}

fn ensure_overlay_window(app: &AppHandle, label: &str) -> DesklingResult<WebviewWindow> {
    if let Some(existing) = app.get_webview_window(label) {
        return Ok(existing);
    }

    WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
        .title("Desklings")
        .inner_size(256.0, 256.0)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .focused(false)
        .visible(false)
        .build()
        .map_err(|e| DesklingError::Settings(format!("create overlay window {label}: {e}")))
}

#[cfg(target_os = "macos")]
fn raise_overlay_window(window: &WebviewWindow) {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

    let Some(mtm) = MainThreadMarker::new() else {
        warn!("raise_overlay_window: not on main thread; skipping");
        return;
    };

    let handle = match window.ns_window() {
        Ok(h) => h,
        Err(e) => {
            warn!(error = %e, "ns_window() unavailable; overlay level unchanged");
            return;
        }
    };

    let ns_window: &NSWindow = unsafe { &*handle.cast::<NSWindow>() };

    const NS_SCREEN_SAVER_WINDOW_LEVEL: isize = 1000;
    ns_window.setLevel(NS_SCREEN_SAVER_WINDOW_LEVEL);

    let current = ns_window.collectionBehavior();
    let extra = NSWindowCollectionBehavior::CanJoinAllSpaces
        | NSWindowCollectionBehavior::FullScreenAuxiliary
        | NSWindowCollectionBehavior::Stationary
        | NSWindowCollectionBehavior::IgnoresCycle;
    ns_window.setCollectionBehavior(current | extra);

    let _ = mtm;
}

pub fn stop_instance(state: &AppState, app: &AppHandle, instance_id: &str) {
    let removed = {
        let mut guard = lock(&state.runtimes);
        guard.remove(instance_id)
    };
    state.arbiter.remove(instance_id);
    if let Some(runtime) = removed {
        if let Some(window) = app.get_webview_window(&runtime.window_label) {
            let _ = window.close();
        }
        drop(runtime);
    }
}

pub fn stop_all_for_slug(state: &AppState, app: &AppHandle, slug: &str) {
    let ids: Vec<String> = {
        let guard = lock(&state.runtimes);
        guard
            .values()
            .filter(|rt| rt.slug == slug)
            .map(|rt| rt.instance_id.clone())
            .collect()
    };
    for id in ids {
        stop_instance(state, app, &id);
    }
}

pub fn instance_ids_for_slug(runtimes: &RuntimeMap, slug: &str) -> Vec<InstanceId> {
    let guard = lock(runtimes);
    let mut ids: Vec<(u32, String)> = guard
        .values()
        .filter(|rt| rt.slug == slug)
        .map(|rt| {
            let idx = rt
                .instance_id
                .rsplit_once('#')
                .and_then(|(_, n)| n.parse::<u32>().ok())
                .unwrap_or(0);
            (idx, rt.instance_id.clone())
        })
        .collect();
    ids.sort_by_key(|(idx, _)| *idx);
    ids.into_iter().map(|(_, id)| id).collect()
}

pub fn set_runtime_quantity(
    state: &AppState,
    app: &AppHandle,
    slug: &str,
    quantity: u32,
) -> DesklingResult<()> {
    let current_ids = instance_ids_for_slug(&state.runtimes, slug);
    let current = current_ids.len() as u32;

    if quantity == 0 {
        stop_all_for_slug(state, app, slug);
        return Ok(());
    }

    if current == quantity {
        return Ok(());
    }

    if current < quantity {
        let needed = quantity - current;
        let mut used: std::collections::BTreeSet<u32> = current_ids
            .iter()
            .filter_map(|id| id.rsplit_once('#').and_then(|(_, n)| n.parse::<u32>().ok()))
            .collect();
        for _ in 0..needed {
            let mut next = 1u32;
            while used.contains(&next) {
                next += 1;
            }
            used.insert(next);
            let instance_id = make_instance_id(slug, next);
            let runtime = start_runtime(
                app,
                &state.desklings_dir,
                &instance_id,
                Arc::clone(&state.mouse),
                Arc::clone(&state.arbiter),
            )?;
            let mut guard = lock(&state.runtimes);
            guard.insert(instance_id, runtime);
        }
    } else {
        let excess = (current - quantity) as usize;
        let to_remove: Vec<String> = current_ids
            .iter()
            .rev()
            .take(excess)
            .cloned()
            .collect();
        for id in to_remove {
            stop_instance(state, app, &id);
        }
    }
    Ok(())
}
