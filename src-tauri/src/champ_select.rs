use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectSession {
    pub actions: Vec<Vec<Action>>,
    pub allow_battle_boost: bool,
    pub allow_duplicate_picks: bool,
    pub allow_locked_events: bool,
    pub allow_rerolling: bool,
    pub allow_skin_selection: bool,
    pub bans: Bans,
    pub bench_champions: Vec<Option<serde_json::Value>>,
    pub bench_enabled: bool,
    pub boostable_skin_count: i64,
    pub chat_details: ChatDetails,
    pub counter: i64,
    pub game_id: i64,
    pub has_simultaneous_bans: bool,
    pub has_simultaneous_picks: bool,
    pub is_custom_game: bool,
    pub is_spectating: bool,
    pub local_player_cell_id: i64,
    pub locked_event_index: i64,
    pub my_team: Vec<MyTeam>,
    pub pick_order_swaps: Vec<Option<serde_json::Value>>,
    pub recovery_counter: i64,
    pub rerolls_remaining: i64,
    pub skip_champion_select: bool,
    pub their_team: Vec<Option<serde_json::Value>>,
    pub timer: Timer,
    pub trades: Vec<Option<serde_json::Value>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub actor_cell_id: i64,
    pub champion_id: i64,
    pub completed: bool,
    pub id: i64,
    pub is_ally_action: bool,
    pub is_in_progress: bool,
    pub pick_turn: i64,
    #[serde(rename = "type")]
    pub action_type: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bans {
    pub my_team_bans: Vec<Option<serde_json::Value>>,
    pub num_bans: i64,
    pub their_team_bans: Vec<Option<serde_json::Value>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatDetails {
    pub muc_jwt_dto: MucJwtDto,
    pub multi_user_chat_id: String,
    pub multi_user_chat_password: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MucJwtDto {
    pub channel_claim: String,
    pub domain: String,
    pub jwt: String,
    pub target_region: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyTeam {
    pub assigned_position: String,
    pub cell_id: i64,
    pub champion_id: i64,
    pub champion_pick_intent: i64,
    pub name_visibility_type: String,
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub adjusted_time_left_in_phase: u64,
    pub internal_now_in_epoch_ms: u64,
    pub is_infinite: bool,
    pub phase: String,
    pub total_time_in_phase: i64,
}
