//! Hands-off queueing and last-second bans and picks.
//!
//! Each watcher is started when the client enters its phase and reads the settings again before
//! acting, so turning something off takes effect immediately.

use crate::{
    app_state::QueuePause,
    config::{AppConfig, Config},
    notify::{self, Alert, Notice},
};
use serde::Deserialize;
use serde_json::json;
use shaco::rest::RESTClient;
use std::{
    collections::{HashMap, HashSet},
    future::Future,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const QUEUE_POLL_INTERVAL: Duration = Duration::from_secs(2);
const DRAFT_POLL_INTERVAL: Duration = Duration::from_millis(200);
/// "The last second": bans and picks happen once this little of your turn is left, so you get the
/// rest of it. It leaves room for one missed poll and League's round trip. Earlier is a setting.
const LAST_SECONDS_MS: f64 = 1_200.0;
/// A ban or pick League turns down (or that times out) is tried again on the next poll, up to this
/// many times, while the turn lasts.
const MAX_ACTION_ATTEMPTS: u32 = 3;
/// With nothing to choose, Reveal keeps checking (a hover may still come in) and only says it
/// couldn't once this little of the turn is left.
const NOTHING_TO_CHOOSE_MS: f64 = 300.0;
const LOBBY_WAIT: Duration = Duration::from_secs(10);

static DRAFT_WATCH: AtomicBool = AtomicBool::new(false);
static QUEUE_WATCH: AtomicBool = AtomicBool::new(false);
static REQUEUE_WAIT: AtomicBool = AtomicBool::new(false);
/// Counts pauses after a dodge, so a timer only ends the pause it was started for.
static DODGE_PAUSES: AtomicU64 = AtomicU64::new(0);

/// Starts the watcher for a phase the client just entered, unless one is already running.
pub fn on_phase(phase: &str, app_handle: &AppHandle, remoting_client: &RESTClient) {
    let (app_handle, client) = (app_handle.clone(), remoting_client.clone());
    match phase {
        "ChampSelect" => spawn_once(&DRAFT_WATCH, watch_draft(app_handle, client)),
        "Matchmaking" => spawn_once(&QUEUE_WATCH, watch_queue(app_handle, client)),
        "EndOfGame" => spawn_once(&REQUEUE_WAIT, requeue_after_game(app_handle, client)),
        _ => {}
    }
}

fn spawn_once<F>(running: &'static AtomicBool, task: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    if running
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    tauri::async_runtime::spawn(async move {
        task.await;
        running.store(false, Ordering::Release);
    });
}

async fn settings(app_handle: &AppHandle) -> Config {
    let config = app_handle.state::<AppConfig>();
    let value = config.0.lock().await.clone();
    value
}

async fn phase(client: &RESTClient) -> Option<String> {
    client
        .get("/lol-gameflow/v1/gameflow-phase".to_string())
        .await
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
}

/// Waits `seconds` while the window counts down to the next queue. `after` says what the wait
/// follows ("game", "queue" or "dodge"), so the window can drop the countdown if you move on
/// first.
async fn requeue_countdown(app_handle: &AppHandle, after: &'static str, seconds: u32) {
    let delay = Duration::from_secs(u64::from(seconds));
    if delay.is_zero() {
        return;
    }
    let deadline = SystemTime::now() + delay;
    let deadline_ms = deadline.duration_since(UNIX_EPOCH).map_or(0, |time| time.as_millis() as u64);
    let _ = app_handle.emit_all("requeue_countdown", json!({ "deadline": deadline_ms, "after": after }));
    tokio::time::sleep(delay).await;
    let _ = app_handle.emit_all("requeue_countdown", serde_json::Value::Null);
}

async fn requeue_after_game(app_handle: AppHandle, client: RESTClient) {
    if !settings(&app_handle).await.auto_requeue {
        return;
    }
    let seconds = settings(&app_handle).await.requeue_delay_seconds;
    requeue_countdown(&app_handle, "game", seconds).await;
    if !settings(&app_handle).await.auto_requeue {
        return;
    }
    match phase(&client).await.as_deref() {
        Some("EndOfGame") => {
            if let Err(error) = client
                .post_no_body("/lol-lobby/v2/play-again".to_string())
                .await
            {
                log_warn!("Couldn't return to the lobby after the game: {error}");
                return;
            }
            wait_for_lobby(&client).await;
        }
        Some("Lobby") => {}
        // Already queued, or gone somewhere else.
        _ => return,
    }
    start_search(
        &app_handle,
        &client,
        "after the game",
        "Reveal started the same queue again after your game.",
    )
    .await;
}

/// Ends a pause after a dodge once the time you set has passed, and queues again. Does nothing
/// when the pause lasts until you queue yourself.
pub fn resume_after_dodge(app_handle: &AppHandle, client: &RESTClient) {
    let (app_handle, client) = (app_handle.clone(), client.clone());
    let pause_number = DODGE_PAUSES.fetch_add(1, Ordering::AcqRel) + 1;
    tauri::async_runtime::spawn(async move {
        let seconds = settings(&app_handle).await.dodge_pause_seconds;
        if seconds == 0 {
            return;
        }
        requeue_countdown(&app_handle, "dodge", seconds).await;
        if DODGE_PAUSES.load(Ordering::Acquire) != pause_number {
            return;
        }
        let ended = {
            let pause = app_handle.state::<QueuePause>();
            let mut pause = pause.0.lock().await;
            // You queued yourself or turned the pause off in the meantime.
            if !pause.paused() {
                return;
            }
            pause.disable()
        };
        if let Some(paused) = ended {
            let _ = app_handle.emit_all("queue_pause_update", paused);
        }
        log_info!("Pause after the dodge is over; queueing again");
        start_search(
            &app_handle,
            &client,
            "after the dodge pause",
            "The pause after the dodge is over, so Reveal started the queue again.",
        )
        .await;
    });
}

async fn wait_for_lobby(client: &RESTClient) {
    let started = std::time::Instant::now();
    while started.elapsed() < LOBBY_WAIT {
        if phase(client).await.as_deref() == Some("Lobby") {
            return;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// Starts matchmaking from the lobby, unless the queue is paused after a dodge. `reason` is for
/// the log, `explanation` for the alert.
async fn start_search(app_handle: &AppHandle, client: &RESTClient, reason: &str, explanation: &str) {
    if phase(client).await.as_deref() != Some("Lobby") {
        return;
    }
    let paused = {
        let pause = app_handle.state::<QueuePause>();
        let value = pause.0.lock().await.paused();
        value
    };
    if paused {
        log_info!("Not queueing {reason}: the queue is paused after a dodge");
        return;
    }
    match client
        .post_no_body("/lol-lobby/v2/lobby/matchmaking/search".to_string())
        .await
    {
        Ok(_) => {
            log_info!("Queued again {reason}");
            notify::send(app_handle, Notice::new(Alert::AutoQueue, "Queued again", explanation));
        }
        Err(error) => {
            log_warn!("Couldn't queue again {reason}: {error}");
            notify::send(
                app_handle,
                Notice::new(
                    Alert::AutoQueue,
                    "Couldn't queue again",
                    "League turned it down. In a party, only the leader can start the queue.",
                ),
            );
        }
    }
}

/// Leaves and rejoins the queue once the wait runs well past League's estimate.
async fn watch_queue(app_handle: AppHandle, client: RESTClient) {
    loop {
        tokio::time::sleep(QUEUE_POLL_INTERVAL).await;
        if phase(&client).await.as_deref() != Some("Matchmaking") {
            return;
        }
        if !settings(&app_handle).await.requeue_past_estimate {
            continue;
        }
        let Ok(search) = client.get("/lol-matchmaking/v1/search".to_string()).await else {
            continue;
        };
        let (Some(waited), Some(estimate)) = (
            search["timeInQueue"].as_f64(),
            search["estimatedQueueTime"].as_f64(),
        ) else {
            continue;
        };
        if !still_searching(&search) || !past_estimate(waited, estimate) {
            continue;
        }
        // Leaving the queue while a match pops would decline its ready check, so look once more
        // right before leaving.
        let latest = client.get("/lol-matchmaking/v1/search".to_string()).await;
        if !latest.as_ref().is_ok_and(still_searching)
            || phase(&client).await.as_deref() != Some("Matchmaking")
        {
            continue;
        }

        log_info!("Queue ran well past its {estimate:.0}s estimate ({waited:.0}s); restarting it");
        if let Err(error) = client
            .delete("/lol-lobby/v2/lobby/matchmaking/search".to_string())
            .await
        {
            log_warn!("Couldn't leave the long queue: {error}");
            return;
        }
        let seconds = settings(&app_handle).await.long_queue_requeue_delay_seconds;
        requeue_countdown(&app_handle, "queue", seconds).await;
        if !settings(&app_handle).await.requeue_past_estimate {
            return;
        }
        // Keep watching the new search; the loop ends if it didn't start.
        start_search(
            &app_handle,
            &client,
            "after a long queue",
            "Your wait ran past League's estimate, so Reveal restarted the queue.",
        )
        .await;
    }
}

/// Still looking for a match: none found and no ready check up.
fn still_searching(search: &serde_json::Value) -> bool {
    search["searchState"].as_str() == Some("Searching")
        // "Invalid" is League's word for no ready check; any other state means one is up.
        && matches!(search["readyCheck"]["state"].as_str(), None | Some("Invalid"))
}

/// League's estimates run short, so only restart once the wait is half again as long, and at
/// least a minute past it; restarting throws away the time already waited.
fn past_estimate(waited: f64, estimate: f64) -> bool {
    estimate > 0.0 && waited > (estimate * 1.5).max(estimate + 60.0)
}

/// Announces your ban and pick turns, and bans and locks in during their last seconds if you
/// haven't yet.
async fn watch_draft(app_handle: AppHandle, client: RESTClient) {
    let mut attempts = TurnAttempts::default();
    let mut announced: HashSet<i64> = HashSet::new();
    loop {
        tokio::time::sleep(DRAFT_POLL_INTERVAL).await;
        if phase(&client).await.as_deref() != Some("ChampSelect") {
            return;
        }
        let settings = settings(&app_handle).await;
        let automatic = settings.auto_ban || settings.auto_pick;
        let announce = notify::wanted(&settings.notifications, Alert::YourTurn);
        if !automatic && !announce {
            continue;
        }
        let Ok(value) = client
            .get("/lol-champ-select/v1/session".to_string())
            .await
        else {
            continue;
        };
        let Ok(session) = serde_json::from_value::<DraftSession>(value) else {
            continue;
        };
        let now = now_ms();
        if announce {
            if let Some(turn) = current_turn(&session) {
                if announced.insert(turn.id) {
                    notify::send(&app_handle, turn_notice(&session, turn, &settings, now));
                }
            }
        }
        if !automatic {
            continue;
        }
        let Some(turn) = due_turn(&session, now, &settings) else {
            continue;
        };
        if attempts.is_finished(turn.id) {
            continue;
        }

        let available = available_champions(&client, &turn.kind).await;
        let Some(champion_id) = choose_champion(&session, turn, &settings, available.as_ref())
        else {
            // A hover may still come in, so keep looking until the very end of the turn.
            if time_left_ms(&session, now) > NOTHING_TO_CHOOSE_MS || !attempts.give_up(turn.id) {
                continue;
            }
            log_info!(
                "No champion to {} automatically: hovering {}, {} champions available, ban list {:?}, picks {:?}",
                turn.kind,
                turn.champion_id,
                available.as_ref().map_or("unknown".to_string(), |ids| ids.len().to_string()),
                settings.ban_champions,
                settings.pick_champions
            );
            let enabled = if turn.kind == "ban" { settings.auto_ban } else { settings.auto_pick };
            if enabled {
                let list = if turn.kind == "ban" { "ban list" } else { "picks for your role" };
                notify::send(
                    &app_handle,
                    Notice::new(
                        Alert::AutoAction,
                        format!("Nothing to {} automatically", turn.kind),
                        format!("None of your {list} was available, so League's timer ran out."),
                    ),
                );
            }
            continue;
        };
        match complete_action(&client, turn.id, champion_id).await {
            Ok(()) => {
                attempts.give_up(turn.id);
                log_info!("Automatic {} of champion {champion_id}", turn.kind);
                let name = champion_name(&client, champion_id).await;
                let done = if turn.kind == "ban" { "Banned" } else { "Locked in" };
                notify::send(
                    &app_handle,
                    Notice::new(
                        Alert::AutoAction,
                        format!("{done} {name}"),
                        format!("Reveal did it {}.", describe_when(&settings, &turn.kind)),
                    ),
                );
            }
            Err(error) if !attempts.failed(turn.id) => {
                log_info!("Automatic {} failed, trying again: {error}", turn.kind);
            }
            Err(error) => {
                log_warn!("Automatic {} failed {MAX_ACTION_ATTEMPTS} times: {error}", turn.kind);
                notify::send(
                    &app_handle,
                    Notice::new(
                        Alert::AutoAction,
                        format!("Automatic {} failed", turn.kind),
                        "League turned it down; the log has the details.",
                    ),
                );
            }
        }
    }
}

/// What's been tried for each ban or pick, so a failed one is retried while its turn lasts.
#[derive(Default)]
struct TurnAttempts {
    finished: HashSet<i64>,
    failures: HashMap<i64, u32>,
}

impl TurnAttempts {
    fn is_finished(&self, action_id: i64) -> bool {
        self.finished.contains(&action_id)
    }

    /// Stops trying this turn (done, or nothing to do). False if it was already stopped.
    fn give_up(&mut self, action_id: i64) -> bool {
        self.finished.insert(action_id)
    }

    /// Counts a failed try. False once that was the last one, and the turn is given up.
    fn failed(&mut self, action_id: i64) -> bool {
        let failures = self.failures.entry(action_id).or_default();
        *failures += 1;
        if *failures < MAX_ACTION_ATTEMPTS {
            return true;
        }
        self.give_up(action_id);
        false
    }
}

/// How much of the turn is left when Reveal bans or locks in, in milliseconds.
fn act_at_ms(settings: &Config, kind: &str) -> f64 {
    let seconds = if kind == "ban" {
        settings.auto_ban_seconds_left
    } else {
        settings.auto_pick_seconds_left
    };
    (f64::from(seconds) * 1_000.0).max(LAST_SECONDS_MS)
}

/// "in the last second" or "with 5 seconds left".
fn describe_when(settings: &Config, kind: &str) -> String {
    let seconds = if kind == "ban" {
        settings.auto_ban_seconds_left
    } else {
        settings.auto_pick_seconds_left
    };
    if seconds <= 1 {
        "in the last second".to_string()
    } else {
        format!("with {seconds} seconds left")
    }
}

/// "Your turn to pick", with the time left and whether Reveal will step in.
fn turn_notice(session: &DraftSession, turn: &DraftAction, settings: &Config, now_ms: f64) -> Notice {
    let seconds = (time_left_ms(session, now_ms) / 1_000.0).round().max(0.0);
    let when = describe_when(settings, &turn.kind);
    let (automatic, fallback) = if turn.kind == "ban" {
        (settings.auto_ban, format!("Reveal bans from your list {when} if you don't."))
    } else {
        (settings.auto_pick, format!("Reveal locks in your hover or list {when} if you don't."))
    };
    let mut body = format!("{seconds:.0} seconds left.");
    if automatic {
        body.push(' ');
        body.push_str(&fallback);
    }
    Notice::new(Alert::YourTurn, format!("Your turn to {}", turn.kind), body)
}

async fn champion_name(client: &RESTClient, champion_id: i64) -> String {
    let names = crate::champions::get_champion_names(client).await.ok();
    i32::try_from(champion_id)
        .ok()
        .and_then(|id| names?.remove(&id))
        .unwrap_or_else(|| format!("champion {champion_id}"))
}

fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as f64)
        .unwrap_or_default()
}

/// The champions you own and could pick now; None if League won't say, or for bans, where its
/// list of bannable champions comes back with a single id even in ranked. Bans already made are
/// skipped either way, and League turns down a ban it doesn't allow.
async fn available_champions(client: &RESTClient, kind: &str) -> Option<HashSet<i64>> {
    if kind == "ban" {
        return None;
    }
    let ids: Vec<i64> = serde_json::from_value(
        client
            .get("/lol-champ-select/v1/pickable-champion-ids".to_string())
            .await
            .ok()?,
    )
    .ok()?;
    // Some modes answer with an empty list, which means it doesn't know, not that nothing is allowed.
    (!ids.is_empty()).then(|| ids.into_iter().collect())
}

/// Chooses the champion and locks the ban or pick in.
async fn complete_action(client: &RESTClient, action_id: i64, champion_id: i64) -> anyhow::Result<()> {
    client
        .patch(
            format!("/lol-champ-select/v1/session/actions/{action_id}"),
            json!({ "championId": champion_id, "completed": true }),
        )
        .await?;
    Ok(())
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DraftSession {
    #[serde(default = "no_cell")]
    local_player_cell_id: i64,
    #[serde(default)]
    actions: Vec<Vec<DraftAction>>,
    #[serde(default)]
    timer: DraftTimer,
    #[serde(default)]
    my_team: Vec<DraftMember>,
}

fn no_cell() -> i64 {
    -1
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DraftAction {
    id: i64,
    #[serde(default = "no_cell")]
    actor_cell_id: i64,
    #[serde(default)]
    champion_id: i64,
    #[serde(default)]
    completed: bool,
    #[serde(default)]
    is_in_progress: bool,
    /// "ban", "pick", or others like "ten_bans_reveal".
    #[serde(rename = "type", default)]
    kind: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DraftTimer {
    #[serde(default)]
    adjusted_time_left_in_phase: f64,
    /// When League worked out the time left, in its epoch milliseconds.
    #[serde(default)]
    internal_now_in_epoch_ms: f64,
    #[serde(default)]
    phase: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DraftMember {
    cell_id: i64,
    #[serde(default)]
    assigned_position: String,
    #[serde(default)]
    champion_pick_intent: i64,
}

/// Your ban or pick that's under way and not done yet.
fn current_turn(session: &DraftSession) -> Option<&DraftAction> {
    if session.timer.phase != "BAN_PICK" {
        return None;
    }
    session.actions.iter().flatten().find(|action| {
        action.actor_cell_id == session.local_player_cell_id
            && action.is_in_progress
            && !action.completed
            && matches!(action.kind.as_str(), "ban" | "pick")
    })
}

/// The time left in the current turn, in milliseconds.
fn time_left_ms(session: &DraftSession, now_ms: f64) -> f64 {
    let timer = &session.timer;
    // The session only changes on events, so count the time since League measured it.
    let since = if timer.internal_now_in_epoch_ms > 0.0 {
        (now_ms - timer.internal_now_in_epoch_ms).max(0.0)
    } else {
        0.0
    };
    timer.adjusted_time_left_in_phase - since
}

/// Your unfinished ban or pick, once only the time you set is left of it.
fn due_turn<'a>(session: &'a DraftSession, now_ms: f64, settings: &Config) -> Option<&'a DraftAction> {
    let turn = current_turn(session)?;
    (time_left_ms(session, now_ms) <= act_at_ms(settings, &turn.kind)).then_some(turn)
}

/// What to lock in: what you already chose, or else the first available of your list. Champions
/// already banned or picked, and teammates' picks and hovers, are never chosen from the list.
fn choose_champion(
    session: &DraftSession,
    turn: &DraftAction,
    settings: &Config,
    available: Option<&HashSet<i64>>,
) -> Option<i64> {
    let me = session
        .my_team
        .iter()
        .find(|member| member.cell_id == session.local_player_cell_id);
    let (enabled, chosen, list) = if turn.kind == "ban" {
        (settings.auto_ban, turn.champion_id, settings.ban_champions.as_slice())
    } else {
        let hovered = [turn.champion_id, me.map_or(0, |me| me.champion_pick_intent)]
            .into_iter()
            .find(|id| *id > 0)
            .unwrap_or_default();
        let role = me.map_or("", |me| me.assigned_position.as_str());
        let list = settings
            .pick_champions
            .get(role)
            .map(Vec::as_slice)
            .unwrap_or_default();
        (settings.auto_pick, hovered, list)
    };
    if !enabled {
        return None;
    }

    let locked: HashSet<i64> = session
        .actions
        .iter()
        .flatten()
        .filter(|action| action.completed && action.champion_id > 0)
        .map(|action| action.champion_id)
        .collect();
    let teammates_want: HashSet<i64> = session
        .my_team
        .iter()
        .filter(|member| member.cell_id != session.local_player_cell_id)
        .map(|member| member.champion_pick_intent)
        .filter(|id| *id > 0)
        .collect();
    let free = |id: i64| !locked.contains(&id) && available.map_or(true, |ids| ids.contains(&id));
    // Your own pick hover stands even if a teammate hovers the same champion: whoever locks
    // first gets it. Banning a teammate's hover, or picking it off your list, never happens.
    let hover_ok = |id: i64| free(id) && (turn.kind != "ban" || !teammates_want.contains(&id));

    if chosen > 0 && hover_ok(chosen) {
        return Some(chosen);
    }
    list.iter()
        .copied()
        .find(|id| free(*id) && !teammates_want.contains(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const NOW: f64 = 1_000_000.0;

    fn action(id: i64, actor: i64, kind: &str, champion_id: i64) -> DraftAction {
        DraftAction {
            id,
            actor_cell_id: actor,
            champion_id,
            completed: false,
            is_in_progress: true,
            kind: kind.to_string(),
        }
    }

    fn session(time_left_ms: f64, actions: Vec<DraftAction>) -> DraftSession {
        DraftSession {
            local_player_cell_id: 1,
            actions: vec![actions],
            timer: DraftTimer {
                adjusted_time_left_in_phase: time_left_ms,
                internal_now_in_epoch_ms: NOW,
                phase: "BAN_PICK".to_string(),
            },
            my_team: vec![
                DraftMember {
                    cell_id: 1,
                    assigned_position: "jungle".to_string(),
                    champion_pick_intent: 0,
                },
                DraftMember {
                    cell_id: 2,
                    assigned_position: "middle".to_string(),
                    champion_pick_intent: 103,
                },
            ],
        }
    }

    fn settings() -> Config {
        Config {
            auto_ban: true,
            ban_champions: vec![103, 64, 421],
            auto_pick: true,
            pick_champions: HashMap::from([("jungle".to_string(), vec![59, 104])]),
            ..Config::default()
        }
    }

    #[test]
    fn waits_for_the_last_seconds_of_your_own_turn() {
        let early = session(10_000.0, vec![action(7, 1, "pick", 0)]);
        assert!(due_turn(&early, NOW, &Config::default()).is_none());
        assert!(due_turn(&early, NOW + 8_000.0, &Config::default()).is_none(), "2 seconds left is still early");
        // The session was measured 9 seconds ago, so only 1 is left now.
        assert_eq!(due_turn(&early, NOW + 9_000.0, &Config::default()).map(|turn| turn.id), Some(7));

        let teammate = session(1_000.0, vec![action(8, 2, "pick", 0)]);
        assert!(due_turn(&teammate, NOW, &Config::default()).is_none());

        let mut done = action(9, 1, "ban", 64);
        done.completed = true;
        assert!(due_turn(&session(1_000.0, vec![done]), NOW, &Config::default()).is_none());

        let mut planning = session(1_000.0, vec![action(10, 1, "pick", 0)]);
        planning.timer.phase = "PLANNING".to_string();
        assert!(due_turn(&planning, NOW, &Config::default()).is_none());
    }

    #[test]
    fn locks_in_your_own_hover_before_anything_from_the_list() {
        let hovered = session(1_000.0, vec![action(7, 1, "pick", 238)]);
        assert_eq!(
            choose_champion(&hovered, &hovered.actions[0][0], &settings(), None),
            Some(238)
        );

        let mut declared = session(1_000.0, vec![action(7, 1, "pick", 0)]);
        declared.my_team[0].champion_pick_intent = 60;
        assert_eq!(
            choose_champion(&declared, &declared.actions[0][0], &settings(), None),
            Some(60)
        );
    }

    #[test]
    fn locks_in_your_hover_even_when_a_teammate_hovers_it_too() {
        // Your mid laner hovers Ahri (103), and so do you.
        let draft = session(1_000.0, vec![action(7, 1, "pick", 103)]);
        assert_eq!(
            choose_champion(&draft, &draft.actions[0][0], &settings(), None),
            Some(103)
        );
    }

    #[test]
    fn picks_the_first_available_champion_for_your_role() {
        let mut banned = action(3, 6, "ban", 59);
        banned.completed = true;
        banned.is_in_progress = false;
        let draft = session(1_000.0, vec![banned, action(7, 1, "pick", 0)]);
        let turn = &draft.actions[0][1];

        assert_eq!(choose_champion(&draft, turn, &settings(), None), Some(104));
        let unowned = HashSet::from([59, 60]);
        assert_eq!(
            choose_champion(&draft, turn, &settings(), Some(&unowned)),
            None,
            "nothing from the list is owned and free"
        );
        let off = Config {
            auto_pick: false,
            ..settings()
        };
        assert_eq!(choose_champion(&draft, turn, &off, None), None);
    }

    #[test]
    fn never_bans_a_teammates_champion() {
        // Your mid laner is hovering Ahri (103), the first ban on the list.
        let draft = session(1_000.0, vec![action(4, 1, "ban", 0)]);
        assert_eq!(
            choose_champion(&draft, &draft.actions[0][0], &settings(), None),
            Some(64)
        );
        let bannable = HashSet::from([421]);
        assert_eq!(
            choose_champion(&draft, &draft.actions[0][0], &settings(), Some(&bannable)),
            Some(421)
        );
    }

    #[test]
    fn retries_a_failed_ban_or_pick_a_few_times() {
        let mut attempts = TurnAttempts::default();
        assert!(!attempts.is_finished(7));
        assert!(attempts.failed(7), "first failure: try again");
        assert!(attempts.failed(7), "second failure: try again");
        assert!(!attempts.failed(7), "third failure: give up");
        assert!(attempts.is_finished(7));
        assert!(!attempts.is_finished(8), "other turns are separate");

        assert!(attempts.give_up(8));
        assert!(!attempts.give_up(8), "already given up");
    }

    #[test]
    fn announces_your_turn_as_soon_as_it_starts() {
        let draft = session(27_400.0, vec![action(7, 1, "pick", 0)]);
        let turn = current_turn(&draft).expect("your pick is under way");
        assert!(due_turn(&draft, NOW, &Config::default()).is_none(), "too early to lock in");

        let manual = Config::default();
        let notice = turn_notice(&draft, turn, &manual, NOW + 400.0);
        assert_eq!(notice.title, "Your turn to pick");
        assert_eq!(notice.body, "27 seconds left.");
        let notice = turn_notice(&draft, turn, &settings(), NOW);
        assert!(notice.body.ends_with("in the last second if you don't."));
        let early = Config {
            auto_pick_seconds_left: 5,
            ..settings()
        };
        let notice = turn_notice(&draft, turn, &early, NOW);
        assert!(notice.body.ends_with("with 5 seconds left if you don't."));
    }

    #[test]
    fn acts_as_early_as_set_for_bans_and_picks_separately() {
        let early = Config {
            auto_ban_seconds_left: 10,
            auto_pick_seconds_left: 3,
            ..Config::default()
        };
        let ban = session(9_500.0, vec![action(4, 1, "ban", 0)]);
        assert!(due_turn(&ban, NOW, &Config::default()).is_none(), "last second by default");
        assert_eq!(due_turn(&ban, NOW, &early).map(|turn| turn.id), Some(4));

        let pick = session(4_000.0, vec![action(7, 1, "pick", 0)]);
        assert!(due_turn(&pick, NOW, &early).is_none(), "4 seconds left, set to 3");
        assert_eq!(due_turn(&pick, NOW + 1_000.0, &early).map(|turn| turn.id), Some(7));

        let zero = Config {
            auto_pick_seconds_left: 0,
            ..Config::default()
        };
        assert_eq!(act_at_ms(&zero, "pick"), LAST_SECONDS_MS, "never later than the last second");
    }

    #[test]
    fn restarts_only_queues_past_a_real_estimate() {
        assert!(!past_estimate(95.0, 94.0), "just past the estimate is normal");
        assert!(!past_estimate(150.0, 94.0), "under a minute past it");
        assert!(past_estimate(155.0, 94.0));
        assert!(!past_estimate(400.0, 300.0), "under half again as long");
        assert!(past_estimate(451.0, 300.0));
        assert!(!past_estimate(60.0, 0.0), "no estimate yet");
    }

    #[test]
    fn never_leaves_a_queue_that_found_a_match() {
        assert!(still_searching(&json!({ "searchState": "Searching", "readyCheck": { "state": "Invalid" } })));
        assert!(!still_searching(&json!({ "searchState": "Found", "readyCheck": { "state": "Invalid" } })));
        assert!(!still_searching(&json!({ "searchState": "Searching", "readyCheck": { "state": "InProgress" } })));
        assert!(!still_searching(&json!({ "searchState": "Searching", "readyCheck": { "state": "EveryoneReady" } })));
        assert!(!still_searching(&json!({})));
    }

    #[test]
    fn reads_the_session_league_sends() {
        let session: DraftSession = serde_json::from_value(json!({
            "localPlayerCellId": 2,
            "actions": [[{ "id": 1, "actorCellId": 2, "championId": 0, "completed": false,
                           "isInProgress": true, "type": "ban", "pickTurn": 1 }]],
            "timer": { "adjustedTimeLeftInPhase": 1000, "internalNowInEpochMs": 1000,
                       "phase": "BAN_PICK", "isInfinite": false },
            "myTeam": [{ "cellId": 2, "assignedPosition": "jungle", "championPickIntent": 0 }],
            "theirTeam": []
        }))
        .unwrap();
        assert_eq!(due_turn(&session, 1_000.0, &Config::default()).map(|turn| turn.id), Some(1));
    }
}
