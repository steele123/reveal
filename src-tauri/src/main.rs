// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use shaco::rest::RESTClient;
use tauri::State;
use tokio::{time::sleep, sync::{futures, Mutex}, runtime::Builder};

struct LCU(Mutex<LCUState>);

pub struct LCUState {
    pub connected: bool,
    pub data: Option<LCUData>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct LCUData {
    pub riotClientPort: u16,
    pub lcuPort: u16,
    pub password: String,
}

#[tauri::command]
async fn is_connected(state: State<'_, LCU>) -> bool {
    let state = state.0.lock().await;
    state.connected
}

#[tauri::command]
async fn get_data(state: State<'_, LCU>) -> Option<LCUData> {
    let state = state.0.lock().await;
    state.data.clone()
}

fn main() {
    tauri::Builder::default()
        .manage(LCU(Mutex::new(LCUState {
            connected: false,
            data: None,
        })))
        .setup(|app| {
            let rt = Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            rt.spawn(async {
                let mut client = RESTClient::new();
                let mut connected = false;

                loop {
                    

                    sleep(Duration::from_secs(1)).await;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![is_connected, get_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
