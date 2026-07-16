use crate::{
    analytics, config::Config, lobby, region::RegionInfo, summoner, utils::display_champ_select,
};
use serde::Deserialize;
use shaco::rest::RESTClient;
use tauri::{AppHandle, Manager};

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
    let region_info: RegionInfo = match app_client
        .get("/riotclient/region-locale".to_string())
        .await
        .map_err(anyhow::Error::from)
        .and_then(|value| serde_json::from_value(value).map_err(anyhow::Error::from))
    {
        Ok(region_info) => region_info,
        Err(error) => {
            eprintln!("Failed to read League region: {error}");
            return;
        }
    };

    let region = match region_info.web_region.as_str() {
        "SG2" => "SG",
        _ => &region_info.web_region,
    };

    let mut auto_opened = false;
    let mut last_participant_count = 0;

    // Poll until we have all 5 teammates or champ select ends
    loop {
        // Champ select sanity check
        let gameflow_state = remoting_client
            .get("/lol-gameflow/v1/gameflow-phase".to_string())
            .await;

        if let Ok(state) = gameflow_state {
            let state_str = state.to_string().replace('\"', "");
            if state_str != "ChampSelect" {
                println!("Left champ select, stopping poll");
                break;
            }
        } else {
            // Invalid gameflow state - assume we're not in champ select
            break;
        }

        // This will return an error if we aren't connected to chat
        let team = match lobby::get_lobby_info(app_client).await {
            Ok(team) => team,
            Err(error) => {
                println!("Lobby info not available yet, retrying: {error}");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
        };
        let participant_count = team.participants.len();

        if participant_count > last_participant_count {
            println!(
                "Found {} participants (was {})",
                participant_count, last_participant_count
            );
            last_participant_count = participant_count;

            if let Err(error) = app_handle.emit_all("champ_select_started", &team) {
                eprintln!("Failed to emit Champ Select participants: {error}");
            }

            // Auto open multi link on first emission
            if config.auto_open && !auto_opened && participant_count > 0 {
                if let Err(error) = display_champ_select(&team, region, &config.multi_provider) {
                    eprintln!("Failed to open multi link: {error}");
                }
                auto_opened = true;
            }
        }

        // Only send analytics once we have all 5 teammates
        if participant_count >= 5 {
            match summoner::get_current_summoner(remoting_client).await {
                Ok(summoner) => {
                    analytics::send_analytics_event(&team, &summoner, &region_info).await;
                }
                Err(error) => {
                    eprintln!("Failed to read current summoner for analytics: {error}");
                }
            }
            println!("Found all 5 participants, stopping poll");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
