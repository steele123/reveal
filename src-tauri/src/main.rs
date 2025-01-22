// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analytics;
mod champ_select;
mod commands;
mod lobby;
mod region;
mod state;
mod summoner;
mod utils;

use crate::champ_select::ChampSelectSession;
use crate::lobby::Lobby;
use crate::region::RegionInfo;
use crate::utils::display_champ_select;
use commands::{
    app_ready, dodge, enable_dodge, get_config, get_lcu_info, get_lcu_state, open_opgg_link,
    set_config,
};
use futures_util::StreamExt;
use lobby::Participant;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shaco::model::ws::LcuEvent;
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

struct ManagedDodgeState(Mutex<DodgeState>);

pub struct DodgeState {
    pub last_dodge: Option<u64>,
    pub enabled: Option<u64>,
}

struct AppConfig(Mutex<Config>);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Config {
    pub auto_open: bool,
    pub auto_accept: bool,
    pub accept_delay: u32,
    #[serde(default = "default_provider")]
    pub multi_provider: String,
}

fn default_provider() -> String {
    "opgg".to_string()
}

fn main() {
    tauri::Builder::default()
        .manage(LCU(Mutex::new(LCUState {
            connected: false,
            data: None,
        })))
        .manage(ManagedDodgeState(Mutex::new(DodgeState {
            last_dodge: None,
            enabled: None,
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
                    auto_accept: false,
                    accept_delay: 2000,
                    multi_provider: "opgg".to_string(),
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

                    let cloned_app_handle = app_handle.clone();
                    let lcu = cloned_app_handle.state::<LCU>();

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

                    ws.subscribe(JsonApiEvent("/lol-champ-select/v1/session".to_string()))
                        .await
                        .unwrap();

                    println!("Connected to League Client!");

                    let state = state::get_gameflow_state(&remoting_client).await;
                    state::handle_client_state(state, &app_handle, &remoting_client, &app_client)
                        .await;

                    while let Some(msg) = ws.next().await {
                        handle_ws_message(msg, &app_handle, &remoting_client, &app_client).await;
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
            set_config,
            open_opgg_link,
            dodge,
            enable_dodge
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn handle_ws_message(
    msg: LcuEvent,
    app_handle: &AppHandle,
    remoting_client: &RESTClient,
    app_client: &RESTClient,
) {
    let msg_type = msg.subscription_type.to_string();

    match msg_type.as_str() {
        "OnJsonApiEvent_lol-gameflow_v1_gameflow-phase" => {
            let client_state = msg.data.to_string().replace('\"', "");
            state::handle_client_state(client_state, app_handle, remoting_client, app_client).await;
        }
        "OnJsonApiEvent_lol-champ-select_v1_session" => {
            let champ_select = serde_json::from_value::<ChampSelectSession>(msg.data.clone());
            if champ_select.is_err() {
                println!("Failed to parse champ select session!, {:?}", champ_select);
                return;
            }

            let champ_select = champ_select.unwrap();
            if champ_select.timer.phase == "FINALIZATION" {
                let time = champ_select.timer.adjusted_time_left_in_phase;
                let cloned_remoting = remoting_client.clone();
                let game_id = champ_select.game_id;
                let dodge_state = app_handle.state::<ManagedDodgeState>();
                let mut dodge_state = dodge_state.0.lock().await;

                if let Some(last_dodge) = dodge_state.last_dodge {
                    if last_dodge == game_id {
                        return;
                    }
                }

                if (dodge_state.enabled.is_some() && dodge_state.enabled.unwrap() != game_id)
                    || dodge_state.enabled.is_none()
                {
                    return;
                }

                dodge_state.last_dodge = Some(game_id);
                drop(dodge_state);

                println!("Spawned task to dodge in finalization timer: {}ms", time);

                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(time)).await;
                    println!("Last second dodge calling quit endpoint...");
                    let _resp = cloned_remoting
                        .post(
                            "/lol-login/v1/session/invoke?destination=lcdsServiceProxy&method=call&args=[\"\",\"teambuilder-draft\",\"quitV2\",\"\"]".to_string(),
                            serde_json::json!({}),
                        )
                        .await
                        .unwrap();
                });
            }
        }
        _ => {
            println!("Unhandled Message: {}", msg_type);
        }
    }
}
