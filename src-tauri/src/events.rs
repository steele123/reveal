//! Events the window must catch up on if it loads after they were sent, e.g. when Reveal is
//! opened in the middle of Champ Select or a game.

use serde::Serialize;
use serde_json::Value;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

/// Replayed in this order: the client state goes first, because the window starts a fresh
/// lobby (and clears the team) when it sees Champ Select begin.
const REPLAYED: [&str; 4] = [
    "client_state_update",
    "queue_name",
    "champ_select_started",
    "live_game_update",
];

static LATEST: Mutex<[Option<Value>; 4]> = Mutex::new([None, None, None, None]);

/// Sends an event to the window and remembers it for `replay`.
pub fn emit<P: Serialize + Clone>(app_handle: &AppHandle, event: &'static str, payload: P) {
    if let Ok(value) = serde_json::to_value(&payload) {
        let mut latest = LATEST.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        remember(&mut latest, event, value);
    }
    if let Err(error) = app_handle.emit_all(event, payload) {
        log_error!("Failed to emit {event}: {error}");
    }
}

/// Re-sends the latest of each event, for a window that just finished loading.
pub fn replay(app_handle: &AppHandle) {
    let latest = LATEST
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    for (event, value) in REPLAYED.iter().zip(latest) {
        if let Some(value) = value {
            if let Err(error) = app_handle.emit_all(event, value) {
                log_error!("Failed to replay {event}: {error}");
            }
        }
    }
}

fn remember(latest: &mut [Option<Value>; 4], event: &str, value: Value) {
    let Some(index) = REPLAYED.iter().position(|replayed| *replayed == event) else {
        return;
    };
    // A team from a Champ Select that has ended must not come back on the next replay.
    if event == "client_state_update" && value != "ChampSelect" {
        latest[2] = None;
    }
    latest[index] = Some(value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keeps_the_latest_of_each_event() {
        let mut latest = Default::default();
        remember(&mut latest, "client_state_update", json!("ChampSelect"));
        remember(&mut latest, "champ_select_started", json!({ "participants": [1] }));
        remember(&mut latest, "champ_select_started", json!({ "participants": [1, 2] }));
        remember(&mut latest, "lcu_state_update", json!(true));

        assert_eq!(latest[0], Some(json!("ChampSelect")));
        assert_eq!(latest[2], Some(json!({ "participants": [1, 2] })));
        assert_eq!(latest.iter().flatten().count(), 2, "unlisted events are not kept");
    }

    #[test]
    fn forgets_the_team_once_champ_select_ends() {
        let mut latest = Default::default();
        remember(&mut latest, "client_state_update", json!("ChampSelect"));
        remember(&mut latest, "champ_select_started", json!({ "participants": [1] }));
        remember(&mut latest, "client_state_update", json!("InProgress"));

        assert_eq!(latest[0], Some(json!("InProgress")));
        assert_eq!(latest[2], None);
    }
}
