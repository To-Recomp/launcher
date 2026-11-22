//! The launcher around its window: the notification area icon, and starting with the system.
//!
//! What the player chose (`behavior.*` in `launcher.json`) is read here, when the page asks for
//! it to be applied; "quit once a game has started" is checked by `library::start_game`.

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Deserialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, State, Window, WindowEvent};
use tauri_plugin_autostart::ManagerExt;

const TRAY_ID: &str = "launcher";

/// Whether closing the window hides it instead of quitting.
#[derive(Default)]
pub struct Tray {
    hides_on_close: AtomicBool,
}

/// The menu texts, in the interface language (the backend has no translations of its own).
#[derive(Debug, Deserialize)]
pub struct TrayLabels {
    show: String,
    quit: String,
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// With the notification area icon on, closing the window only hides it.
pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        let tray = window.state::<Tray>();
        if tray.hides_on_close.load(Ordering::Relaxed) {
            api.prevent_close();
            let _ = window.hide();
        }
    }
}

fn add_icon(app: &AppHandle, labels: &TrayLabels) -> Result<(), String> {
    let fail = |e: tauri::Error| format!("Can't put the launcher in the notification area: {e}");
    let show = MenuItem::with_id(app, "show", &labels.show, true, None::<&str>).map_err(fail)?;
    let quit = MenuItem::with_id(app, "quit", &labels.quit, true, None::<&str>).map_err(fail)?;
    let menu = Menu::with_items(app, &[&show, &quit]).map_err(fail)?;
    let mut icon = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_window(app),
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(image) = app.default_window_icon() {
        icon = icon.icon(image.clone());
    }
    icon.build(app).map(|_| ()).map_err(fail)
}

/// Puts the notification area icon on or off as the settings say.
#[tauri::command]
pub fn apply_behavior(app: AppHandle, tray: State<Tray>, labels: TrayLabels) -> Result<(), String> {
    let wanted = crate::launcher_settings::config_dir(&app)
        .and_then(|dir| crate::launcher_settings::behavior(&dir))?
        .tray;
    let present = app.tray_by_id(TRAY_ID).is_some();
    if wanted && !present {
        add_icon(&app, &labels)?;
    } else if wanted {
        // The language may have changed: rebuild so the menu follows it.
        app.remove_tray_by_id(TRAY_ID);
        add_icon(&app, &labels)?;
    } else if present {
        app.remove_tray_by_id(TRAY_ID);
        show_window(&app); // a window hidden in the area must not stay unreachable
    }
    tray.hides_on_close.store(wanted, Ordering::Relaxed);
    Ok(())
}

/// Whether the launcher can be started with the system here.
#[tauri::command]
pub fn autostart_supported() -> bool {
    cfg!(any(target_os = "linux", target_os = "windows"))
}

#[tauri::command]
pub fn autostart_enabled(app: AppHandle) -> Result<bool, String> {
    app.autolaunch()
        .is_enabled()
        .map_err(|e| format!("Can't read whether the launcher starts with the system: {e}"))
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let launch = app.autolaunch();
    let done = if enabled {
        launch.enable()
    } else {
        launch.disable()
    };
    done.map_err(|e| format!("Can't change whether the launcher starts with the system: {e}"))?;
    autostart_enabled(app)
}
