// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod lobby;
mod utils;

use crate::lobby::Lobby;
use crate::utils::display_champ_select;
use futures_util::StreamExt;
use shaco::model::ws::LcuSubscriptionType::JsonApiEvent;
use shaco::rest::RESTClient;
use shaco::ws::LcuWebsocketClient;
use std::time::Duration;
use tauri::Manager;
use tokio::sync::Mutex;

struct LCU(Mutex<LCUState>);

pub struct LCUState {
    pub connected: bool,
    pub data: Option<LCUData>,
}
pub struct LCUData {
    pub client: RESTClient,
}

struct AppConfig(Mutex<Config>);

struct Config {
    pub auto_open: bool,
    pub auto_accept: bool
}

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    tauri::Builder::default()
        .manage(LCU(Mutex::new(LCUState {
            connected: false,
            data: None,
        })))
        .manage(AppConfig(Mutex::new(Config {
            auto_open: false,
            auto_accept: false
        })))
        .setup(|app| {
            let app_handle = app.handle();
            tokio::task::spawn(async move {
                let mut connected = true;

                loop {
                    let client = match RESTClient::new() {
                        Ok(client) => {
                            connected = true;
                            app_handle.emit_all("lcu_state_update", true).unwrap();
                            client
                        }
                        Err(_) => {
                            if connected {
                                println!(
                                    "Lost connection to league client, trying to reconnect..."
                                );
                                connected = false;
                                app_handle.emit_all("lcu_state_update", false).unwrap();
                            }

                            continue;
                        }
                    };

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
                        client
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
                                client
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
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
