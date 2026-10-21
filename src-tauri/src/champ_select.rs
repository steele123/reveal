use crate::{
    analytics, champions, config::Config, lobby, ranked, region::RegionInfo, roles, summoner,
    utils::display_champ_select,
};
use serde::Deserialize;
use shaco::rest::RESTClient;
use std::{
    collections::HashMap,
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
    pub timer: Timer,
    #[serde(default)]
    pub my_team: Vec<ChampSelectTeamMember>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectTeamMember {
    #[serde(default)]
    pub assigned_position: String,
    pub cell_id: u32,
    pub pick_turn: u32,
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

    let mut auto_opened = false;
    let mut last_participant_count = 0;
    let mut last_emitted_team: Option<lobby::Lobby> = None;
    let mut first_participant_seen_at: Option<Instant> = None;
    let mut session_warning_logged = false;
    let mut players = PlayerCache::default();
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

        let session = match get_champ_select_session(remoting_client).await {
            Ok(session) => {
                session_warning_logged = false;
                apply_champ_select_context(&mut team, &session.my_team);
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
        let participant_count = team.participants.len();

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
            if let Err(error) = display_champ_select(&team, region, &config.multi_provider) {
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
        participant.pick_turn = Some(member.pick_turn);
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
        .filter(|&index| lobby.participants[index].cell_id.is_none())
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
        participant.pick_turn = Some(member.pick_turn);
        participant.position_guessed = true;
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
        }
    }

    fn member_defaults() -> ChampSelectTeamMember {
        ChampSelectTeamMember {
            assigned_position: String::new(),
            cell_id: 0,
            pick_turn: 0,
            puuid: String::new(),
            spell1_id: 0,
            spell2_id: 0,
            champion_id: 0,
            champion_pick_intent: 0,
            is_autofilled: false,
        }
    }

    #[test]
    fn guesses_hidden_slots_from_history_and_spells() {
        let mut team = lobby::Lobby {
            participants: vec![participant("adc", "Adc"), participant("jungler", "Jungler")],
        };
        let hidden = |cell_id, position: &str, spells: (i64, i64)| ChampSelectTeamMember {
            assigned_position: position.to_string(),
            cell_id,
            pick_turn: cell_id,
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
                pick_turn: 1,
                puuid: "second-puuid".to_string(),
                ..member_defaults()
            },
            ChampSelectTeamMember {
                assigned_position: "jungle".to_string(),
                cell_id: 2,
                pick_turn: 0,
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
