// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod lobby;
mod utils;

use crate::lobby::Lobby;
use crate::utils::display_champ_select;

use futures_util::StreamExt;
use shaco::{model::ws::LcuSubscriptionType::JsonApiEvent, rest::LCUClientInfo};
use shaco::rest::RESTClient;
use shaco::ws::LcuWebsocketClient;
use std::{time::Duration, sync::Arc};
use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex};

struct LCU(Mutex<LCUState>);

pub struct LCUState {
    pub connected: bool,
    pub data: Option<LCUClientInfo>,
}

struct AppConfig(Mutex<Config>);

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Config {
    pub auto_open: bool,
    pub auto_accept: bool,
    pub accept_delay: u32,
}

#[tauri::command]
async fn app_ready(
    app_handle: AppHandle,
    lcu: tauri::State<'_, LCU>,
    cfg: tauri::State<'_, AppConfig>,
) -> Result<Config, ()> {
    let lcu = lcu.0.lock().await;
    let cfg = cfg.0.lock().await;

    app_handle
        .emit_all("lcu_state_update", lcu.connected)
        .unwrap();
    
    Ok(cfg.clone())
}

#[tauri::command]
async fn get_lcu_state(lcu: tauri::State<'_, LCU>) -> Result<bool, ()> {
    let lcu = lcu.0.lock().await;
    Ok(lcu.connected)
}

#[tauri::command]
async fn get_config(cfg: tauri::State<'_, AppConfig>) -> Result<Config, ()> {
    let cfg = cfg.0.lock().await;
    Ok(cfg.clone())
}

#[tauri::command]
async fn set_config(cfg: tauri::State<'_, AppConfig>, new_cfg: Config) -> Result<(), ()> {
    let mut cfg = cfg.0.lock().await;
    *cfg = new_cfg;
    Ok(())
}

#[tauri::command]
async fn get_lcu_info(lcu: tauri::State<'_, LCU>) -> Result<LCUClientInfo, ()> {
    let lcu = lcu.0.lock().await;
    Ok(lcu.data.clone().unwrap())
}

fn main() {
    tauri::Builder::default()
        .manage(LCU(Mutex::new(LCUState {
            connected: false,
            data: None,
        })))
        .setup(|app| {
            let app_handle = app.handle();
            let cfg_folder = app.path_resolver().app_config_dir().unwrap();
            if !cfg_folder.exists() {
                std::fs::create_dir(&cfg_folder).unwrap();
            }

            let cfg_path = cfg_folder.join("config.json");
            if !cfg_path.exists() {
                let cfg = Config {
                    auto_open: true,
                    auto_accept: true,
                    accept_delay: 2000,
                };

                let cfg_json = serde_json::to_string(&cfg).unwrap();
                std::fs::write(&cfg_path, cfg_json).unwrap();
            }

            let cfg_json = std::fs::read_to_string(&cfg_path).unwrap();
            let cfg: Config = serde_json::from_str(&cfg_json).unwrap();
            app.manage(AppConfig(Mutex::new(cfg)));

            tauri::async_runtime::spawn(async move {
                let mut connected = true;

                loop {
                    let args = shaco::utils::process_info::get_league_process_args();
                    if args.is_none() {
                        if connected {
                            println!("Waiting for League Client to open...");
                            connected = false;
                            app_handle.emit_all("lcu_state_update", false).unwrap();
                        }

                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }

                    let args = args.unwrap();
                    let lcu_info = shaco::utils::process_info::get_auth_info(args).unwrap();
                    let client = RESTClient::new(lcu_info.clone()).unwrap();
                    connected = true;
                    let lcu = app_handle.state::<LCU>();
                    app_handle.emit_all("lcu_state_update", true).unwrap();

                    let mut lcu = lcu.0.lock().await;
                    let arc = Arc::new(client);
                    lcu.connected = true;
                    lcu.data = Some(lcu_info);

                    drop(lcu);

                    // The websocket event API will not be opened until a few seconds after the client is opened.
                    let mut ws = match LcuWebsocketClient::connect().await {
                        Ok(ws) => ws,
                        Err(_) => {
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            LcuWebsocketClient::connect().await.unwrap()
                        }
                    };

                    ws.subscribe(JsonApiEvent("/lol-gameflow/v1/gameflow-phase".to_string()))
                        .await
                        .unwrap();

                    println!("Connected to League Client!");
                    let team: Lobby = serde_json::from_value(
                        arc
                            .get("/chat/v5/participants/champ-select".to_string())
                            .await
                            .unwrap(),
                    )
                    .unwrap();

                    if !team.participants.is_empty() {
                        display_champ_select(team);
                    }

                    while let Some(msg) = ws.next().await {
                        let client_state = msg.data.to_string().replace('\"', "");
                        if client_state == "ChampSelect" {
                            println!("Champ select started, grabbing team mates...");

                            tokio::time::sleep(Duration::from_secs(3)).await;
                            let team: Lobby = serde_json::from_value(
                                arc
                                    .get("/chat/v5/participants/champ-select".to_string())
                                    .await
                                    .unwrap(),
                            )
                            .unwrap();

                            display_champ_select(team);
                            app_handle.emit_all("champ_select_started", ()).unwrap();
                            continue;
                        }

                        println!("Client State Update: {}", client_state);
                        app_handle
                            .emit_all("client_state_update", client_state)
                            .unwrap();
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![app_ready, get_lcu_state, get_lcu_info, get_config, set_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
