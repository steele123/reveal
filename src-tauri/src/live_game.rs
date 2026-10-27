//! In-game scoreboard from League's Live Client Data API (127.0.0.1:2999), which the game runs
//! locally for third-party tools. Enemy ranks come from the League Client, when it is open.
//!
//! Only data Riot's API reports is shown; no cooldown tracking (Riot's third-party policy).

use crate::{
    lobby::{Lobby, Participant},
    lookup::Lookup,
    ranked::{self, RankSummary, RecentForm},
    region::RegionInfo,
    roles,
};
use serde::Serialize;
use serde_json::Value;
use shaco::rest::{LCUClientInfo, RESTClient};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::{AppHandle, Manager};

const LIVE_CLIENT_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";
const LIVE_CLIENT_STATS_URL: &str = "https://127.0.0.1:2999/liveclientdata/gamestats";
const POLL_INTERVAL: Duration = Duration::from_secs(3);
const DETECT_INTERVAL: Duration = Duration::from_secs(5);
/// How long to wait through the loading screen for the game's API to come up.
const MAX_LOADING_POLLS: u32 = 100; // 5 minutes
const CDRAGON: &str =
    "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default";

static WATCHING: AtomicBool = AtomicBool::new(false);
/// The last Champ Select team, to put names back on teammates Riot hides in game.
static CHAMP_SELECT_ROSTER: Mutex<Vec<Participant>> = Mutex::new(Vec::new());
/// The last League Client connection and region. The client often closes during games, but
/// its API may still answer, and the region is still right for profile links.
static LAST_CLIENT: Mutex<Option<LCUClientInfo>> = Mutex::new(None);
static LAST_REGION: Mutex<String> = Mutex::new(String::new());

pub fn remember_client(info: &LCUClientInfo) {
    if let Ok(mut last) = LAST_CLIENT.lock() {
        *last = Some(info.clone());
    }
}

pub fn remember_region(region: &str) {
    if let Ok(mut last) = LAST_REGION.lock() {
        *last = region.to_string();
    }
}

const DEFAULT_INSTALL_DIR: &str = r"C:\Riot Games\League of Legends";
const PRODUCT_SETTINGS: &str = r"C:\ProgramData\Riot Games\Metadata\league_of_legends.live\league_of_legends.live.product_settings.yaml";

/// Connection details from League's lockfile ("LeagueClient:pid:port:password:https"), which the
/// client's background process keeps while its window is closed during games.
fn lockfile_client_info() -> Option<LCUClientInfo> {
    let install_dir = std::fs::read_to_string(PRODUCT_SETTINGS)
        .ok()
        .and_then(|settings| install_dir_from_settings(&settings))
        .unwrap_or_else(|| DEFAULT_INSTALL_DIR.to_string());
    let lockfile =
        std::fs::read_to_string(std::path::Path::new(&install_dir).join("lockfile")).ok()?;
    client_info_from_lockfile(&lockfile)
}

fn install_dir_from_settings(settings: &str) -> Option<String> {
    settings.lines().find_map(|line| {
        line.trim()
            .strip_prefix("product_install_full_path:")
            .map(|path| path.trim().trim_matches('"').to_string())
            .filter(|path| !path.is_empty())
    })
}

fn client_info_from_lockfile(lockfile: &str) -> Option<LCUClientInfo> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let parts: Vec<&str> = lockfile.trim().split(':').collect();
    let [_, _, port, password, _] = parts.as_slice() else {
        return None;
    };
    let port: u16 = port.parse().ok()?;
    let token = STANDARD.encode(format!("riot:{password}"));
    // The lockfile's port serves both the client API and the riotclient endpoints Reveal uses.
    Some(LCUClientInfo {
        port,
        token: token.clone(),
        remoting_port: port,
        remoting_token: token,
    })
}

/// Watches for games by asking the game itself, so the scoreboard works even when the League
/// Client is closed during games or Reveal is opened mid-game.
pub async fn detect(app_handle: AppHandle) {
    let Ok(http) = local_client() else { return };
    loop {
        tokio::time::sleep(DETECT_INTERVAL).await;
        if WATCHING.load(Ordering::Acquire) {
            continue;
        }
        let game_running = matches!(
            http.get(LIVE_CLIENT_STATS_URL).send().await,
            Ok(response) if response.status().is_success()
        );
        if !game_running {
            continue;
        }
        // The lockfile is current even while the client's window is closed; fall back to the
        // last connection Reveal saw.
        let info = lockfile_client_info()
            .or_else(|| LAST_CLIENT.lock().ok().and_then(|last| last.clone()));
        let remoting = info
            .clone()
            .and_then(|info| RESTClient::new(info, true).ok());
        let app = info.and_then(|info| RESTClient::new(info, false).ok());
        watch(app_handle.clone(), remoting, app).await;
    }
}

pub fn remember_champ_select(team: &Lobby) {
    if let Ok(mut roster) = CHAMP_SELECT_ROSTER.lock() {
        *roster = team.participants.clone();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveGame {
    pub game_time: f64,
    pub game_mode: String,
    /// Web region for profile links, e.g. "NA"; empty if the League Client is closed.
    pub region: String,
    /// The queue's display name, e.g. "Ranked Solo/Duo", when the League Client is reachable.
    pub queue_name: Option<String>,
    pub players: Vec<LivePlayer>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LivePlayer {
    pub game_name: String,
    pub tag_line: String,
    /// Riot hid this player's Riot ID and Reveal couldn't recover it.
    pub name_hidden: bool,
    /// "ORDER" (blue side) or "CHAOS" (red side).
    pub team: String,
    pub is_self: bool,
    pub is_bot: bool,
    pub champion_name: String,
    pub champion_id: Option<i64>,
    pub position: String,
    pub level: u32,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub creep_score: u32,
    pub is_dead: bool,
    pub respawn_timer: f64,
    pub items: Vec<LiveItem>,
    /// Filled in from the League Client when it is open.
    pub puuid: Option<String>,
    pub rank: Option<RankSummary>,
    pub recent: Option<RecentForm>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveItem {
    pub id: i64,
    pub name: String,
    pub slot: i64,
    pub icon: Option<String>,
}

/// Watches the game until it ends, emitting `live_game_update` with the scoreboard and
/// `null` when the game is over. Only one watcher runs at a time.
pub async fn watch(
    app_handle: AppHandle,
    remoting_client: Option<RESTClient>,
    app_client: Option<RESTClient>,
) {
    if WATCHING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    log_info!("Watching the live game");

    let http = match local_client() {
        Ok(http) => http,
        Err(error) => {
            log_error!("Could not create the live game client: {error}");
            WATCHING.store(false, Ordering::Release);
            return;
        }
    };
    let mut assets = Assets::default();
    let mut lookups = PlayerLookups::default();
    let mut session_players: Vec<SessionPlayer> = Vec::new();
    let queue_name = match &remoting_client {
        Some(client) => crate::queue::current_queue_name(client).await,
        None => None,
    };
    let region = match &app_client {
        Some(app_client) => web_region(app_client).await,
        None => None,
    }
    .or_else(|| LAST_REGION.lock().ok().map(|last| last.clone()))
    .unwrap_or_default();
    let mut last_sent: Option<LiveGame> = None;
    let mut loading_polls = 0;

    loop {
        let phase = match &remoting_client {
            Some(client) => client
                .get("/lol-gameflow/v1/gameflow-phase".to_string())
                .await
                .map(|phase| phase.as_str().unwrap_or_default().to_string())
                .unwrap_or_default(),
            None => String::new(),
        };
        // The client can be closed during games; keep watching while the game API answers.
        let in_game = phase.is_empty() || phase == "InProgress" || phase == "Reconnect";

        let game = match http.get(LIVE_CLIENT_URL).send().await {
            Ok(response) if response.status().is_success() => response.json::<Value>().await.ok(),
            _ => None,
        };

        match game {
            Some(data) if in_game => {
                assets.load_once().await;
                let mut live = parse_live_game(&data, &assets);
                live.region = region.clone();
                live.queue_name = queue_name.clone();
                if session_players.is_empty() {
                    if let Some(client) = &remoting_client {
                        session_players = game_session_players(client).await;
                    }
                }
                let roster = CHAMP_SELECT_ROSTER
                    .lock()
                    .map(|roster| roster.clone())
                    .unwrap_or_default();
                recover_hidden_players(&mut live, &roster, &session_players);
                if let Some(client) = &remoting_client {
                    lookups.fill(client, &mut live);
                }
                if last_sent.as_ref() != Some(&live) {
                    let _ = app_handle.emit_all("live_game_update", &live);
                    last_sent = Some(live);
                }
            }
            // Loading screen: the API isn't up yet.
            None if in_game && last_sent.is_none() && loading_polls < MAX_LOADING_POLLS => {
                loading_polls += 1;
            }
            _ => break,
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }

    let _ = app_handle.emit_all("live_game_update", Option::<LiveGame>::None);
    WATCHING.store(false, Ordering::Release);
    log_info!("Live game ended");
}

/// The game's API uses a Riot-signed certificate for 127.0.0.1. This client only ever talks to
/// the local game, so it skips certificate checks rather than bundling Riot's root.
fn local_client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(2))
        .build()
}

fn parse_live_game(data: &Value, assets: &Assets) -> LiveGame {
    let me = data["activePlayer"]["riotId"].as_str().unwrap_or_default();
    let text = |value: &Value| value.as_str().unwrap_or_default().to_string();
    let number = |value: &Value| value.as_u64().unwrap_or(0) as u32;

    let players = data["allPlayers"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|player| {
            let riot_id = text(&player["riotId"]);
            // Hidden players have no Riot ID (sometimes a placeholder without a tag).
            let (game_name, tag_line) = match riot_id.split_once('#') {
                Some((name, tag)) if !name.is_empty() => (name.to_string(), tag.to_string()),
                _ => (String::new(), String::new()),
            };
            let scores = &player["scores"];
            let mut items: Vec<LiveItem> = player["items"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|item| {
                    let id = item["itemID"].as_i64().unwrap_or(0);
                    LiveItem {
                        id,
                        name: text(&item["displayName"]),
                        slot: item["slot"].as_i64().unwrap_or(0),
                        icon: assets.item_icons.get(&id).cloned(),
                    }
                })
                .collect();
            items.sort_by_key(|item| item.slot);
            let champion_alias =
                text(&player["rawChampionName"]).replace("game_character_displayname_", "");
            LivePlayer {
                is_self: !me.is_empty() && riot_id == me,
                name_hidden: game_name.is_empty(),
                game_name,
                tag_line,
                team: text(&player["team"]),
                is_bot: player["isBot"].as_bool().unwrap_or(false),
                champion_id: assets
                    .champion_ids
                    .get(&champion_alias.to_lowercase())
                    .copied(),
                champion_name: text(&player["championName"]),
                position: text(&player["position"]),
                level: number(&player["level"]),
                kills: number(&scores["kills"]),
                deaths: number(&scores["deaths"]),
                assists: number(&scores["assists"]),
                creep_score: number(&scores["creepScore"]),
                is_dead: player["isDead"].as_bool().unwrap_or(false),
                respawn_timer: player["respawnTimer"]
                    .as_f64()
                    .unwrap_or(0.0)
                    .max(0.0)
                    .round(),
                items,
                ..Default::default()
            }
        })
        .collect();

    LiveGame {
        // Whole seconds, so the scoreboard only re-sends when something changes.
        game_time: data["gameData"]["gameTime"].as_f64().unwrap_or(0.0).floor(),
        game_mode: text(&data["gameData"]["gameMode"]),
        region: String::new(),
        queue_name: None,
        players,
    }
}

/// A player from the League Client's game session, which can include their puuid even when
/// the game hides their Riot ID.
#[derive(Clone, Debug, Default, PartialEq)]
struct SessionPlayer {
    champion_id: i64,
    puuid: String,
    game_name: String,
    tag_line: String,
}

async fn game_session_players(remoting_client: &RESTClient) -> Vec<SessionPlayer> {
    match remoting_client
        .get("/lol-gameflow/v1/session".to_string())
        .await
    {
        Ok(session) => parse_session_players(&session),
        Err(_) => Vec::new(),
    }
}

fn parse_session_players(session: &Value) -> Vec<SessionPlayer> {
    let text = |value: &Value| value.as_str().unwrap_or_default().to_string();
    ["teamOne", "teamTwo"]
        .iter()
        .flat_map(|team| {
            session["gameData"][team]
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
        .filter_map(|player| {
            let champion_id = player["championId"].as_i64().filter(|id| *id > 0)?;
            let game_name = [&player["gameName"], &player["riotIdGameName"]]
                .into_iter()
                .map(text)
                .find(|name| !name.is_empty())
                .unwrap_or_default();
            let tag_line = [&player["tagLine"], &player["riotIdTagLine"]]
                .into_iter()
                .map(text)
                .find(|tag| !tag.is_empty())
                .unwrap_or_default();
            Some(SessionPlayer {
                champion_id,
                puuid: text(&player["puuid"]),
                game_name,
                tag_line,
            })
        })
        .collect()
}

/// Puts names and puuids back on hidden players, matched by champion: teammates from the
/// Champ Select roster, then anyone the League Client's game session still identifies.
fn recover_hidden_players(game: &mut LiveGame, roster: &[Participant], session: &[SessionPlayer]) {
    let my_team = game
        .players
        .iter()
        .find(|player| player.is_self)
        .map(|player| player.team.clone());

    for player in &mut game.players {
        let Some(champion_id) = player.champion_id else {
            continue;
        };

        if player.name_hidden && my_team.as_deref() == Some(player.team.as_str()) {
            if let Some(teammate) = roster
                .iter()
                .find(|teammate| teammate.champion_id == Some(champion_id))
            {
                player.game_name = teammate.game_name.clone();
                player.tag_line = teammate.game_tag.clone();
                player.puuid = (!teammate.puuid.is_empty()).then(|| teammate.puuid.clone());
                player.name_hidden = false;
            }
        }

        if let Some(known) = session
            .iter()
            .find(|known| known.champion_id == champion_id)
        {
            if player.puuid.is_none() && !known.puuid.is_empty() {
                player.puuid = Some(known.puuid.clone());
            }
            if player.name_hidden && !known.game_name.is_empty() && !known.tag_line.is_empty() {
                player.game_name = known.game_name.clone();
                player.tag_line = known.tag_line.clone();
                player.name_hidden = false;
            }
        }
    }
}

async fn web_region(app_client: &RESTClient) -> Option<String> {
    let info: RegionInfo = serde_json::from_value(
        app_client
            .get("/riotclient/region-locale".to_string())
            .await
            .ok()?,
    )
    .ok()?;
    Some(info.web())
}

/// Champion ids and item icons from Community Dragon, fetched once per app run.
#[derive(Default)]
struct Assets {
    loaded: bool,
    champion_ids: HashMap<String, i64>,
    item_icons: HashMap<i64, String>,
}

impl Assets {
    async fn load_once(&mut self) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        if let Ok(champions) = fetch_json(&format!("{CDRAGON}/v1/champion-summary.json")).await {
            for champion in champions.as_array().into_iter().flatten() {
                if let (Some(alias), Some(id)) =
                    (champion["alias"].as_str(), champion["id"].as_i64())
                {
                    self.champion_ids.insert(alias.to_lowercase(), id);
                }
            }
        }
        if let Ok(items) = fetch_json(&format!("{CDRAGON}/v1/items.json")).await {
            for item in items.as_array().into_iter().flatten() {
                if let (Some(id), Some(path)) = (item["id"].as_i64(), item["iconPath"].as_str()) {
                    self.item_icons.insert(id, item_icon_url(path));
                }
            }
        }
        log_info!(
            "Loaded live game assets: {} champions, {} items",
            self.champion_ids.len(),
            self.item_icons.len()
        );
    }
}

/// "/lol-game-data/assets/ASSETS/Items/Icons2D/1001_Boots.png" -> Community Dragon URL.
fn item_icon_url(path: &str) -> String {
    let relative = path
        .trim_start_matches("/lol-game-data/assets/")
        .to_lowercase();
    format!("{CDRAGON}/{relative}")
}

async fn fetch_json(url: &str) -> anyhow::Result<Value> {
    Ok(reqwest::Client::new()
        .get(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await?
        .json()
        .await?)
}

/// A player's puuid, Solo/Duo rank and recent form.
type PlayerDetails = (String, Option<RankSummary>, Option<RecentForm>);

/// Ranks and recent form per player, looked up in the background through the League Client.
#[derive(Default)]
struct PlayerLookups {
    details: Lookup<PlayerDetails>,
}

impl PlayerLookups {
    /// Applies the details found so far and starts lookups for players still missing them.
    fn fill(&mut self, remoting_client: &RESTClient, game: &mut LiveGame) {
        self.details.collect("Player details");
        for player in &mut game.players {
            if player.is_bot {
                continue;
            }
            let key = match (&player.puuid, player.tag_line.is_empty()) {
                (Some(puuid), _) => puuid.clone(),
                (None, false) => format!("{}#{}", player.game_name, player.tag_line),
                (None, true) => continue,
            };
            self.details.spawn(&key, || {
                let client = remoting_client.clone();
                let (puuid, game_name, tag_line) = (
                    player.puuid.clone(),
                    player.game_name.clone(),
                    player.tag_line.clone(),
                );
                async move {
                    match puuid {
                        Some(puuid) => details_for(&client, puuid).await,
                        None => lookup(&client, &game_name, &tag_line).await,
                    }
                    .ok_or_else(|| anyhow::anyhow!("player not found"))
                }
            });
            if let Some((puuid, rank, recent)) = self.details.get(&key) {
                player.puuid = Some(puuid);
                player.rank = rank;
                player.recent = recent;
            }
        }
    }
}

async fn lookup(
    remoting_client: &RESTClient,
    game_name: &str,
    tag_line: &str,
) -> Option<PlayerDetails> {
    let summoner = remoting_client
        .get(format!(
            "/lol-summoner/v1/alias/lookup?gameName={}&tagLine={}",
            urlencoding::encode(game_name),
            urlencoding::encode(tag_line)
        ))
        .await
        .ok()?;
    let puuid = summoner["puuid"]
        .as_str()
        .filter(|puuid| !puuid.is_empty())?
        .to_string();
    details_for(remoting_client, puuid).await
}

async fn details_for(remoting_client: &RESTClient, puuid: String) -> Option<PlayerDetails> {
    let rank = ranked::get_solo_rank(remoting_client, &puuid).await.ok();
    let recent = roles::get_player_history(remoting_client, &puuid)
        .await
        .ok()
        .map(|history| history.recent);
    Some((puuid, rank, recent))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> Value {
        json!({
            "activePlayer": { "riotId": "spooky#2729" },
            "allPlayers": [
                {
                    "riotId": "spooky#2729", "summonerName": "spooky#2729", "team": "ORDER",
                    "championName": "Taliyah", "rawChampionName": "game_character_displayname_Taliyah",
                    "position": "MIDDLE", "level": 11, "isBot": false, "isDead": false, "respawnTimer": 0.0,
                    "scores": { "kills": 4, "deaths": 1, "assists": 7, "creepScore": 142, "wardScore": 9.0 },
                    "items": [
                        { "itemID": 3020, "displayName": "Sorcerer's Shoes", "slot": 1 },
                        { "itemID": 6655, "displayName": "Luden's Companion", "slot": 0 }
                    ]
                },
                {
                    "riotId": "Shopkeeper#123", "team": "CHAOS", "championName": "Wukong",
                    "rawChampionName": "game_character_displayname_MonkeyKing", "position": "TOP",
                    "level": 9, "isBot": false, "isDead": true, "respawnTimer": 12.6,
                    "scores": { "kills": 1, "deaths": 3, "assists": 0, "creepScore": 98 }, "items": []
                }
            ],
            "gameData": { "gameTime": 812.73, "gameMode": "CLASSIC" }
        })
    }

    #[test]
    fn parses_players_scores_items_and_champion_ids() {
        let assets = Assets {
            loaded: true,
            champion_ids: HashMap::from([
                ("taliyah".to_string(), 163),
                ("monkeyking".to_string(), 62),
            ]),
            item_icons: HashMap::from([(6655, "https://cdragon/ludens.png".to_string())]),
        };

        let game = parse_live_game(&sample(), &assets);

        assert_eq!(game.game_time, 812.0);
        assert_eq!(game.players.len(), 2);
        let me = &game.players[0];
        assert!(me.is_self);
        assert_eq!(
            (me.game_name.as_str(), me.tag_line.as_str()),
            ("spooky", "2729")
        );
        assert_eq!(me.champion_id, Some(163));
        assert_eq!(
            (me.kills, me.deaths, me.assists, me.creep_score),
            (4, 1, 7, 142)
        );
        assert_eq!(
            me.items.iter().map(|item| item.id).collect::<Vec<_>>(),
            vec![6655, 3020]
        );
        assert_eq!(
            me.items[0].icon.as_deref(),
            Some("https://cdragon/ludens.png")
        );

        let enemy = &game.players[1];
        assert!(!enemy.is_self);
        assert_eq!(enemy.team, "CHAOS");
        assert_eq!(
            enemy.champion_id,
            Some(62),
            "Wukong's internal alias is MonkeyKing"
        );
        assert!(enemy.is_dead);
        assert_eq!(enemy.respawn_timer, 13.0);
    }

    #[test]
    fn recovers_hidden_teammates_from_champ_select_and_enemies_from_the_session() {
        let mut data = sample();
        data["allPlayers"][1]["riotId"] = json!("");
        data["allPlayers"].as_array_mut().unwrap().push(json!({
            "riotId": "", "team": "ORDER", "championName": "Qiyana",
            "rawChampionName": "game_character_displayname_Qiyana", "position": "JUNGLE",
            "level": 8, "scores": {}, "items": []
        }));
        let assets = Assets {
            loaded: true,
            champion_ids: HashMap::from([
                ("taliyah".to_string(), 163),
                ("monkeyking".to_string(), 62),
                ("qiyana".to_string(), 246),
            ]),
            item_icons: HashMap::new(),
        };
        let mut game = parse_live_game(&data, &assets);
        assert!(game.players[1].name_hidden && game.players[2].name_hidden);

        let mut jungler = Participant {
            game_name: "困困猫".into(),
            game_tag: "0406".into(),
            puuid: "jungler-puuid".into(),
            champion_id: Some(246),
            ..Default::default()
        };
        jungler.cid = "champ-select".into();
        let session = parse_session_players(&json!({ "gameData": {
            "teamOne": [{ "championId": 246, "puuid": "" }],
            "teamTwo": [{ "championId": 62, "puuid": "enemy-puuid", "gameName": "", "tagLine": "" }]
        }}));

        recover_hidden_players(&mut game, &[jungler], &session);

        let ally = &game.players[2];
        assert!(!ally.name_hidden);
        assert_eq!(
            (ally.game_name.as_str(), ally.tag_line.as_str()),
            ("困困猫", "0406")
        );
        assert_eq!(ally.puuid.as_deref(), Some("jungler-puuid"));

        let enemy = &game.players[1];
        assert!(enemy.name_hidden, "no name anywhere, so it stays hidden");
        assert_eq!(
            enemy.puuid.as_deref(),
            Some("enemy-puuid"),
            "but ranks can still load"
        );
    }

    #[test]
    fn reads_the_lockfile_and_install_dir() {
        let info = client_info_from_lockfile(
            "LeagueClient:1234:61234:s3cret:https
",
        )
        .unwrap();
        assert_eq!((info.port, info.remoting_port), (61234, 61234));
        assert_eq!(info.token, "cmlvdDpzM2NyZXQ=", "base64 of riot:s3cret");
        assert!(client_info_from_lockfile("garbage").is_none());

        let settings = "product_install_full_path: \"D:/Games/League of Legends\"
product_install_root: \"D:/Games\"";
        assert_eq!(
            install_dir_from_settings(settings).as_deref(),
            Some("D:/Games/League of Legends")
        );
    }

    #[test]
    fn builds_item_icon_urls_from_game_data_paths() {
        assert_eq!(
            item_icon_url(
                "/lol-game-data/assets/ASSETS/Items/Icons2D/1001_Class_T1_BootsofSpeed.png"
            ),
            format!("{CDRAGON}/assets/items/icons2d/1001_class_t1_bootsofspeed.png")
        );
    }
}
