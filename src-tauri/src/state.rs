use crate::{champ_select::handle_champ_select_start, config::AppConfig};
use shaco::rest::RESTClient;
use tauri::{AppHandle, Manager};

pub async fn get_gameflow_state(remoting_client: &RESTClient) -> anyhow::Result<String> {
    let gameflow_state = remoting_client
        .get("/lol-gameflow/v1/gameflow-phase".to_string())
        .await?;

    Ok(gameflow_state.to_string().replace('\"', ""))
}

pub async fn handle_client_state(
    client_state: String,
    app_handle: &AppHandle,
    remoting_client: &RESTClient,
    app_client: &RESTClient,
) {
    log_info!("League Client state changed to {client_state}");
    match client_state.as_str() {
        "ChampSelect" => {
            let cloned_app_handle = app_handle.clone();
            let cloned_app_client = app_client.clone();
            let cloned_remoting = remoting_client.clone();

            // clone config and pass to async task
            let cfg = app_handle.state::<AppConfig>();
            let cfg = cfg.0.lock().await.clone();

            tauri::async_runtime::spawn(async move {
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
            let cfg = {
                let cfg = app_handle.state::<AppConfig>();
                let value = cfg.0.lock().await.clone();
                value
            };
            if cfg.auto_accept {
                log_info!("Auto-accept is enabled; scheduling ready-check acceptance");
                tokio::time::sleep(std::time::Duration::from_millis(
                    u64::from(cfg.accept_delay).saturating_sub(1_000),
                ))
                .await;
                if let Err(error) = remoting_client
                    .post(
                        "/lol-matchmaking/v1/ready-check/accept".to_string(),
                        serde_json::json!({}),
                    )
                    .await
                {
                    log_error!("Ready-check auto-accept failed: {error}");
                } else {
                    log_info!("Ready check accepted automatically");
                }
            }
        }
        _ => {}
    }

    if let Err(error) = app_handle.emit_all("client_state_update", client_state) {
        log_error!("Failed to emit League Client state: {error}");
    }
}
