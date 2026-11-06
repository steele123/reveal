use crate::{
    app_state::QueuePause,
    champ_select::{handle_champ_select_start, PollingGuard},
    config::{AppConfig, Config},
    notify::{Alert, Notice},
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
        if paused {
            crate::automation::resume_after_dodge(app_handle, remoting_client);
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

    if phase_update.ready_check_generation.is_some() {
        crate::notify::send(app_handle, match_found_notice(&cfg));
    }
    if phase_update.dodged {
        crate::notify::send(app_handle, dodge_notice(&cfg, &client_state));
    }

    crate::automation::on_phase(&client_state, app_handle, remoting_client);

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
            let app_client = app_client.clone();
            tauri::async_runtime::spawn(async move {
                crate::live_game::watch(app_handle, Some(remoting_client), Some(app_client)).await;
            });
        }
        "ReadyCheck" => {
            if let (true, Some(generation)) = (cfg.auto_accept, phase_update.ready_check_generation)
            {
                log_info!("Auto-accept is enabled; scheduling ready-check acceptance");
                let app_handle = app_handle.clone();
                let remoting_client = remoting_client.clone();
                tauri::async_runtime::spawn(async move {
                    wait_until_accept_time(&remoting_client, accept_wait(&cfg)).await;

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
                    // The wait is there so you can decline; whatever you chose stands.
                    let ready_check = remoting_client
                        .get("/lol-matchmaking/v1/ready-check".to_string())
                        .await;
                    if ready_check.as_ref().is_ok_and(answered_yourself) {
                        log_info!("Skipping auto-accept: you answered the ready check yourself");
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

/// The ready check lasts 10 seconds; accepting at 9 leaves a second for the request to land.
const MAX_ACCEPT_WAIT: std::time::Duration = std::time::Duration::from_secs(9);

/// Waits until the ready check has been up for `wait`. League's own timer counts from the
/// pop, which was before Reveal heard of it, so whichever clock is further along wins.
async fn wait_until_accept_time(remoting_client: &RESTClient, wait: std::time::Duration) {
    const POLL: std::time::Duration = std::time::Duration::from_millis(200);
    let started = std::time::Instant::now();
    loop {
        let league_elapsed = remoting_client
            .get("/lol-matchmaking/v1/ready-check".to_string())
            .await
            .ok()
            .and_then(|ready_check| ready_check["timer"].as_f64())
            .map(std::time::Duration::from_secs_f64)
            .unwrap_or_default();
        let elapsed = started.elapsed().max(league_elapsed);
        if elapsed >= wait {
            return;
        }
        tokio::time::sleep((wait - elapsed).min(POLL)).await;
    }
}

/// How long auto accept waits after the ready check pops, so you can still decline.
fn accept_wait(cfg: &Config) -> std::time::Duration {
    std::time::Duration::from_millis(u64::from(cfg.accept_delay)).min(MAX_ACCEPT_WAIT)
}

/// Whether you already accepted or declined the ready check in League.
fn answered_yourself(ready_check: &serde_json::Value) -> bool {
    matches!(
        ready_check["playerResponse"].as_str(),
        Some("Accepted" | "Declined")
    )
}

fn match_found_notice(cfg: &Config) -> Notice {
    let wait = accept_wait(cfg);
    let seconds = wait.as_secs();
    let body = if !cfg.auto_accept {
        "Accept it in League before the timer runs out.".to_string()
    } else if wait == MAX_ACCEPT_WAIT {
        "Reveal accepts it in the last second unless you answer first.".to_string()
    } else if seconds == 0 {
        "Reveal accepts it right away.".to_string()
    } else {
        format!("Reveal accepts it in {seconds} seconds unless you answer first.")
    };
    Notice::new(Alert::MatchFound, "Match found", body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn waits_as_long_as_set_but_never_past_the_ready_check() {
        let wait = |accept_delay| accept_wait(&Config { accept_delay, ..Config::default() });
        assert_eq!(wait(0).as_secs(), 0);
        assert_eq!(wait(4_000).as_secs(), 4);
        assert_eq!(wait(60_000), MAX_ACCEPT_WAIT);
    }

    #[test]
    fn leaves_ready_checks_you_answered() {
        assert!(!answered_yourself(&json!({ "playerResponse": "None", "state": "InProgress" })));
        assert!(answered_yourself(&json!({ "playerResponse": "Declined" })));
        assert!(answered_yourself(&json!({ "playerResponse": "Accepted" })));
        assert!(!answered_yourself(&json!({})));
    }

    #[test]
    fn says_when_reveal_will_accept() {
        let notice = |auto_accept, accept_delay| {
            match_found_notice(&Config { auto_accept, accept_delay, ..Config::default() }).body
        };
        assert_eq!(notice(true, 4_000), "Reveal accepts it in 4 seconds unless you answer first.");
        assert_eq!(notice(true, 0), "Reveal accepts it right away.");
        assert_eq!(notice(true, 9_000), "Reveal accepts it in the last second unless you answer first.");
        assert_eq!(notice(false, 4_000), "Accept it in League before the timer runs out.");
    }
}

/// `phase` is where the client went after Champ Select.
fn dodge_notice(cfg: &Config, phase: &str) -> Notice {
    let body = if cfg.pause_queue_after_dodge {
        "Reveal paused your queue until you start it again."
    } else if phase == "Matchmaking" {
        "You're back in queue."
    } else {
        "You're back in the lobby."
    };
    Notice::new(Alert::Dodge, "Champ Select ended without a game", body)
}

fn emit_client_state(app_handle: &AppHandle, client_state: String) {
    crate::events::emit(app_handle, "client_state_update", client_state);
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
