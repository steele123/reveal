// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod lobby;
mod utils;

use crate::lobby::Lobby;
use crate::utils::display_champ_select;

use futures_util::StreamExt;
use shaco::rest::RESTClient;
use shaco::utils::process_info;
use shaco::ws::LcuWebsocketClient;
use shaco::{model::ws::LcuSubscriptionType::JsonApiEvent, rest::LCUClientInfo};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

struct LCU(Mutex<LCUState>);

pub struct LCUState {
    pub connected: bool,
    pub data: Option<LCUClientInfo>,
}

struct AppConfig(Mutex<Config>);

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
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
    println!("App Ready!");
    let lcu = lcu.0.lock().await;
    let cfg = cfg.0.lock().await;

    println!("LCU State: {}", lcu.connected);
    println!("Config: {:?}", cfg);

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

#[tauri::command(rename_all = "snake_case")]
async fn set_config(
    cfg: tauri::State<'_, AppConfig>,
    new_cfg: Config,
    app_handle: AppHandle,
) -> Result<(), ()> {
    let mut cfg = cfg.0.lock().await;
    *cfg = new_cfg;

    // Save config to disk
    let cfg_folder = app_handle.path_resolver().app_config_dir().unwrap();
    let cfg_path = cfg_folder.join("config.json");
    let cfg_json = serde_json::to_string(&cfg.clone()).unwrap();
    tokio::fs::write(&cfg_path, cfg_json).await.unwrap();

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
                    let args = process_info::get_league_process_args();
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

                    let lcu_info = process_info::get_auth_info(args).unwrap();
                    let app_client = RESTClient::new(lcu_info.clone(), false).unwrap();
                    let remoting_client = RESTClient::new(lcu_info.clone(), true).unwrap();

                    let lcu = app_handle.state::<LCU>();

                    connected = true;
                    app_handle.emit_all("lcu_state_update", true).unwrap();

                    let mut lcu = lcu.0.lock().await;
                    lcu.connected = true;
                    lcu.data = Some(lcu_info);

                    drop(lcu);

                    // The websocket event API will not be opened until a few seconds after the client is opened.
                    let mut ws = match LcuWebsocketClient::connect().await {
                        Ok(ws) => ws,
                        Err(_) => {
                            let mut attempts = 0;
                            loop {
                                tokio::time::sleep(Duration::from_secs(3)).await;
                                if attempts > 5 {
                                    panic!("Failed to connect to League Client!");
                                }

                                attempts += 1;
                                match LcuWebsocketClient::connect().await {
                                    Ok(ws) => break ws,
                                    Err(_) => continue,
                                }
                            }
                        }
                    };

                    ws.subscribe(JsonApiEvent("/lol-gameflow/v1/gameflow-phase".to_string()))
                        .await
                        .unwrap();

                    println!("Connected to League Client!");
                    let team: Lobby = serde_json::from_value(
                        app_client
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
                                app_client
                                    .get("/chat/v5/participants/champ-select".to_string())
                                    .await
                                    .unwrap(),
                            )
                            .unwrap();

                            app_handle.emit_all("champ_select_started", &team).unwrap();

                            let player_cid = &team.participants[0].cid;
                            let _resp = remoting_client
                                .post(
                                    format!("/lol-chat/v1/conversations/{}/messages", player_cid),
                                    serde_json::json!({
                                        "body": "Champ select started!",
                                        "type": "chat"
                                    }),
                                )
                                .await
                                .unwrap();

                            let cfg = app_handle.state::<AppConfig>();
                            let cfg = cfg.0.lock().await;
                            if cfg.auto_open {
                                println!("{}", cfg.auto_open);
                                display_champ_select(team);
                            }
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
        .invoke_handler(tauri::generate_handler![
            app_ready,
            get_lcu_state,
            get_lcu_info,
            get_config,
            set_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
