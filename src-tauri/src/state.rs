use crate::{
    app_state::QueuePause,
    champ_select::{handle_champ_select_start, PollingGuard},
    config::AppConfig,
};
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
    let cfg = {
        let cfg = app_handle.state::<AppConfig>();
        let value = cfg.0.lock().await.clone();
        value
    };
    let phase_update = {
        let pause = app_handle.state::<QueuePause>();
        let update = pause
            .0
            .lock()
            .await
            .observe_phase(&client_state, cfg.pause_queue_after_dodge);
        update
    };

    if let Some(paused) = phase_update.pause_changed {
        log_info!("Queue pause after dodge changed to {paused}");
        if let Err(error) = app_handle.emit_all("queue_pause_update", paused) {
            log_error!("Failed to emit queue pause state: {error}");
        }
    }

    if phase_update.cancel_search {
        log_info!("Stopping automatic matchmaking search after a dodge");
        let app_handle = app_handle.clone();
        let remoting_client = remoting_client.clone();
        tauri::async_runtime::spawn(async move {
            stop_matchmaking_after_dodge(&app_handle, &remoting_client).await;
        });
    }

    match client_state.as_str() {
        "ChampSelect" => {
            let Some(polling_guard) = PollingGuard::acquire() else {
                log_info!("Champ Select participant polling is already running");
                return emit_client_state(app_handle, client_state);
            };
            let cloned_app_handle = app_handle.clone();
            let cloned_app_client = app_client.clone();
            let cloned_remoting = remoting_client.clone();

            tauri::async_runtime::spawn(async move {
                let _polling_guard = polling_guard;
                handle_champ_select_start(
                    &cloned_app_client,
                    &cloned_remoting,
                    &cfg,
                    &cloned_app_handle,
                )
                .await;
            });
        }
        "InProgress" | "Reconnect" => {
            let app_handle = app_handle.clone();
            let remoting_client = remoting_client.clone();
            tauri::async_runtime::spawn(async move {
                crate::live_game::watch(app_handle, remoting_client).await;
            });
        }
        "ReadyCheck" => {
            if let (true, Some(generation)) = (cfg.auto_accept, phase_update.ready_check_generation)
            {
                log_info!("Auto-accept is enabled; scheduling ready-check acceptance");
                let app_handle = app_handle.clone();
                let remoting_client = remoting_client.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(
                        u64::from(cfg.accept_delay).saturating_sub(1_000),
                    ))
                    .await;

                    let ready_check_is_current = {
                        let pause = app_handle.state::<QueuePause>();
                        let value = pause.0.lock().await.ready_check_is_current(generation);
                        value
                    };
                    let auto_accept = {
                        let cfg = app_handle.state::<AppConfig>();
                        let value = cfg.0.lock().await.auto_accept;
                        value
                    };
                    let phase = get_gameflow_state(&remoting_client).await;
                    let still_current = {
                        let pause = app_handle.state::<QueuePause>();
                        let value = pause.0.lock().await.ready_check_is_current(generation);
                        value
                    };
                    if !ready_check_is_current
                        || !still_current
                        || !auto_accept
                        || !matches!(phase.as_deref(), Ok("ReadyCheck"))
                    {
                        log_info!("Skipping stale or paused ready-check auto-accept");
                        return;
                    }

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
                });
            }
        }
        _ => {}
    }

    emit_client_state(app_handle, client_state);
}

fn emit_client_state(app_handle: &AppHandle, client_state: String) {
    if let Err(error) = app_handle.emit_all("client_state_update", client_state) {
        log_error!("Failed to emit League Client state: {error}");
    }
}

async fn stop_matchmaking_after_dodge(app_handle: &AppHandle, remoting_client: &RESTClient) {
    for attempt in 1..=3 {
        if attempt > 1 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let still_paused_in_search = {
                let pause = app_handle.state::<QueuePause>();
                let value = pause.0.lock().await.should_cancel_search();
                value
            };
            if !still_paused_in_search
                || !matches!(
                    get_gameflow_state(remoting_client).await.as_deref(),
                    Ok("Matchmaking")
                )
            {
                return;
            }
        }

        match remoting_client
            .delete("/lol-lobby/v2/lobby/matchmaking/search".to_string())
            .await
        {
            Ok(_) => {
                log_info!("Stopped matchmaking after a dodge");
                return;
            }
            Err(error) => {
                log_warn!("Attempt {attempt}/3 to stop matchmaking after a dodge failed: {error}")
            }
        }
    }
    log_error!("Could not stop matchmaking after a dodge; auto-accept remains paused");
}
