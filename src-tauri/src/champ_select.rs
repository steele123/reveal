use crate::{analytics, lobby, region::RegionInfo, summoner, utils::display_champ_select, Config};
use serde::{Deserialize, Serialize};
use shaco::rest::RESTClient;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectSession {
    pub allow_battle_boost: bool,
    pub allow_duplicate_picks: bool,
    pub allow_locked_events: bool,
    pub allow_rerolling: bool,
    pub allow_skin_selection: bool,
    pub bench_enabled: bool,
    pub boostable_skin_count: i64,
    pub counter: i64,
    pub game_id: u64,
    pub has_simultaneous_bans: bool,
    pub has_simultaneous_picks: bool,
    pub is_custom_game: bool,
    pub is_spectating: bool,
    pub local_player_cell_id: i64,
    pub locked_event_index: i64,
    //pub my_team: Vec<Team>,
    pub recovery_counter: i64,
    pub rerolls_remaining: i64,
    pub skip_champion_select: bool,
    pub timer: Timer,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub actor_cell_id: i64,
    pub champion_id: i64,
    pub completed: bool,
    pub id: i64,
    pub is_ally_action: bool,
    pub is_in_progress: bool,
    #[serde(rename = "type")]
    pub action_type: Type,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Ban,
    Pick,
    #[serde(rename = "ten_bans_reveal")]
    TenBansReveal,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bans {
    pub my_team_bans: Vec<Option<serde_json::Value>>,
    pub num_bans: i64,
    pub their_team_bans: Vec<Option<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatDetails {
    pub muc_jwt_dto: MucJwtDto,
    pub multi_user_chat_id: String,
    pub multi_user_chat_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MucJwtDto {
    pub channel_claim: String,
    pub domain: String,
    pub jwt: String,
    pub target_region: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub assigned_position: String,
    pub cell_id: i64,
    pub champion_id: i64,
    pub champion_pick_intent: i64,
    pub name_visibility_type: NameVisibilityType,
    pub obfuscated_puuid: String,
    pub obfuscated_summoner_id: i64,
    pub puuid: String,
    pub selected_skin_id: i64,
    pub spell1_id: i64,
    pub spell2_id: i64,
    pub summoner_id: i64,
    pub team: i64,
    pub ward_skin_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NameVisibilityType {
    #[serde(rename = "HIDDEN")]
    Hidden,
    #[serde(rename = "UNHIDDEN")]
    Unhidden,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickOrderSwap {
    pub cell_id: i64,
    pub id: i64,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub adjusted_time_left_in_phase: u64,
    pub internal_now_in_epoch_ms: u64,
    pub is_infinite: bool,
    pub phase: String,
    pub total_time_in_phase: i64,
}

pub async fn handle_champ_select_start(
    app_client: &RESTClient,
    remoting_client: &RESTClient,
    config: &Config,
    app_handle: &AppHandle,
) {
    let team = lobby::get_lobby_info(app_client).await;
    let region_info: RegionInfo = serde_json::from_value(
        app_client
            .get("/riotclient/region-locale".to_string())
            .await
            .unwrap(),
    )
    .unwrap();

    app_handle.emit_all("champ_select_started", &team).unwrap();

    if config.auto_open {
        let region = match region_info.web_region.as_str() {
            "SG2" => "SG",
            _ => &region_info.web_region,
        };

        display_champ_select(&team, region, &config.multi_provider);
    }

    let summoner = summoner::get_current_summoner(remoting_client).await;
    analytics::send_analytics_event(&team, &summoner, &region_info).await;
}
