use crate::{
    analytics, config::Config, lobby, region::RegionInfo, summoner, utils::display_champ_select,
};
use serde::Deserialize;
use shaco::rest::RESTClient;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};

const EXPECTED_PARTICIPANT_COUNT: usize = 5;
const PARTICIPANT_POLL_INTERVAL: Duration = Duration::from_secs(2);

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
    pub puuid: String,
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

    // Poll until we have all 5 teammates or champ select ends
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

        let mapped_participant_count = match get_champ_select_session(remoting_client).await {
            Ok(session) => {
                session_warning_logged = false;
                apply_champ_select_context(&mut team, &session.my_team)
            }
            Err(error) => {
                if !session_warning_logged {
                    log_warn!(
                        "Champ Select slot information is not available yet; names will still be revealed: {error}"
                    );
                    session_warning_logged = true;
                }
                0
            }
        };
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

        // Only send analytics once we have all 5 teammates
        if participant_count >= EXPECTED_PARTICIPANT_COUNT
            && mapped_participant_count >= participant_count
        {
            match summoner::get_current_summoner(remoting_client).await {
                Ok(summoner) => {
                    analytics::send_analytics_event(&team, &summoner, &region_info).await;
                }
                Err(error) => {
                    log_warn!("Failed to read current summoner for analytics: {error}");
                }
            }
            log_info!("Complete Champ Select team found; participant polling finished");
            break;
        }

        tokio::time::sleep(PARTICIPANT_POLL_INTERVAL).await;
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
        }
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
            },
            ChampSelectTeamMember {
                assigned_position: "jungle".to_string(),
                cell_id: 2,
                pick_turn: 0,
                puuid: "first-puuid".to_string(),
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
