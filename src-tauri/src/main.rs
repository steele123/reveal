// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
mod logging;

mod analytics;
mod app_state;
mod automation;
mod champ_select;
mod champions;
mod commands;
mod config;
mod events;
mod lcu;
mod legacy_install;
mod live_game;
mod lobby;
mod lookup;
mod matchups;
mod notify;
mod queue;
mod queue_pause;
mod ranked;
mod region;
mod roles;
mod state;
mod summoner;
mod utils;

use app_state::{Lcu, QueuePause};
use commands::{
    app_ready, get_champion_list, get_config, get_lcu_info, get_lcu_state, get_legacy_install,
    get_own_profile,
    notify_lobby_revealed,
    open_log_folder, open_opgg_link, remove_legacy_install, send_test_notification, set_config,
    set_enemy_role, write_frontend_log,
};
use commands::hide_to_tray;
use config::AppConfig;
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowEvent,
};

fn main() {
    match logging::init() {
        Ok(path) => log_info!("Reveal process starting; log file: {}", path.display()),
        Err(error) => eprintln!("Failed to initialize Reveal logging: {error:#}"),
    }

    let builder = tauri::Builder::default();
    // A second copy can't open its window while the first holds WebView2's data folder, so hand
    // the launch to the running copy instead. Dev builds may run next to an installed Reveal.
    #[cfg(all(windows, not(debug_assertions)))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
        log_info!("Reveal was launched again; showing the running window");
        show_main_window(app);
    }));

    let result = builder
        .system_tray(tray())
        .on_system_tray_event(on_tray_event)
        // Closing the window only hides it, so Reveal keeps watching League from the tray.
        .on_window_event(|event| {
            if let WindowEvent::CloseRequested { api, .. } = event.event() {
                api.prevent_close();
                commands::hide_to_tray(event.window().clone());
            }
        })
        .manage(Lcu::default())
        .manage(QueuePause::default())
        .setup(|app| {
            let app_handle = app.handle();
            log_info!(
                "Tauri setup started for Reveal v{}",
                app.package_info().version
            );

            let loaded_config = config::load(&app_handle).map_err(|error| {
                log_error!("Startup config load failed: {error:#}");
                error
            })?;
            app.manage(AppConfig::new(loaded_config));

            let live_game_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                live_game::detect(live_game_handle).await;
            });
            tauri::async_runtime::spawn(async move {
                log_info!("League Client monitor started");
                lcu::run(app_handle).await;
            });

            log_info!("Tauri setup completed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_ready,
            get_lcu_state,
            get_lcu_info,
            get_config,
            set_config,
            open_log_folder,
            open_opgg_link,
            get_legacy_install,
            remove_legacy_install,
            write_frontend_log,
            set_enemy_role,
            hide_to_tray,
            send_test_notification,
            notify_lobby_revealed,
            get_own_profile,
            get_champion_list
        ])
        .run(tauri::generate_context!());

    match result {
        Ok(()) => log_info!("Reveal process exited normally"),
        Err(error) => {
            log_error!("Fatal Tauri runtime error: {error}");
            show_startup_error(&error.to_string());
        }
    }
}

fn tray() -> SystemTray {
    let menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show", "Open Reveal"))
        .add_item(CustomMenuItem::new("quit", "Quit Reveal"));
    SystemTray::new().with_menu(menu).with_tooltip("Reveal")
}

fn on_tray_event(app: &AppHandle, event: SystemTrayEvent) {
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

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Reveal has no window to show errors in when Tauri fails, so use a native message box.
#[cfg(windows)]
fn show_startup_error(error: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

    let hint = if error.contains("WebView2") {
        "Reveal couldn't open its window because Microsoft Edge WebView2 failed to start.\n\n\
         Restart your PC, or reinstall WebView2 from\n\
         https://developer.microsoft.com/microsoft-edge/webview2/"
    } else {
        "Reveal couldn't start."
    };
    let text =
        format!("{hint}\n\nDetails are in %LOCALAPPDATA%\\Reveal\\logs\\reveal.log\n\n{error}");
    let wide = |value: &str| value.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    let (text, caption) = (wide(&text), wide("Reveal"));
    // SAFETY: both strings are NUL-terminated UTF-16 buffers that outlive the call.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(not(windows))]
fn show_startup_error(_error: &str) {}
