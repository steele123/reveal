//! The tray icon. It only exists while "Close to tray" is on; then closing the window hides
//! Reveal there instead of quitting, so it keeps watching League.

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    App, AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, Window,
    Wry,
};

const TRAY_ID: &str = "main";

/// The setting, kept here so closing the window doesn't wait on the config lock.
static CLOSE_TO_TRAY: AtomicBool = AtomicBool::new(false);

/// Adds the tray icon at startup when the setting is on.
pub fn init(app: &App, close_to_tray: bool) {
    CLOSE_TO_TRAY.store(close_to_tray, Ordering::Release);
    if close_to_tray {
        add(app);
    }
}

/// Adds or removes the tray icon when the setting changes.
pub fn apply(app: &AppHandle, close_to_tray: bool) {
    if CLOSE_TO_TRAY.swap(close_to_tray, Ordering::AcqRel) == close_to_tray {
        return;
    }
    if close_to_tray {
        add(app);
    } else if let Some(tray) = app.tray_handle_by_id(TRAY_ID) {
        if let Err(error) = tray.destroy() {
            log_error!("Couldn't remove the tray icon: {error}");
        }
    }
}

pub fn closes_to_tray() -> bool {
    CLOSE_TO_TRAY.load(Ordering::Acquire)
}

fn add<M: Manager<Wry>>(manager: &M) {
    let menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show", "Open Reveal"))
        .add_item(CustomMenuItem::new("quit", "Quit Reveal"));
    if let Err(error) = SystemTray::new()
        .with_id(TRAY_ID)
        .with_menu(menu)
        .with_tooltip("Reveal")
        .build(manager)
    {
        log_error!("Couldn't add the tray icon: {error}");
    }
}

pub fn on_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => show_main_window(app),
        SystemTrayEvent::MenuItemClick { id, .. } if id == "show" => show_main_window(app),
        SystemTrayEvent::MenuItemClick { id, .. } if id == "quit" => {
            log_info!("Quit chosen from the tray");
            app.exit(0);
        }
        _ => {}
    }
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Hides the window instead of quitting, and says so, so Reveal doesn't seem to have closed.
pub fn hide_window(window: &Window) {
    if let Err(error) = window.hide() {
        log_error!("Couldn't hide the window: {error}");
        return;
    }
    log_info!("Window hidden; Reveal keeps running in the tray");
    let identifier = &window.app_handle().config().tauri.bundle.identifier;
    if let Err(error) = tauri::api::notification::Notification::new(identifier)
        .title("Reveal is still running")
        .body("It keeps watching League from the tray. Right-click the tray icon to quit.")
        .show()
    {
        log_error!("Couldn't show the tray notice: {error}");
    }
}
