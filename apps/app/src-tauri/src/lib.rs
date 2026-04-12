use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tauri::scope::fs::Scope as FsScope;
use tauri::{Manager, WindowEvent};
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;

mod commands;
mod config;
mod deskling;
pub mod error;
mod input;
mod runtime;
mod state;
mod util;

use crate::config::{first_installed_slug, user_desklings_dir, SettingsStore};
use crate::error::DesklingResult;
use crate::input::{CursorArbiter, MouseHook};
use crate::state::{make_instance_id, start_runtime, AppState, RuntimeMap};

const SETTINGS_WINDOW_LABEL: &str = "settings";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> DesklingResult<()> {
    init_tracing();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let desklings_dir_runtime = user_desklings_dir(app.handle())?;
            allow_desklings_dir(&app.asset_protocol_scope(), &desklings_dir_runtime);

            #[cfg(target_os = "macos")]
            if let Err(e) = app
                .handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory)
            {
                warn!(error = %e, "failed to set accessory activation policy");
            }

            let settings = Arc::new(SettingsStore::load(app.handle())?);
            let mouse = MouseHook::spawn();
            let arbiter = Arc::new(CursorArbiter::new());

            let mut initial_quantities = settings.snapshot().active_desklings;
            if initial_quantities.is_empty() {
                if let Some(slug) = first_installed_slug(&desklings_dir_runtime) {
                    initial_quantities.insert(slug, 1);
                }
            }

            let runtimes: RuntimeMap = Arc::new(Mutex::new(HashMap::new()));

            for (slug, quantity) in &initial_quantities {
                for n in 1..=*quantity {
                    let instance_id = make_instance_id(slug, n);
                    match start_runtime(
                        app.handle(),
                        &desklings_dir_runtime,
                        &instance_id,
                        Arc::clone(&mouse),
                        Arc::clone(&arbiter),
                    ) {
                        Ok(rt) => {
                            runtimes
                                .lock()
                                .expect("runtimes mutex poisoned")
                                .insert(instance_id, rt);
                        }
                        Err(e) => {
                            warn!(error = %e, %slug, %n, "failed to start deskling runtime; skipping");
                        }
                    }
                }
            }

            if runtimes.lock().expect("runtimes mutex poisoned").is_empty() {
                tracing::info!(
                    dir = %desklings_dir_runtime.display(),
                    "no desklings active; nothing shown until one is enabled"
                );
            }

            if let Some(settings_window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
                let settings_clone = settings_window.clone();
                settings_window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = settings_clone.hide();
                    }
                });
            }

            install_tray(app.handle())?;

            if let Err(e) = config::spawn_watcher(
                Arc::clone(&runtimes),
                Arc::clone(&settings),
                &desklings_dir_runtime,
                app.handle().clone(),
            ) {
                warn!(error = %e, "hot-reload watcher disabled");
            }

            app.manage(AppState {
                desklings_dir: desklings_dir_runtime,
                settings: Arc::clone(&settings),
                runtimes,
                mouse,
                arbiter,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_deskling_info,
            commands::list_desklings,
            commands::get_active_desklings,
            commands::enable_deskling,
            commands::disable_deskling,
            commands::set_deskling_quantity,
            commands::get_settings,
            commands::set_locale,
            commands::get_theme,
            commands::set_theme,
            commands::get_autostart_enabled,
            commands::set_autostart_enabled,
            commands::app_version,
            commands::app_identifier,
            commands::app_website,
            commands::app_author,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("desklings_lib=info,warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .compact()
        .try_init();
}

fn allow_desklings_dir(scope: &FsScope, desklings_dir: &std::path::Path) {
    if let Err(e) = scope.allow_directory(desklings_dir, true) {
        error!(
            dir = %desklings_dir.display(),
            error = %e,
            "failed to whitelist desklings directory on asset protocol scope"
        );
    }
}

fn install_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::{image::Image, menu::MenuBuilder, tray::TrayIconBuilder};

    let menu = MenuBuilder::new(app)
        .text("settings", "Settings…")
        .separator()
        .text("quit", "Quit Desklings")
        .build()?;

    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

    TrayIconBuilder::new()
        .tooltip("Desklings")
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" => show_settings_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn show_settings_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    } else {
        warn!("settings window missing from tauri.conf.json");
    }
}
