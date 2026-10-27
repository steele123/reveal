use crate::app_state::{Lcu, QueuePause};
use crate::state;
use futures_util::StreamExt;
use shaco::model::ws::{LcuEvent, LcuSubscriptionType::JsonApiEvent};
use shaco::rest::RESTClient;
use shaco::utils::process_info;
use shaco::ws::LcuWebsocketClient;
use std::time::Duration;
use tauri::{AppHandle, Manager};

const CLIENT_POLL_INTERVAL: Duration = Duration::from_secs(2);
const WEBSOCKET_RETRY_INTERVAL: Duration = Duration::from_secs(3);
const WEBSOCKET_ATTEMPTS: usize = 7;

pub async fn run(app_handle: AppHandle) {
    let mut was_connected = true;

    loop {
        let Some(args) = process_info::get_league_process_args() else {
            if was_connected {
                log_info!("League Client is not running; waiting for it to open");
                was_connected = false;
                set_connection_state(&app_handle, false, None).await;
            }

            tokio::time::sleep(CLIENT_POLL_INTERVAL).await;
            continue;
        };

        let lcu_info = match process_info::get_auth_info(args) {
            Ok(info) => info,
            Err(error) => {
                log_warn!("Failed to read League Client connection info: {error}");
                tokio::time::sleep(CLIENT_POLL_INTERVAL).await;
                continue;
            }
        };

        let Some(app_client) = create_rest_client(lcu_info.clone(), false) else {
            tokio::time::sleep(CLIENT_POLL_INTERVAL).await;
            continue;
        };
        let Some(remoting_client) = create_rest_client(lcu_info.clone(), true) else {
            tokio::time::sleep(CLIENT_POLL_INTERVAL).await;
            continue;
        };

        was_connected = true;
        crate::live_game::remember_client(&lcu_info);
        set_connection_state(&app_handle, true, Some(lcu_info)).await;

        let Some(mut websocket) = connect_websocket().await else {
            log_error!("League Client websocket retries were exhausted");
            tokio::time::sleep(CLIENT_POLL_INTERVAL).await;
            continue;
        };

        if let Err(error) = websocket
            .subscribe(JsonApiEvent("/lol-gameflow/v1/gameflow-phase".to_string()))
            .await
        {
            log_error!("Failed to subscribe to gameflow updates: {error}");
            continue;
        }

        log_info!("Connected to League Client");

        match state::get_gameflow_state(&remoting_client).await {
            Ok(client_state) => {
                state::handle_client_state(
                    client_state,
                    &app_handle,
                    &remoting_client,
                    &app_client,
                )
                .await;
            }
            Err(error) => log_warn!("Failed to read initial gameflow state: {error}"),
        }

        while let Some(message) = websocket.next().await {
            handle_websocket_message(message, &app_handle, &remoting_client, &app_client).await;
        }
        log_warn!("League Client websocket closed; reconnecting");
    }
}

fn create_rest_client(lcu_info: shaco::rest::LCUClientInfo, remoting: bool) -> Option<RESTClient> {
    match RESTClient::new(lcu_info, remoting) {
        Ok(client) => Some(client),
        Err(error) => {
            log_warn!("Failed to create League Client API client: {error}");
            None
        }
    }
}

async fn connect_websocket() -> Option<LcuWebsocketClient> {
    for attempt in 1..=WEBSOCKET_ATTEMPTS {
        if attempt > 1 {
            tokio::time::sleep(WEBSOCKET_RETRY_INTERVAL).await;
        }

        match LcuWebsocketClient::connect().await {
            Ok(websocket) => {
                log_info!("League Client websocket connected on attempt {attempt}");
                return Some(websocket);
            }
            Err(error) if attempt == WEBSOCKET_ATTEMPTS => {
                log_error!("League websocket connection failed: {error}");
            }
            Err(error) => {
                log_warn!(
                    "League websocket connection attempt {attempt}/{WEBSOCKET_ATTEMPTS} failed: {error}"
                );
            }
        }
    }

    None
}

async fn set_connection_state(
    app_handle: &AppHandle,
    connected: bool,
    data: Option<shaco::rest::LCUClientInfo>,
) {
    let lcu = app_handle.state::<Lcu>();
    let mut lcu = lcu.0.lock().await;
    lcu.connected = connected;
    lcu.data = data;
    drop(lcu);

    if !connected {
        let pause = app_handle.state::<QueuePause>();
        let pause_changed = pause.0.lock().await.reset();
        if let Some(paused) = pause_changed {
            if let Err(error) = app_handle.emit_all("queue_pause_update", paused) {
                log_error!("Failed to emit queue pause state: {error}");
            }
        }
    }

    if let Err(error) = app_handle.emit_all("lcu_state_update", connected) {
        log_error!("Failed to emit League connection state: {error}");
    }
}

async fn handle_websocket_message(
    message: LcuEvent,
    app_handle: &AppHandle,
    remoting_client: &RESTClient,
    app_client: &RESTClient,
) {
    let message_type = message.subscription_type.to_string();

    match message_type.as_str() {
        "OnJsonApiEvent_lol-gameflow_v1_gameflow-phase" => {
            let client_state = message.data.to_string().replace('"', "");
            state::handle_client_state(client_state, app_handle, remoting_client, app_client).await;
        }
        _ => log_warn!("Unhandled League websocket message: {message_type}"),
    }
}
