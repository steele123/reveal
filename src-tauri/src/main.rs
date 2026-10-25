// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
mod logging;

mod analytics;
mod app_state;
mod champ_select;
mod champions;
mod commands;
mod config;
mod lcu;
mod legacy_install;
mod live_game;
mod lobby;
mod queue;
mod queue_pause;
mod ranked;
mod region;
mod roles;
mod state;
mod summoner;
mod utils;

use app_state::{Dodge, Lcu, QueuePause};
use commands::{
    app_ready, dodge, enable_dodge, get_config, get_lcu_info, get_lcu_state, get_legacy_install,
    open_opgg_link, remove_legacy_install, set_config, write_frontend_log,
};
use config::AppConfig;
use tauri::Manager;

fn main() {
    match logging::init() {
        Ok(path) => log_info!("Reveal process starting; log file: {}", path.display()),
        Err(error) => eprintln!("Failed to initialize Reveal logging: {error:#}"),
    }

    let result = tauri::Builder::default()
        .manage(Lcu::default())
        .manage(Dodge::default())
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
            open_opgg_link,
            dodge,
            enable_dodge,
            get_legacy_install,
            remove_legacy_install,
            write_frontend_log
        ])
        .run(tauri::generate_context!());

    match result {
        Ok(()) => log_info!("Reveal process exited normally"),
        Err(error) => log_error!("Fatal Tauri runtime error: {error}"),
    }
}
