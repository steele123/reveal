//! In-game scoreboard from League's Live Client Data API (127.0.0.1:2999), which the game runs
//! locally for third-party tools. Enemy ranks come from the League Client, when it is open.
//!
//! Only data Riot's API reports is shown; no cooldown tracking (Riot's third-party policy).

use crate::{
    ranked::{self, RankSummary, RecentForm},
    roles,
};
use serde::Serialize;
use serde_json::Value;
use shaco::rest::RESTClient;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tauri::{AppHandle, Manager};

const LIVE_CLIENT_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";
const POLL_INTERVAL: Duration = Duration::from_secs(3);
/// How long to wait through the loading screen for the game's API to come up.
const MAX_LOADING_POLLS: u32 = 100; // 5 minutes
const CDRAGON: &str =
    "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default";

static WATCHING: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveGame {
    pub game_time: f64,
    pub game_mode: String,
    pub players: Vec<LivePlayer>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LivePlayer {
    pub game_name: String,
    pub tag_line: String,
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
pub async fn watch(app_handle: AppHandle, remoting_client: RESTClient) {
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
    let mut last_sent: Option<LiveGame> = None;
    let mut loading_polls = 0;

    loop {
        let phase = remoting_client
            .get("/lol-gameflow/v1/gameflow-phase".to_string())
            .await
            .map(|phase| phase.as_str().unwrap_or_default().to_string())
            .unwrap_or_default();
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
                lookups.fill(&remoting_client, &mut live).await;
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
            let (game_name, tag_line) = match riot_id.split_once('#') {
                Some((name, tag)) => (name.to_string(), tag.to_string()),
                None => (text(&player["summonerName"]), String::new()),
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
        players,
    }
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

/// Ranks and recent form per player, looked up once per game through the League Client.
#[derive(Default)]
struct PlayerLookups {
    done: HashMap<String, Option<PlayerDetails>>,
}

impl PlayerLookups {
    async fn fill(&mut self, remoting_client: &RESTClient, game: &mut LiveGame) {
        for player in &mut game.players {
            if player.is_bot || player.tag_line.is_empty() {
                continue;
            }
            let key = format!("{}#{}", player.game_name, player.tag_line);
            if !self.done.contains_key(&key) {
                let found = lookup(remoting_client, &player.game_name, &player.tag_line).await;
                self.done.insert(key.clone(), found);
            }
            if let Some(Some((puuid, rank, recent))) = self.done.get(&key) {
                player.puuid = Some(puuid.clone());
                player.rank = rank.clone();
                player.recent = recent.clone();
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
    fn builds_item_icon_urls_from_game_data_paths() {
        assert_eq!(
            item_icon_url(
                "/lol-game-data/assets/ASSETS/Items/Icons2D/1001_Class_T1_BootsofSpeed.png"
            ),
            format!("{CDRAGON}/assets/items/icons2d/1001_class_t1_bootsofspeed.png")
        );
    }
}
