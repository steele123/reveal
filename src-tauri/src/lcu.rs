use crate::app_state::{Dodge, Lcu};
use crate::champ_select::ChampSelectSession;
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
                println!("Waiting for League Client to open...");
                was_connected = false;
                set_connection_state(&app_handle, false, None).await;
            }

            tokio::time::sleep(CLIENT_POLL_INTERVAL).await;
            continue;
        };

        let lcu_info = match process_info::get_auth_info(args) {
            Ok(info) => info,
            Err(error) => {
                eprintln!("Failed to read League Client connection info: {error}");
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
        set_connection_state(&app_handle, true, Some(lcu_info)).await;

        let Some(mut websocket) = connect_websocket().await else {
            eprintln!("Failed to connect to the League Client websocket");
            tokio::time::sleep(CLIENT_POLL_INTERVAL).await;
            continue;
        };

        if let Err(error) = websocket
            .subscribe(JsonApiEvent("/lol-gameflow/v1/gameflow-phase".to_string()))
            .await
        {
            eprintln!("Failed to subscribe to gameflow updates: {error}");
            continue;
        }

        if let Err(error) = websocket
            .subscribe(JsonApiEvent("/lol-champ-select/v1/session".to_string()))
            .await
        {
            eprintln!("Failed to subscribe to Champ Select updates: {error}");
            continue;
        }

        println!("Connected to League Client!");

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
            Err(error) => eprintln!("Failed to read initial gameflow state: {error}"),
        }

        while let Some(message) = websocket.next().await {
            handle_websocket_message(message, &app_handle, &remoting_client, &app_client).await;
        }
    }
}

fn create_rest_client(lcu_info: shaco::rest::LCUClientInfo, remoting: bool) -> Option<RESTClient> {
    match RESTClient::new(lcu_info, remoting) {
        Ok(client) => Some(client),
        Err(error) => {
            eprintln!("Failed to create League Client API client: {error}");
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
            Ok(websocket) => return Some(websocket),
            Err(error) if attempt == WEBSOCKET_ATTEMPTS => {
                eprintln!("League websocket connection failed: {error}");
            }
            Err(_) => {}
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

    if let Err(error) = app_handle.emit_all("lcu_state_update", connected) {
        eprintln!("Failed to emit League connection state: {error}");
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
        "OnJsonApiEvent_lol-champ-select_v1_session" => {
            handle_last_second_dodge(message, app_handle, remoting_client).await;
        }
        _ => println!("Unhandled Message: {message_type}"),
    }
}

async fn handle_last_second_dodge(
    message: LcuEvent,
    app_handle: &AppHandle,
    remoting_client: &RESTClient,
) {
    let champ_select = match serde_json::from_value::<ChampSelectSession>(message.data) {
        Ok(session) => session,
        Err(error) => {
            eprintln!("Failed to parse Champ Select session: {error}");
            return;
        }
    };

    if champ_select.timer.phase != "FINALIZATION" {
        return;
    }

    let game_id = champ_select.game_id;
    let dodge = app_handle.state::<Dodge>();
    let mut dodge = dodge.0.lock().await;
    if dodge.last_dodge == Some(game_id) || dodge.enabled != Some(game_id) {
        return;
    }

    dodge.last_dodge = Some(game_id);
    drop(dodge);

    let delay = Duration::from_millis(champ_select.timer.adjusted_time_left_in_phase);
    let remoting_client = remoting_client.clone();
    println!("Spawned task to dodge in finalization timer: {delay:?}");

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(delay).await;
        println!("Last second dodge calling quit endpoint...");
        if let Err(error) = remoting_client
            .post(
                "/lol-login/v1/session/invoke?destination=lcdsServiceProxy&method=call&args=[\"\",\"teambuilder-draft\",\"quitV2\",\"\"]".to_string(),
                serde_json::json!({}),
            )
            .await
        {
            eprintln!("Last second dodge failed: {error}");
        }
    });
}
