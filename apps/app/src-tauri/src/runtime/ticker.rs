use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use fastrand::Rng;
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewWindow};
use tracing::{instrument, warn};

use crate::config::{Context, LiveConfig, Mode};
use crate::deskling::hitmask::{hit_test, HitMaskCache};
use crate::deskling::monitors::MonitorLayout;
use crate::deskling::state::DesklingPhysics;
use crate::deskling::state_machine::StateMachine;
use crate::input::{cursor_position, CursorArbiter, DragTracker, MouseHook};
use crate::runtime::event::{DesklingStateEvent, STATE_EVENT};
use crate::util::lock;

pub const TICK_INTERVAL: Duration = Duration::from_millis(16);

const LAYOUT_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug)]
pub struct SharedDeskling {
    pub physics: DesklingPhysics,
    pub state_machine: StateMachine,
    pub drag: DragTracker,
}

#[derive(Debug)]
pub struct TickerHandle {
    shutdown: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl TickerHandle {
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
    }
}

impl Drop for TickerHandle {
    fn drop(&mut self) {
        self.shutdown();
        if let Some(j) = self.join.take() {
            if let Err(e) = j.join() {
                warn!("ticker thread panicked on shutdown: {e:?}");
            }
        }
    }
}

#[must_use]
pub fn spawn_ticker(
    app_handle: AppHandle,
    window_label: String,
    instance_id: String,
    shared: Arc<Mutex<SharedDeskling>>,
    live: Arc<LiveConfig>,
    mouse: Arc<MouseHook>,
    arbiter: Arc<CursorArbiter>,
    initial_layout: MonitorLayout,
    sprites_dir: PathBuf,
) -> TickerHandle {
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_for_thread = Arc::clone(&shutdown);
    let thread_name = format!("desklings-ticker-{window_label}");

    let join = std::thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            ticker_loop(
                &app_handle,
                &window_label,
                &instance_id,
                &shared,
                &live,
                &mouse,
                &arbiter,
                initial_layout,
                sprites_dir,
                &shutdown_for_thread,
            );
        })
        .ok();

    TickerHandle { shutdown, join }
}

#[instrument(name = "ticker", skip_all, fields(window = %window_label))]
fn ticker_loop(
    app_handle: &AppHandle,
    window_label: &str,
    instance_id: &str,
    shared: &Arc<Mutex<SharedDeskling>>,
    live: &Arc<LiveConfig>,
    mouse: &Arc<MouseHook>,
    arbiter: &Arc<CursorArbiter>,
    initial_layout: MonitorLayout,
    sprites_dir: PathBuf,
    shutdown: &Arc<AtomicBool>,
) {
    let mut rng = Rng::new();
    let mut prev_pressed = false;
    let mut last_release_count = mouse.snapshot().release_count;
    let mut cursor_was_over = false;
    let mut last_config_version = live.version();

    let mut layout = initial_layout;
    let mut last_layout_refresh = Instant::now();

    let mut hit_masks = HitMaskCache::new(sprites_dir);

    let mut last_sprite: Option<String> = None;
    let mut last_flip: bool = false;

    while !shutdown.load(Ordering::Acquire) {
        let tick_started = Instant::now();
        let dt = TICK_INTERVAL.as_secs_f64();

        let cfg = live.load();
        let current_version = live.version();
        let config_changed = current_version != last_config_version;
        last_config_version = current_version;

        let layout_changed = maybe_refresh_layout(
            app_handle,
            window_label,
            &mut layout,
            &mut last_layout_refresh,
            tick_started,
        );

        let mouse_state = mouse.snapshot();
        let release_edge = mouse_state.release_count != last_release_count;
        last_release_count = mouse_state.release_count;
        let cursor = cursor_position();

        let (view, should_raise_to_front) = {
            let mut guard = lock(shared);
            let SharedDeskling {
                physics,
                state_machine,
                drag,
            } = &mut *guard;

            if layout_changed {
                physics.apply_layout(&layout);
            }

            if config_changed {
                state_machine.reset_for(&cfg);
                hit_masks.invalidate();
                let new_size = f64::from(cfg.deskling.size);
                if (physics.size - new_size).abs() > f64::EPSILON {
                    physics.size = new_size;
                    let vb = physics.virtual_bounds;
                    physics.x = physics.x.clamp(vb.min_x, vb.max_x - physics.size);
                    physics.y = physics.y.clamp(vb.min_y, vb.max_y - physics.size);
                    resize_window(app_handle, window_label, new_size);
                }
            }

            let mut cursor_over_deskling = false;
            let mut should_raise_to_front = false;

            if let Some((cx, cy)) = cursor {
                let cursor_on_sprite = if state_machine.mode() == Mode::Dragging {
                    true
                } else {
                    let mask = last_sprite.as_deref().and_then(|name| hit_masks.get(name));
                    hit_test(mask, physics.x, physics.y, physics.size, cx, cy, last_flip)
                };

                arbiter.publish_hit(instance_id, cursor_on_sprite);
                cursor_over_deskling = cursor_on_sprite;

                let now = now_secs();

                if mouse_state.left_down
                    && !prev_pressed
                    && cursor_on_sprite
                    && arbiter.should_claim(instance_id)
                {
                    drag.begin(physics, cx, cy, now);
                    state_machine.enter_dragging(&cfg);
                    arbiter.push_top(instance_id);
                    should_raise_to_front = true;
                }
                if mouse_state.left_down && state_machine.mode() == Mode::Dragging {
                    drag.update(physics, &cfg.drag, cx, cy, now);
                    physics.rebind_active_monitor_if_needed(&layout);
                }
                if release_edge && state_machine.mode() == Mode::Dragging {
                    let outcome = drag.end(cfg.physics.max_throw_velocity);
                    physics.vx = outcome.vx;
                    physics.vy = outcome.vy;
                    physics.rebind_active_monitor_if_needed(&layout);
                    state_machine.enter_falling(&cfg);
                }
            } else {
                arbiter.publish_hit(instance_id, false);
            }

            let ctx = Context {
                speed: drag.smooth_speed(),
            };
            let view = state_machine.tick(physics, &cfg, &mut rng, &ctx, dt);

            if cursor_over_deskling != cursor_was_over {
                if let Some(win) = app_handle.get_webview_window(window_label) {
                    let _ = win.set_ignore_cursor_events(!cursor_over_deskling);
                }
                cursor_was_over = cursor_over_deskling;
            }

            (view, should_raise_to_front)
        };

        prev_pressed = mouse_state.left_down;

        last_sprite = Some(view.sprite.clone());
        last_flip = view.flip;

        if let Some(win) = app_handle.get_webview_window(window_label) {
            move_window(&win, view.x, view.y);
        }
        if should_raise_to_front {
            let label_for_main = window_label.to_string();
            let app_for_main = app_handle.clone();
            let _ = app_handle.run_on_main_thread(move || {
                if let Some(win) = app_for_main.get_webview_window(&label_for_main) {
                    raise_window_to_front(&win);
                }
            });
        }

        let payload = DesklingStateEvent::from(&view);
        if let Err(e) = app_handle.emit_to(window_label, STATE_EVENT, &payload) {
            warn!(error = %e, "failed to emit deskling_state event");
        }

        let elapsed = tick_started.elapsed();
        if let Some(remaining) = TICK_INTERVAL.checked_sub(elapsed) {
            std::thread::sleep(remaining);
        }
    }
}

fn maybe_refresh_layout(
    app_handle: &AppHandle,
    window_label: &str,
    layout: &mut MonitorLayout,
    last_refresh: &mut Instant,
    now: Instant,
) -> bool {
    if now.duration_since(*last_refresh) < LAYOUT_REFRESH_INTERVAL {
        return false;
    }
    *last_refresh = now;
    let Some(win) = app_handle.get_webview_window(window_label) else {
        return false;
    };
    let new_layout = MonitorLayout::from_window(&win);
    let changed = new_layout.virtual_bounds != layout.virtual_bounds
        || new_layout.monitors != layout.monitors;
    *layout = new_layout;
    changed
}

fn move_window(window: &WebviewWindow, x: f64, y: f64) {
    let _ = window.set_position(LogicalPosition::new(x, y));
}

fn raise_window_to_front(window: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::NSWindow;

        let Ok(handle) = window.ns_window() else {
            return;
        };
        let ns_window: &NSWindow = unsafe { &*handle.cast::<NSWindow>() };
        ns_window.orderFrontRegardless();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window.set_always_on_top(false);
        let _ = window.set_always_on_top(true);
    }
}

fn resize_window(app: &AppHandle, window_label: &str, size: f64) {
    if let Some(win) = app.get_webview_window(window_label) {
        let _ = win.set_size(LogicalSize::new(size, size));
    }
}

fn now_secs() -> f64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}
