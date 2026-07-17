// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
mod logging;

mod analytics;
mod app_state;
mod champ_select;
mod commands;
mod config;
mod lcu;
mod lobby;
mod region;
mod state;
mod summoner;
mod utils;

use app_state::{Dodge, Lcu};
use commands::{
    app_ready, dodge, enable_dodge, get_config, get_lcu_info, get_lcu_state, open_opgg_link,
    set_config, write_frontend_log,
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
            write_frontend_log
        ])
        .run(tauri::generate_context!());

    match result {
        Ok(()) => log_info!("Reveal process exited normally"),
        Err(error) => log_error!("Fatal Tauri runtime error: {error}"),
    }
}
