use std::collections::BTreeMap;

use tauri::{AppHandle, Emitter, State, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;

use crate::config::{
    list_installed_desklings, AppSettings, DesklingReloadedEvent, DesklingSummary, Theme,
    RELOAD_EVENT,
};
use crate::error::{DesklingError, DesklingResult};
use crate::runtime::event::DesklingInfo;
use crate::state::{set_runtime_quantity, stop_all_for_slug, AppState};
use crate::util::lock;

pub const THEME_CHANGED_EVENT: &str = "theme_changed";

#[tauri::command]
pub fn get_deskling_info(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Option<DesklingInfo> {
    let label = window.label();
    let guard = lock(&state.runtimes);
    let runtime = guard.values().find(|rt| rt.window_label == label)?;
    let live = runtime.live();
    let cfg = live.load();
    let sprites_dir = state.desklings_dir.join(&runtime.slug).join("sprites");
    Some(DesklingInfo {
        name: cfg.deskling.name.clone(),
        size: cfg.deskling.size,
        sprites_dir: sprites_dir.to_string_lossy().into_owned(),
        version: live.version(),
    })
}

#[tauri::command]
pub fn list_desklings(state: State<'_, AppState>) -> Vec<DesklingSummary> {
    list_installed_desklings(&state.desklings_dir)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.snapshot()
}

#[tauri::command]
pub fn get_active_desklings(state: State<'_, AppState>) -> BTreeMap<String, u32> {
    state.settings.snapshot().active_desklings
}

#[tauri::command]
pub fn enable_deskling(
    slug: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> DesklingResult<()> {
    set_deskling_quantity(slug, 1, app, state)
}

#[tauri::command]
pub fn disable_deskling(
    slug: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> DesklingResult<()> {
    stop_all_for_slug(&state, &app, &slug);
    state.settings.disable_deskling(&slug)?;
    app.emit(
        RELOAD_EVENT,
        DesklingReloadedEvent { slug: slug.clone() },
    )?;
    Ok(())
}

#[tauri::command]
pub fn set_deskling_quantity(
    slug: String,
    quantity: u32,
    app: AppHandle,
    state: State<'_, AppState>,
) -> DesklingResult<()> {
    set_runtime_quantity(&state, &app, &slug, quantity)?;
    state.settings.set_deskling_quantity(&slug, quantity)?;
    app.emit(
        RELOAD_EVENT,
        DesklingReloadedEvent { slug: slug.clone() },
    )?;
    Ok(())
}

#[tauri::command]
pub fn set_locale(locale: Option<String>, state: State<'_, AppState>) -> DesklingResult<()> {
    state.settings.set_locale(locale)
}

#[tauri::command]
pub fn get_theme(state: State<'_, AppState>) -> Theme {
    state.settings.snapshot().theme
}

#[tauri::command]
pub fn set_theme(
    theme: Theme,
    app: AppHandle,
    state: State<'_, AppState>,
) -> DesklingResult<()> {
    state.settings.set_theme(theme)?;
    app.emit(THEME_CHANGED_EVENT, theme)?;
    Ok(())
}

#[tauri::command]
pub fn get_autostart_enabled(app: AppHandle) -> DesklingResult<bool> {
    app.autolaunch()
        .is_enabled()
        .map_err(|e| DesklingError::Autostart(e.to_string()))
}

#[tauri::command]
pub fn set_autostart_enabled(app: AppHandle, enabled: bool) -> DesklingResult<()> {
    let manager = app.autolaunch();
    if enabled {
        manager
            .enable()
            .map_err(|e| DesklingError::Autostart(format!("enable: {e}")))
    } else {
        manager
            .disable()
            .map_err(|e| DesklingError::Autostart(format!("disable: {e}")))
    }
}

#[tauri::command]
pub const fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
pub fn app_identifier(app: AppHandle) -> String {
    app.config().identifier.clone()
}

#[tauri::command]
pub const fn app_website() -> &'static str {
    env!("CARGO_PKG_HOMEPAGE")
}

#[tauri::command]
pub const fn app_author() -> &'static str {
    env!("CARGO_PKG_AUTHORS")
}
