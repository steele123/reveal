use crate::{
    analytics, config::Config, lobby, region::RegionInfo, summoner, utils::display_champ_select,
};
use serde::Deserialize;
use shaco::rest::RESTClient;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const EXPECTED_PARTICIPANT_COUNT: usize = 5;
const PARTICIPANT_POLL_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectSession {
    pub game_id: u64,
    pub timer: Timer,
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
    let mut first_participant_seen_at: Option<Instant> = None;

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
        let team = match lobby::get_lobby_info(app_client).await {
            Ok(team) => team,
            Err(error) => {
                log_warn!("Lobby participants are not available yet; retrying: {error}");
                tokio::time::sleep(PARTICIPANT_POLL_INTERVAL).await;
                continue;
            }
        };
        let participant_count = team.participants.len();

        if participant_count > 0 && first_participant_seen_at.is_none() {
            first_participant_seen_at = Some(Instant::now());
        }

        if participant_count > last_participant_count {
            log_info!(
                "Champ Select participant count increased to {} from {}",
                participant_count,
                last_participant_count
            );
            last_participant_count = participant_count;

            if let Err(error) = app_handle.emit_all("champ_select_started", &team) {
                log_error!("Failed to emit Champ Select participants: {error}");
            }
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
        if participant_count >= EXPECTED_PARTICIPANT_COUNT {
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
}
