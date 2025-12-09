use crate::{champ_select::handle_champ_select_start, AppConfig};
use shaco::rest::RESTClient;
use tauri::{AppHandle, Manager};

pub async fn get_gameflow_state(remoting_client: &RESTClient) -> String {
    let gameflow_state = remoting_client
        .get("/lol-gameflow/v1/gameflow-phase".to_string())
        .await
        .unwrap()
        .to_string();

    let cleaned_state = gameflow_state.replace('\"', "");
    cleaned_state
}

pub async fn handle_client_state(
    client_state: String,
    app_handle: &AppHandle,
    remoting_client: &RESTClient,
    app_client: &RESTClient,
) {
    match client_state.as_str() {
        "ChampSelect" => {
            let cloned_app_handle = app_handle.clone();
            let cloned_app_client = app_client.clone();
            let cloned_remoting = remoting_client.clone();

            tauri::async_runtime::spawn(async move {
                let cfg = cloned_app_handle.state::<AppConfig>();
                let cfg = cfg.0.lock().await;
                handle_champ_select_start(
                    &cloned_app_client,
                    &cloned_remoting,
                    &cfg,
                    &cloned_app_handle,
                )
                .await;
            });
        }
        "ReadyCheck" => {
            let cfg = app_handle.state::<AppConfig>();
            let cfg = cfg.0.lock().await;
            if cfg.auto_accept {
                tokio::time::sleep(std::time::Duration::from_millis(
                    (cfg.accept_delay as u64) - 1000,
                ))
                .await;
                let _resp = remoting_client
                    .post(
                        "/lol-matchmaking/v1/ready-check/accept".to_string(),
                        serde_json::json!({}),
                    )
                    .await;
            }
        }
        _ => {}
    }

    println!("Client State Update: {}", client_state);
    app_handle
        .emit_all("client_state_update", client_state)
        .unwrap();
}
