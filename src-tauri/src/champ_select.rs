use crate::{
    analytics, champions, config::Config, lobby, ranked, region::RegionInfo, roles, summoner,
    utils::display_champ_select,
};
use serde::Deserialize;
use shaco::rest::RESTClient;
use std::{
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};

const EXPECTED_PARTICIPANT_COUNT: usize = 5;
const PARTICIPANT_POLL_INTERVAL: Duration = Duration::from_secs(2);
const MAX_LOOKUP_ATTEMPTS: u8 = 3;

static PARTICIPANT_POLLING: AtomicBool = AtomicBool::new(false);

/// Held while Champ Select participants are being polled. The gameflow phase can be reported
/// more than once (initial read plus websocket), and a second loop would open a second tab.
pub struct PollingGuard(());

impl PollingGuard {
    pub fn acquire() -> Option<Self> {
        PARTICIPANT_POLLING
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| Self(()))
    }
}

impl Drop for PollingGuard {
    fn drop(&mut self) {
        PARTICIPANT_POLLING.store(false, Ordering::Release);
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectSession {
    pub game_id: u64,
    #[serde(default = "no_cell")]
    pub local_player_cell_id: i64,
    pub timer: Timer,
    #[serde(default)]
    pub my_team: Vec<ChampSelectTeamMember>,
    #[serde(default)]
    pub their_team: Vec<ChampSelectTeamMember>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectTeamMember {
    #[serde(default)]
    pub assigned_position: String,
    pub cell_id: u32,
    /// Empty when Riot hides the player (ranked).
    #[serde(default)]
    pub puuid: String,
    #[serde(default)]
    pub spell1_id: i64,
    #[serde(default)]
    pub spell2_id: i64,
    #[serde(default)]
    pub champion_id: i64,
    #[serde(default)]
    pub champion_pick_intent: i64,
    #[serde(default)]
    pub is_autofilled: bool,
    #[serde(default)]
    pub game_name: String,
    #[serde(default)]
    pub tag_line: String,
}

impl ChampSelectTeamMember {
    /// The locked champion, or the hovered one before lock-in.
    fn champion(&self) -> Option<i64> {
        [self.champion_id, self.champion_pick_intent]
            .into_iter()
            .find(|id| *id > 0)
    }
}

fn no_cell() -> i64 {
    -1
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub adjusted_time_left_in_phase: u64,
    pub phase: String,
}

pub async fn handle_champ_select_start(
    app_client: &RESTClient,
    remoting_client: &RESTClient,
    config: &Config,
    app_handle: &AppHandle,
) {
    log_info!(
        "Champ Select participant polling started; auto_open={}, max_wait_seconds={}, provider={}",
        config.auto_open,
        config.auto_open_delay_seconds,
        config.multi_provider
    );
    let region_info: RegionInfo = match app_client
        .get("/riotclient/region-locale".to_string())
        .await
        .map_err(anyhow::Error::from)
        .and_then(|value| serde_json::from_value(value).map_err(anyhow::Error::from))
    {
        Ok(region_info) => region_info,
        Err(error) => {
            log_error!("Failed to read League region: {error}");
            return;
        }
    };

    let region = match region_info.web_region.as_str() {
        "SG2" => "SG",
        _ => &region_info.web_region,
    };
    crate::live_game::remember_region(region);

    let mut auto_opened = false;
    let mut last_participant_count = 0;
    let mut last_emitted_team: Option<lobby::Lobby> = None;
    let mut first_participant_seen_at: Option<Instant> = None;
    let mut session_warning_logged = false;
    let mut players = PlayerCache::default();
    let queue_name = crate::queue::current_queue_name(remoting_client).await;
    let _ = app_handle.emit_all("queue_name", &queue_name);
    let mut analytics_sent = false;

    // Poll until champ select ends: slots, spells and picks keep changing, which updates
    // the guessed roles for hidden players.
    loop {
        // Champ select sanity check
        let gameflow_state = remoting_client
            .get("/lol-gameflow/v1/gameflow-phase".to_string())
            .await;

        if let Ok(state) = gameflow_state {
            let state_str = state.to_string().replace('\"', "");
            if state_str != "ChampSelect" {
                log_info!("Champ Select ended; stopping participant polling");
                break;
            }
        } else {
            // Invalid gameflow state - assume we're not in champ select
            log_warn!(
                "Could not confirm the current gameflow state; stopping Champ Select polling"
            );
            break;
        }

        // This will return an error if we aren't connected to chat
        let mut team = match lobby::get_lobby_info(app_client).await {
            Ok(team) => team,
            Err(error) => {
                log_warn!("Lobby participants are not available yet; retrying: {error}");
                tokio::time::sleep(PARTICIPANT_POLL_INTERVAL).await;
                continue;
            }
        };
        // Chat reports its own server names ("eu1" for EUW), so use the client's region instead.
        for participant in &mut team.participants {
            participant.region = region.to_string();
        }

        let session = match get_champ_select_session(remoting_client).await {
            Ok(session) => {
                session_warning_logged = false;
                apply_champ_select_context(&mut team, &session.my_team);
                mark_local_player(&mut team, &session.my_team, session.local_player_cell_id);
                add_visible_enemies(&mut team, &session.their_team);
                mark_enemies(&mut team, &session.my_team, &session.their_team);
                Some(session)
            }
            Err(error) => {
                if !session_warning_logged {
                    log_warn!(
                        "Champ Select slot information is not available yet; names will still be revealed: {error}"
                    );
                    session_warning_logged = true;
                }
                None
            }
        };
        players.fetch_missing(remoting_client, &team).await;
        players.apply(&mut team);
        if let Some(session) = &session {
            guess_hidden_positions(&mut team, &session.my_team, &players);
        }
        players.apply_role_fit(&mut team);
        // Auto-open, counts and analytics are about your own team.
        let allies = lobby::Lobby {
            participants: team
                .participants
                .iter()
                .filter(|p| !p.is_enemy)
                .cloned()
                .collect(),
        };
        let participant_count = allies.participants.len();

        if participant_count > 0 && first_participant_seen_at.is_none() {
            first_participant_seen_at = Some(Instant::now());
        }

        if last_emitted_team.as_ref() != Some(&team) {
            if participant_count > last_participant_count {
                log_info!(
                    "Champ Select participant count increased to {} from {}",
                    participant_count,
                    last_participant_count
                );
                last_participant_count = participant_count;
            }

            crate::live_game::remember_champ_select(&team);
            if let Err(error) = app_handle.emit_all("champ_select_started", &team) {
                log_error!("Failed to emit Champ Select participants: {error}");
            }
            last_emitted_team = Some(team.clone());
        }

        let auto_open_elapsed = first_participant_seen_at
            .map(|started_at| started_at.elapsed())
            .unwrap_or_default();
        let auto_open_max_wait = Duration::from_secs(config.auto_open_delay_seconds.into());

        // Open as soon as the team is complete, or fall back to the latest
        // participant list once the configured maximum wait has elapsed.
        if should_auto_open(
            config.auto_open,
            auto_opened,
            participant_count,
            auto_open_elapsed,
            auto_open_max_wait,
        ) {
            if let Err(error) = display_champ_select(&allies, region, &config.multi_provider) {
                log_error!("Automatic multi-search open failed: {error}");
            } else {
                log_info!(
                    "Automatically opened {} multi-search with {} participants after {:?}",
                    config.multi_provider,
                    participant_count,
                    auto_open_elapsed
                );
            }
            auto_opened = true;
        }

        // Send analytics once, when all 5 teammates and their details are in.
        if !analytics_sent
            && participant_count >= EXPECTED_PARTICIPANT_COUNT
            && players.is_settled(&team)
        {
            analytics::send_analytics_event(
                participant_count,
                &region_info.web_region,
                &config.multi_provider,
                &app_handle.package_info().version.to_string(),
            )
            .await;
            analytics_sent = true;
            log_info!("Complete Champ Select team found; still watching for role changes");
        }

        tokio::time::sleep(PARTICIPANT_POLL_INTERVAL).await;
    }
}

/// A per-player lookup that is tried up to `MAX_LOOKUP_ATTEMPTS` times.
struct Lookup<T> {
    values: HashMap<String, T>,
    attempts: HashMap<String, u8>,
}

impl<T> Default for Lookup<T> {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
            attempts: HashMap::new(),
        }
    }
}

impl<T: Clone> Lookup<T> {
    /// Counts an attempt and returns true if this player still needs a lookup.
    fn start_attempt(&mut self, puuid: &str) -> bool {
        if puuid.is_empty() || self.values.contains_key(puuid) {
            return false;
        }
        let attempts = self.attempts.entry(puuid.to_string()).or_default();
        if *attempts >= MAX_LOOKUP_ATTEMPTS {
            return false;
        }
        *attempts += 1;
        true
    }

    fn record(&mut self, what: &str, puuid: &str, result: anyhow::Result<T>) {
        match result {
            Ok(value) => {
                self.values.insert(puuid.to_string(), value);
            }
            Err(error) => log_warn!(
                "{what} lookup failed (attempt {}/{MAX_LOOKUP_ATTEMPTS}): {error}",
                self.attempts.get(puuid).copied().unwrap_or(0)
            ),
        }
    }

    fn get(&self, puuid: &str) -> Option<T> {
        self.values.get(puuid).cloned()
    }

    fn is_settled(&self, puuid: &str) -> bool {
        puuid.is_empty()
            || self.values.contains_key(puuid)
            || self.attempts.get(puuid).copied().unwrap_or(0) >= MAX_LOOKUP_ATTEMPTS
    }
}

/// Ranks and champion mains, looked up once per player for the current Champ Select.
#[derive(Default)]
struct PlayerCache {
    ranks: Lookup<ranked::RankSummary>,
    mains: Lookup<Vec<champions::ChampionMain>>,
    profile_icons: Lookup<i32>,
    histories: Lookup<roles::PlayerHistory>,
    champion_names: Option<HashMap<i32, String>>,
}

impl PlayerCache {
    async fn fetch_missing(&mut self, remoting_client: &RESTClient, team: &lobby::Lobby) {
        for participant in &team.participants {
            let puuid = participant.puuid.as_str();

            if self.ranks.start_attempt(puuid) {
                let result = ranked::get_solo_rank(remoting_client, puuid).await;
                self.ranks.record("Rank", puuid, result);
            }

            if self.histories.start_attempt(puuid) {
                let result = roles::get_player_history(remoting_client, puuid).await;
                self.histories.record("Match history", puuid, result);
            }

            if self.profile_icons.start_attempt(puuid) {
                let result = summoner::get_profile_icon_id(remoting_client, puuid).await;
                self.profile_icons.record("Profile icon", puuid, result);
            }

            if self.mains.start_attempt(puuid) {
                if self.champion_names.is_none() {
                    match champions::get_champion_names(remoting_client).await {
                        Ok(names) => self.champion_names = Some(names),
                        Err(error) => log_warn!("Champion name lookup failed: {error}"),
                    }
                }
                let empty = HashMap::new();
                let names = self.champion_names.as_ref().unwrap_or(&empty);
                let result = champions::get_top_mains(remoting_client, puuid, names).await;
                self.mains.record("Champion mastery", puuid, result);
            }
        }
    }

    fn apply(&self, team: &mut lobby::Lobby) {
        for participant in &mut team.participants {
            participant.rank = self.ranks.get(&participant.puuid);
            participant.mains = self.mains.get(&participant.puuid);
            participant.profile_icon_id = self.profile_icons.get(&participant.puuid);
            participant.recent = self
                .histories
                .values
                .get(&participant.puuid)
                .map(|history| history.recent.clone());
        }
    }

    /// Compares each placed player's history with their assigned role.
    fn apply_role_fit(&self, team: &mut lobby::Lobby) {
        for participant in &mut team.participants {
            participant.role_fit = participant
                .assigned_position
                .as_deref()
                .and_then(|position| {
                    self.histories
                        .values
                        .get(&participant.puuid)
                        .and_then(|history| history.role_fit(position))
                });
        }
    }

    /// True once every player has their details or has used up their lookup attempts.
    fn is_settled(&self, team: &lobby::Lobby) -> bool {
        team.participants.iter().all(|participant| {
            self.ranks.is_settled(&participant.puuid)
                && self.mains.is_settled(&participant.puuid)
                && self.profile_icons.is_settled(&participant.puuid)
                && self.histories.is_settled(&participant.puuid)
        })
    }
}

async fn get_champ_select_session(
    remoting_client: &RESTClient,
) -> anyhow::Result<ChampSelectSession> {
    let session = remoting_client
        .get("/lol-champ-select/v1/session".to_string())
        .await?;
    Ok(serde_json::from_value(session)?)
}

/// A slot's place in the team's pick order. League reports `pickTurn` as 0 for every slot in
/// ranked, but slots are numbered in pick order, which is also the order the client shows.
fn pick_order(champ_select_team: &[ChampSelectTeamMember], cell_id: u32) -> u32 {
    champ_select_team
        .iter()
        .filter(|member| member.cell_id < cell_id)
        .count() as u32
}

fn apply_champ_select_context(
    lobby: &mut lobby::Lobby,
    champ_select_team: &[ChampSelectTeamMember],
) -> usize {
    let slots_by_puuid: HashMap<&str, &ChampSelectTeamMember> = champ_select_team
        .iter()
        .map(|member| (member.puuid.as_str(), member))
        .collect();
    let mut mapped_count = 0;

    for participant in &mut lobby.participants {
        let Some(member) = slots_by_puuid.get(participant.puuid.as_str()) else {
            continue;
        };

        participant.assigned_position =
            (!member.assigned_position.is_empty()).then(|| member.assigned_position.clone());
        participant.cell_id = Some(member.cell_id);
        participant.pick_turn = Some(pick_order(champ_select_team, member.cell_id));
        participant.autofilled = member.is_autofilled;
        participant.champion_id = member.champion();
        mapped_count += 1;
    }

    lobby.participants.sort_by_key(|participant| {
        (
            participant.pick_turn.unwrap_or(u32::MAX),
            participant.cell_id.unwrap_or(u32::MAX),
        )
    });

    mapped_count
}

/// Adds enemies whose Riot IDs the session shows (e.g. 5-stack queues, where chat only has
/// your own team but the other team's slots include their names).
fn add_visible_enemies(lobby: &mut lobby::Lobby, their_team: &[ChampSelectTeamMember]) {
    let region = lobby
        .participants
        .first()
        .map(|p| p.region.clone())
        .unwrap_or_default();
    for member in their_team {
        if member.puuid.is_empty()
            || member.game_name.is_empty()
            || lobby.participants.iter().any(|p| p.puuid == member.puuid)
        {
            continue;
        }
        lobby.participants.push(lobby::Participant {
            cid: "champ-select-enemy".to_string(),
            game_name: member.game_name.clone(),
            game_tag: member.tag_line.clone(),
            name: member.game_name.clone(),
            pid: member.puuid.clone(),
            puuid: member.puuid.clone(),
            region: region.clone(),
            ..Default::default()
        });
    }
}

/// Marks players on the other team, for lobbies where Riot shows both teams (ranked 5-stacks).
/// Teams are told apart by puuid when the slots include them; otherwise the lobby is split in
/// half, with your half being the one you're in. Enemies matched to a slot get its role and
/// their pick order within their team.
fn mark_enemies(
    lobby: &mut lobby::Lobby,
    my_team: &[ChampSelectTeamMember],
    their_team: &[ChampSelectTeamMember],
) {
    for participant in &mut lobby.participants {
        participant.is_enemy = false;
    }
    let team_size = my_team.len().max(EXPECTED_PARTICIPANT_COUNT);
    if lobby.participants.len() <= team_size {
        return;
    }

    let allies: HashSet<&str> = my_team
        .iter()
        .map(|member| member.puuid.as_str())
        .filter(|puuid| !puuid.is_empty())
        .collect();
    let enemy_slots: HashMap<&str, &ChampSelectTeamMember> = their_team
        .iter()
        .filter(|member| !member.puuid.is_empty())
        .map(|member| (member.puuid.as_str(), member))
        .collect();

    if allies.is_empty() && enemy_slots.is_empty() {
        // No puuids to go on: the first and second halves of the lobby are the two teams.
        let half = lobby.participants.len() / 2;
        let self_in_second_half = lobby
            .participants
            .iter()
            .position(|participant| participant.is_self)
            .is_some_and(|index| index >= half);
        for (index, participant) in lobby.participants.iter_mut().enumerate() {
            participant.is_enemy = (index >= half) != self_in_second_half;
        }
        return;
    }

    let mut ally_count = 0;
    for participant in &mut lobby.participants {
        if allies.contains(participant.puuid.as_str()) {
            ally_count += 1;
        } else if let Some(slot) = enemy_slots.get(participant.puuid.as_str()) {
            participant.is_enemy = true;
            participant.cell_id = Some(slot.cell_id);
            participant.pick_turn = Some(pick_order(their_team, slot.cell_id));
            participant.assigned_position =
                (!slot.assigned_position.is_empty()).then(|| slot.assigned_position.clone());
            participant.champion_id = slot.champion();
        }
    }
    // Players neither list identifies: fill your team up first, the rest are enemies.
    for participant in &mut lobby.participants {
        let known = allies.contains(participant.puuid.as_str()) || participant.is_enemy;
        if !known {
            if ally_count < my_team.len() {
                ally_count += 1;
            } else {
                participant.is_enemy = true;
            }
        }
    }
}

/// Marks the user's own entry; the local player's slot is never hidden from them.
fn mark_local_player(
    lobby: &mut lobby::Lobby,
    champ_select_team: &[ChampSelectTeamMember],
    local_cell_id: i64,
) {
    let local_puuid = champ_select_team
        .iter()
        .find(|member| i64::from(member.cell_id) == local_cell_id)
        .map(|member| member.puuid.as_str())
        .filter(|puuid| !puuid.is_empty());
    for participant in &mut lobby.participants {
        participant.is_self = Some(participant.puuid.as_str()) == local_puuid;
    }
}

/// Places teammates whose slots Riot hid (ranked) by matching their match history against
/// each hidden slot's position, summoner spells and champion. Marks those as guessed.
fn guess_hidden_positions(
    lobby: &mut lobby::Lobby,
    champ_select_team: &[ChampSelectTeamMember],
    players: &PlayerCache,
) {
    let slots: Vec<roles::HiddenSlot> = champ_select_team
        .iter()
        .filter(|member| member.puuid.is_empty() && !member.assigned_position.is_empty())
        .map(|member| roles::HiddenSlot {
            cell_id: member.cell_id,
            position: member.assigned_position.clone(),
            spells: (member.spell1_id, member.spell2_id),
            champion_id: if member.champion_id > 0 {
                member.champion_id
            } else {
                member.champion_pick_intent
            },
            autofilled: member.is_autofilled,
        })
        .collect();
    if slots.is_empty() {
        return;
    }

    let unplaced: Vec<usize> = (0..lobby.participants.len())
        .filter(|&index| {
            let participant = &lobby.participants[index];
            participant.cell_id.is_none() && !participant.is_enemy
        })
        .collect();
    let no_mains = Vec::new();
    let candidates: Vec<roles::Candidate> = unplaced
        .iter()
        .map(|&index| {
            let puuid = &lobby.participants[index].puuid;
            roles::Candidate {
                history: players.histories.values.get(puuid),
                mains: players.mains.values.get(puuid).unwrap_or(&no_mains),
            }
        })
        .collect();

    for (candidate, slot_index) in roles::assign(&candidates, &slots).into_iter().enumerate() {
        let Some(slot_index) = slot_index else {
            continue;
        };
        let slot = &slots[slot_index];
        let member = champ_select_team
            .iter()
            .find(|member| member.cell_id == slot.cell_id)
            .expect("slot comes from the team");
        let participant = &mut lobby.participants[unplaced[candidate]];
        participant.assigned_position = Some(slot.position.clone());
        participant.cell_id = Some(slot.cell_id);
        participant.pick_turn = Some(pick_order(champ_select_team, member.cell_id));
        participant.position_guessed = true;
        participant.autofilled = member.is_autofilled;
        participant.champion_id = member.champion();
    }

    lobby.participants.sort_by_key(|participant| {
        (
            participant.pick_turn.unwrap_or(u32::MAX),
            participant.cell_id.unwrap_or(u32::MAX),
        )
    });
}

fn should_auto_open(
    enabled: bool,
    already_opened: bool,
    participant_count: usize,
    elapsed: Duration,
    max_wait: Duration,
) -> bool {
    enabled
        && !already_opened
        && participant_count > 0
        && (participant_count >= EXPECTED_PARTICIPANT_COUNT || elapsed >= max_wait)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_WAIT: Duration = Duration::from_secs(6);

    #[test]
    fn only_one_participant_poll_runs_at_a_time() {
        let guard = PollingGuard::acquire().expect("first poll should start");
        assert!(PollingGuard::acquire().is_none());
        drop(guard);
        let guard = PollingGuard::acquire().expect("poll should restart after the first ends");
        drop(guard);
    }

    #[test]
    fn opens_immediately_for_a_complete_team() {
        assert!(should_auto_open(
            true,
            false,
            EXPECTED_PARTICIPANT_COUNT,
            Duration::ZERO,
            MAX_WAIT,
        ));
    }

    #[test]
    fn waits_for_an_incomplete_team_until_the_timeout() {
        assert!(!should_auto_open(
            true,
            false,
            3,
            Duration::from_secs(4),
            MAX_WAIT,
        ));
        assert!(should_auto_open(true, false, 3, MAX_WAIT, MAX_WAIT,));
    }

    #[test]
    fn does_not_open_without_participants_or_when_disabled() {
        assert!(!should_auto_open(true, false, 0, MAX_WAIT, MAX_WAIT));
        assert!(!should_auto_open(
            false,
            false,
            EXPECTED_PARTICIPANT_COUNT,
            MAX_WAIT,
            MAX_WAIT,
        ));
        assert!(!should_auto_open(
            true,
            true,
            EXPECTED_PARTICIPANT_COUNT,
            MAX_WAIT,
            MAX_WAIT,
        ));
    }

    fn participant(puuid: &str, game_name: &str) -> lobby::Participant {
        lobby::Participant {
            cid: "champ-select".to_string(),
            game_name: game_name.to_string(),
            game_tag: "NA1".to_string(),
            muted: false,
            name: game_name.to_string(),
            pid: puuid.to_string(),
            puuid: puuid.to_string(),
            region: "NA".to_string(),
            assigned_position: None,
            cell_id: None,
            pick_turn: None,
            rank: None,
            mains: None,
            profile_icon_id: None,
            recent: None,
            position_guessed: false,
            autofilled: false,
            role_fit: None,
            is_self: false,
            champion_id: None,
            is_enemy: false,
        }
    }

    fn member_defaults() -> ChampSelectTeamMember {
        ChampSelectTeamMember {
            assigned_position: String::new(),
            cell_id: 0,
            puuid: String::new(),
            spell1_id: 0,
            spell2_id: 0,
            champion_id: 0,
            champion_pick_intent: 0,
            is_autofilled: false,
            game_name: String::new(),
            tag_line: String::new(),
        }
    }

    #[test]
    fn numbers_picks_by_slot_even_when_pick_turn_is_zero() {
        let mut team = lobby::Lobby {
            participants: vec![
                participant("c", "C"),
                participant("a", "A"),
                participant("b", "B"),
            ],
        };
        let slot = |cell_id, puuid: &str| ChampSelectTeamMember {
            assigned_position: "top".to_string(),
            cell_id,
            puuid: puuid.to_string(),
            ..member_defaults()
        };
        let slots = vec![slot(5, "a"), slot(6, "b"), slot(9, "c")];

        apply_champ_select_context(&mut team, &slots);

        let order: Vec<(&str, Option<u32>)> = team
            .participants
            .iter()
            .map(|participant| (participant.puuid.as_str(), participant.pick_turn))
            .collect();
        assert_eq!(order, vec![("a", Some(0)), ("b", Some(1)), ("c", Some(2))]);
    }

    fn five_stack_lobby() -> lobby::Lobby {
        lobby::Lobby {
            participants: (0..10)
                .map(|i| participant(&format!("p{i}"), &format!("P{i}")))
                .collect(),
        }
    }

    #[test]
    fn splits_a_five_stack_lobby_by_puuid_and_orders_enemies_by_their_slots() {
        let mut team = five_stack_lobby();
        let slot = |cell_id, puuid: &str, position: &str| ChampSelectTeamMember {
            cell_id,
            puuid: puuid.to_string(),
            assigned_position: position.to_string(),
            ..member_defaults()
        };
        let mine: Vec<_> = (0..5).map(|i| slot(i, &format!("p{i}"), "")).collect();
        let theirs = vec![slot(9, "p5", "top"), slot(5, "p6", "jungle")];

        mark_enemies(&mut team, &mine, &theirs);

        let enemies: Vec<&str> = team
            .participants
            .iter()
            .filter(|p| p.is_enemy)
            .map(|p| p.puuid.as_str())
            .collect();
        assert_eq!(enemies, vec!["p5", "p6", "p7", "p8", "p9"]);
        let p6 = team.participants.iter().find(|p| p.puuid == "p6").unwrap();
        assert_eq!(
            (p6.pick_turn, p6.assigned_position.as_deref()),
            (Some(0), Some("jungle"))
        );
        let p5 = team.participants.iter().find(|p| p.puuid == "p5").unwrap();
        assert_eq!(p5.pick_turn, Some(1));
    }

    #[test]
    fn adds_enemies_the_session_names_when_chat_only_has_your_team() {
        let mut team = lobby::Lobby {
            participants: (0..5).map(|i| participant(&format!("p{i}"), "P")).collect(),
        };
        let mine: Vec<_> = (0..5)
            .map(|i| ChampSelectTeamMember {
                cell_id: 5 + i,
                puuid: format!("p{i}"),
                ..member_defaults()
            })
            .collect();
        let theirs: Vec<_> = (0..5)
            .map(|i| ChampSelectTeamMember {
                cell_id: 4 - i,
                puuid: format!("e{i}"),
                game_name: format!("Enemy{i}"),
                tag_line: "NA1".into(),
                ..member_defaults()
            })
            .collect();

        add_visible_enemies(&mut team, &theirs);
        add_visible_enemies(&mut team, &theirs);
        mark_enemies(&mut team, &mine, &theirs);

        assert_eq!(team.participants.len(), 10, "added once");
        let enemies: Vec<_> = team.participants.iter().filter(|p| p.is_enemy).collect();
        assert_eq!(enemies.len(), 5);
        assert_eq!(
            (enemies[0].game_name.as_str(), enemies[0].game_tag.as_str()),
            ("Enemy0", "NA1")
        );
        assert_eq!(enemies[0].pick_turn, Some(4), "cell 4 is last of cells 0-4");
        assert!(team.participants[..5].iter().all(|p| !p.is_enemy));
    }

    #[test]
    fn splits_a_five_stack_lobby_in_half_without_puuids() {
        let mut team = five_stack_lobby();
        team.participants[7].is_self = true;
        let hidden: Vec<_> = (0..5)
            .map(|i| ChampSelectTeamMember {
                cell_id: i,
                ..member_defaults()
            })
            .collect();

        mark_enemies(&mut team, &hidden, &[]);

        let allies: Vec<&str> = team
            .participants
            .iter()
            .filter(|p| !p.is_enemy)
            .map(|p| p.puuid.as_str())
            .collect();
        assert_eq!(
            allies,
            vec!["p5", "p6", "p7", "p8", "p9"],
            "your half is the one you're in"
        );
    }

    #[test]
    fn leaves_a_normal_lobby_alone() {
        let mut team = lobby::Lobby {
            participants: (0..5).map(|i| participant(&format!("p{i}"), "P")).collect(),
        };
        mark_enemies(&mut team, &[], &[]);
        assert!(team.participants.iter().all(|p| !p.is_enemy));
    }

    #[test]
    fn marks_only_the_local_player_as_self() {
        let mut team = lobby::Lobby {
            participants: vec![participant("me", "Me"), participant("them", "Them")],
        };
        let slots = vec![
            ChampSelectTeamMember {
                cell_id: 3,
                puuid: "me".to_string(),
                ..member_defaults()
            },
            ChampSelectTeamMember {
                cell_id: 4,
                ..member_defaults()
            },
        ];

        mark_local_player(&mut team, &slots, 3);
        assert!(team.participants[0].is_self);
        assert!(!team.participants[1].is_self);

        mark_local_player(&mut team, &slots, -1);
        assert!(!team.participants[0].is_self);
    }

    #[test]
    fn guesses_hidden_slots_from_history_and_spells() {
        let mut team = lobby::Lobby {
            participants: vec![participant("adc", "Adc"), participant("jungler", "Jungler")],
        };
        let hidden = |cell_id, position: &str, spells: (i64, i64)| ChampSelectTeamMember {
            assigned_position: position.to_string(),
            cell_id,
            spell1_id: spells.0,
            spell2_id: spells.1,
            ..member_defaults()
        };
        let slots = vec![hidden(1, "jungle", (11, 4)), hidden(2, "bottom", (4, 21))];
        let mut cache = PlayerCache::default();
        for (puuid, position, spells) in
            [("adc", "bottom", (4, 21)), ("jungler", "jungle", (11, 4))]
        {
            let mut history = roles::PlayerHistory {
                games: 10,
                ..Default::default()
            };
            history.positions.insert(position.to_string(), 8);
            history.spells.insert(spells, 9);
            assert!(cache.histories.start_attempt(puuid));
            cache.histories.record("Match history", puuid, Ok(history));
        }

        guess_hidden_positions(&mut team, &slots, &cache);

        assert_eq!(team.participants[0].puuid, "jungler");
        assert_eq!(
            team.participants[0].assigned_position.as_deref(),
            Some("jungle")
        );
        assert!(team.participants[0].position_guessed);
        assert_eq!(
            team.participants[1].assigned_position.as_deref(),
            Some("bottom")
        );
        assert_eq!(team.participants[1].cell_id, Some(2));
    }

    #[test]
    fn maps_chat_participants_to_pick_order_and_roles_by_puuid() {
        let mut team = lobby::Lobby {
            participants: vec![
                participant("second-puuid", "Second Player"),
                participant("first-puuid", "First Player"),
            ],
        };
        let champ_select_team = vec![
            ChampSelectTeamMember {
                assigned_position: "middle".to_string(),
                cell_id: 4,
                puuid: "second-puuid".to_string(),
                ..member_defaults()
            },
            ChampSelectTeamMember {
                assigned_position: "jungle".to_string(),
                cell_id: 2,
                puuid: "first-puuid".to_string(),
                ..member_defaults()
            },
        ];

        let mapped_count = apply_champ_select_context(&mut team, &champ_select_team);

        assert_eq!(mapped_count, 2);
        assert_eq!(team.participants[0].puuid, "first-puuid");
        assert_eq!(team.participants[0].pick_turn, Some(0));
        assert_eq!(team.participants[0].cell_id, Some(2));
        assert_eq!(
            team.participants[0].assigned_position.as_deref(),
            Some("jungle")
        );
        assert_eq!(team.participants[1].puuid, "second-puuid");
        assert_eq!(team.participants[1].pick_turn, Some(1));
    }

    #[test]
    fn player_cache_applies_details_and_settles_after_attempts() {
        let mut team = lobby::Lobby {
            participants: vec![
                participant("found", "Found"),
                participant("failing", "Failing"),
            ],
        };
        let mut cache = PlayerCache::default();
        assert!(cache.ranks.start_attempt("found"));
        cache.ranks.record(
            "Rank",
            "found",
            Ok(ranked::RankSummary {
                tier: "GOLD".to_string(),
                division: Some("I".to_string()),
                league_points: 12,
                wins: 10,
                losses: 8,
                provisional: false,
            }),
        );
        assert!(cache.mains.start_attempt("found"));
        cache.mains.record(
            "Champion mastery",
            "found",
            Ok(vec![champions::ChampionMain {
                champion_id: 157,
                name: "Yasuo".to_string(),
                level: 40,
                points: 406_821,
            }]),
        );

        cache.apply(&mut team);
        assert_eq!(team.participants[0].rank.as_ref().unwrap().tier, "GOLD");
        assert_eq!(
            team.participants[0].mains.as_ref().unwrap()[0].name,
            "Yasuo"
        );
        assert_eq!(team.participants[1].rank, None);
        assert!(!cache.is_settled(&team));

        for _ in 0..MAX_LOOKUP_ATTEMPTS {
            assert!(cache.ranks.start_attempt("failing"));
            cache
                .ranks
                .record("Rank", "failing", Err(anyhow::anyhow!("offline")));
            assert!(cache.mains.start_attempt("failing"));
            cache.mains.record(
                "Champion mastery",
                "failing",
                Err(anyhow::anyhow!("offline")),
            );
        }
        assert!(!cache.ranks.start_attempt("failing"));
        assert!(!cache.is_settled(&team), "profile icons are still pending");
        for puuid in ["found", "failing"] {
            assert!(cache.profile_icons.start_attempt(puuid));
            cache.profile_icons.record("Profile icon", puuid, Ok(7085));
            assert!(cache.histories.start_attempt(puuid));
            cache.histories.record(
                "Match history",
                puuid,
                Ok(roles::PlayerHistory {
                    recent: ranked::RecentForm { wins: 6, losses: 7 },
                    ..Default::default()
                }),
            );
        }
        assert!(cache.is_settled(&team));
        cache.apply(&mut team);
        assert_eq!(team.participants[1].profile_icon_id, Some(7085));
        assert!(!cache.ranks.start_attempt("found"));
    }

    #[test]
    fn leaves_unmatched_chat_participants_visible() {
        let mut team = lobby::Lobby {
            participants: vec![participant("chat-only-puuid", "Chat Player")],
        };

        let mapped_count = apply_champ_select_context(&mut team, &[]);

        assert_eq!(mapped_count, 0);
        assert_eq!(team.participants[0].puuid, "chat-only-puuid");
        assert_eq!(team.participants[0].pick_turn, None);
        assert_eq!(team.participants[0].assigned_position, None);
    }
}
