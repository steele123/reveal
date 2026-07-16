// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
    set_config,
};
use config::AppConfig;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .manage(Lcu::default())
        .manage(Dodge::default())
        .setup(|app| {
            let app_handle = app.handle();
            app.manage(AppConfig::new(config::load(&app_handle)?));

            tauri::async_runtime::spawn(async move {
                lcu::run(app_handle).await;
            });

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
            enable_dodge
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
