//! Master+ matchup stats for Champ Select: the best counters to each enemy pick and how your
//! teammate in that role fares against it.
//!
//! The numbers come from the JSON u.gg's and Lolalytics' own pages load (every region, Master and
//! above, Ranked Solo/Duo), which needs no key or cookies. Both sites are read and weighed
//! together: a counter both agree on ranks above one only one of them backs. U.GG also has leads
//! at 15 minutes, and its role stats place the enemy picks. The window links each number back to
//! the page it comes from.

use crate::{
    champions::{self, ChampionInfo},
    lookup::Lookup,
};
use anyhow::Context;
use serde::Serialize;
use serde_json::Value;
use shaco::rest::RESTClient;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    sync::{Arc, Mutex, MutexGuard, OnceLock},
    time::{Duration, Instant},
};

const STATS_BASE: &str = "https://stats2.u.gg/lol/1.5";
const PATCHES_URL: &str =
    "https://static.bigbrain.gg/assets/lol/riot_patch_update/prod/ugg/patches.json";
const API_VERSIONS_URL: &str =
    "https://static.bigbrain.gg/assets/lol/riot_patch_update/prod/ugg/ugg-api-versions.json";
const DEFAULT_API_VERSION: &str = "1.5.0";
const LOLALYTICS_COUNTERS_URL: &str = "https://a1.lolalytics.com/mega/?ep=counter&v=1";
/// U.GG's keys for every region together and for Master and above.
const WORLD: &str = "12";
const MASTER_PLUS: &str = "14";
/// A patch's first days have too few Master+ games for matchups, so Reveal stays on the
/// previous patch until the new one has this many.
const MIN_PATCH_GAMES: f64 = 100_000.0;
/// U.GG refreshes its stats about once a day.
const STATS_MAX_AGE: Duration = Duration::from_secs(6 * 60 * 60);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const COUNTERS_PER_ENEMY: usize = 15;
/// A counter needs this share of the enemy champion's games in the role, but never more than
/// `MIN_COUNTER_GAMES` nor fewer than `MIN_SPARSE_COUNTER_GAMES` (for rarely played picks).
const COUNTER_GAMES_SHARE: f64 = 0.025;
const MIN_COUNTER_GAMES: f64 = 200.0;
const MIN_SPARSE_COUNTER_GAMES: f64 = 50.0;
/// Each site's matchup win rate is pulled towards what an average pick wins against the enemy as
/// if it had `SHRINK_GAMES` games of that, so a few hundred lucky games don't outrank thousands of
/// solid ones.
const SHRINK_GAMES: f64 = 400.0;
const STATS_KEY: &str = "stats";

/// A lane, sent to the window under League's position names ("top" ... "utility").
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Top,
    Jungle,
    Middle,
    Bottom,
    Utility,
}

impl Role {
    const ALL: [Role; 5] = [
        Role::Top,
        Role::Jungle,
        Role::Middle,
        Role::Bottom,
        Role::Utility,
    ];

    pub fn from_position(position: &str) -> Option<Self> {
        match position.to_lowercase().as_str() {
            "top" => Some(Role::Top),
            "jungle" => Some(Role::Jungle),
            "middle" | "mid" => Some(Role::Middle),
            "bottom" | "adc" => Some(Role::Bottom),
            "utility" | "support" => Some(Role::Utility),
            _ => None,
        }
    }

    /// U.GG's role id in matchup files.
    fn ugg_id(self) -> &'static str {
        match self {
            Role::Jungle => "1",
            Role::Utility => "2",
            Role::Bottom => "3",
            Role::Top => "4",
            Role::Middle => "5",
        }
    }

    /// U.GG's role name in the champion ranking file.
    fn ranking_key(self) -> &'static str {
        match self {
            Role::Top => "top",
            Role::Jungle => "jungle",
            Role::Middle => "mid",
            Role::Bottom => "adc",
            Role::Utility => "supp",
        }
    }

    fn lolalytics_lane(self) -> &'static str {
        match self {
            Role::Top => "top",
            Role::Jungle => "jungle",
            Role::Middle => "middle",
            Role::Bottom => "bottom",
            Role::Utility => "support",
        }
    }
}

/// A stats site. Both are always read; the one picked in the Counters tab supplies the main win
/// rate shown for each matchup.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    Lolalytics,
    Ugg,
}

impl Provider {
    const ALL: [Provider; 2] = [Provider::Lolalytics, Provider::Ugg];

    /// The saved setting; anything unknown means Lolalytics.
    pub fn from_setting(setting: &str) -> Self {
        match setting {
            "ugg" => Provider::Ugg,
            _ => Provider::Lolalytics,
        }
    }

    /// The download for one enemy pick: U.GG has every role in one file, Lolalytics one per lane.
    fn file_key(self, champion_id: i64, role: Role) -> String {
        match self {
            Provider::Ugg => format!("ugg:{champion_id}"),
            Provider::Lolalytics => format!("lolalytics:{champion_id}:{}", role.lolalytics_lane()),
        }
    }
}

/// Enemy roles set by dragging a pick to another lane in the window, by champion id. They count
/// as if Riot had shown them, and last until the next Champ Select.
static ROLE_OVERRIDES: Mutex<Option<HashMap<i64, Role>>> = Mutex::new(None);

/// Puts an enemy pick in `role`. A pick the user had already put there gives the lane up and is
/// guessed again.
pub fn set_role_override(champion_id: i64, role: Role) {
    let mut overrides = lock(&ROLE_OVERRIDES);
    let overrides = overrides.get_or_insert_with(HashMap::new);
    overrides.retain(|_, placed| *placed != role);
    overrides.insert(champion_id, role);
}

pub fn clear_role_overrides() {
    *lock(&ROLE_OVERRIDES) = None;
}

/// Gives enemy picks the roles the user set.
pub fn apply_role_overrides(picks: &mut DraftPicks) {
    let overrides = lock(&ROLE_OVERRIDES);
    let Some(overrides) = overrides.as_ref() else {
        return;
    };
    for enemy in &mut picks.enemies {
        if let Some(role) = overrides.get(&enemy.champion_id) {
            enemy.role = Some(*role);
        }
    }
    // A lane Riot showed for another pick goes to the one the user put there.
    for (champion_id, role) in overrides {
        for enemy in &mut picks.enemies {
            if enemy.champion_id != *champion_id && enemy.role == Some(*role) {
                enemy.role = None;
            }
        }
    }
}

/// What Champ Select shows of each team's picks (locked, or hovered for your own team).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DraftPicks {
    /// False in modes without lanes (ARAM, blind pick), which get no matchups.
    pub has_roles: bool,
    pub self_role: Option<Role>,
    pub allies: Vec<Pick>,
    pub enemies: Vec<Pick>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pick {
    pub champion_id: i64,
    /// Riot hides the enemy team's roles in ranked.
    pub role: Option<Role>,
    pub is_self: bool,
}

/// Counters and lane matchups for every enemy pick, sent to the window.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchupBoard {
    pub provider: Provider,
    /// Riot's name for the patch the stats are from, e.g. "26.21".
    pub patch: Option<String>,
    pub error: Option<String>,
    pub self_role: Option<Role>,
    /// Your team's picks (hovers included), so the window can line the lanes up.
    pub allies: Vec<AllyPick>,
    /// In lane order once their roles are known.
    pub enemies: Vec<EnemyMatchups>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllyPick {
    pub champion: ChampionRef,
    pub role: Role,
    pub is_self: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyMatchups {
    pub champion: ChampionRef,
    pub role: Option<Role>,
    /// Riot hid the role, so it's the likeliest one given what every enemy pick usually plays.
    pub role_guessed: bool,
    /// The enemy champion's own games and win rate in the role.
    pub games: u32,
    pub win_rate: Option<f64>,
    /// Its matchups are still downloading.
    pub loading: bool,
    /// The best picks into it, best first, each from the counter pick's side.
    pub counters: Vec<Counter>,
    /// Your team's pick in the same role against it, from your teammate's side.
    pub ally: Option<AllyMatchup>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionRef {
    pub id: i64,
    pub name: String,
    /// The name stats sites use in URLs, e.g. "leesin"; empty when the client didn't say.
    pub slug: String,
}

/// One side of a matchup, per game. Win rates run 0-100; the "15" stats are U.GG's leads at 15
/// minutes, negative when behind.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchupStats {
    /// The main site's numbers, or the other site's when the main one lacks the matchup.
    pub games: u32,
    pub win_rate: f64,
    /// Win rate minus what both champions' overall win rates in the role predict, in points:
    /// Lolalytics' Δ2, or the same worked out from U.GG's records.
    pub win_rate_delta: Option<f64>,
    pub gold_diff_15: Option<f64>,
    pub xp_diff_15: Option<f64>,
    pub cs_diff_15: Option<f64>,
    pub jungle_cs_diff_15: Option<f64>,
    pub kill_diff_15: Option<f64>,
    /// Each site's own numbers, when it has enough games of the matchup.
    pub lolalytics: Option<SourceStats>,
    pub ugg: Option<SourceStats>,
    /// How many sites see this matchup going better than expected, out of `sources`.
    pub agreement: u8,
    /// How many sites' stats for this enemy pick have loaded.
    pub sources: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceStats {
    pub games: u32,
    pub win_rate: f64,
    pub win_rate_delta: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Counter {
    pub champion: ChampionRef,
    pub stats: MatchupStats,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllyMatchup {
    pub champion: ChampionRef,
    pub is_self: bool,
    /// Missing when neither site has Master+ games of this matchup.
    pub stats: Option<MatchupStats>,
}

/// Downloads for one Champ Select, started in the background as picks come in. The files
/// themselves are cached for the whole run.
#[derive(Default)]
pub struct MatchupLookups {
    stats: Lookup<Arc<PatchStats>>,
    champions: Lookup<Arc<ChampionMatchups>>,
    names: Option<Arc<HashMap<i32, ChampionInfo>>>,
}

impl MatchupLookups {
    /// Stores finished downloads and starts any still missing, without waiting for them.
    pub async fn fetch_missing(&mut self, remoting_client: &RESTClient, picks: &DraftPicks) {
        self.stats.collect("Matchup stats");
        self.champions.collect("Champion matchups");
        if !picks.has_roles {
            return;
        }

        if self.names.is_none() && !(picks.allies.is_empty() && picks.enemies.is_empty()) {
            match champions::get_champion_info(remoting_client).await {
                Ok(names) => self.names = Some(Arc::new(names)),
                Err(error) => log_warn!("Champion lookup for matchups failed: {error}"),
            }
        }

        self.stats.spawn(STATS_KEY, load_patch_stats);
        let Some(stats) = self.stats.get(STATS_KEY) else {
            return;
        };
        let roles = assign_roles(&picks.enemies, &stats);
        for (enemy, role) in picks.enemies.iter().zip(roles) {
            let Some(role) = role else { continue };
            let champion_id = enemy.champion_id;
            let stats = stats.clone();
            self.champions
                .spawn(&Provider::Ugg.file_key(champion_id, role), move || {
                    load_champion(stats, champion_id)
                });
            // Lolalytics names champions in its URLs, so it waits for the client's names.
            if let Some(slug) = self.slug(champion_id) {
                self.champions
                    .spawn(&Provider::Lolalytics.file_key(champion_id, role), move || {
                        load_lolalytics(slug, champion_id, role)
                    });
            }
        }
    }

    /// The board for the picks so far, or None until the enemy team has picked.
    pub fn board(&self, picks: &DraftPicks, main: Provider) -> Option<MatchupBoard> {
        if !picks.has_roles || picks.enemies.is_empty() {
            return None;
        }
        let stats = self.stats.get(STATS_KEY);
        let error = (stats.is_none() && self.stats.is_settled(STATS_KEY))
            .then(|| "Couldn't load Master+ stats from U.GG".to_string());
        let roles = stats
            .as_deref()
            .map(|stats| assign_roles(&picks.enemies, stats))
            .unwrap_or_default();
        let mut files = HashMap::new();
        let mut loading = HashSet::new();
        for (enemy, role) in picks.enemies.iter().zip(roles) {
            let Some(role) = role else { continue };
            let mut pending = false;
            for provider in Provider::ALL {
                let key = provider.file_key(enemy.champion_id, role);
                if let Some(file) = self.champions.get(&key) {
                    files.insert((provider, enemy.champion_id), file);
                } else if !self.champions.is_settled(&key) {
                    pending = true;
                }
            }
            // Show whichever site answers first; spin only while there's nothing yet.
            let any_loaded = Provider::ALL
                .iter()
                .any(|provider| files.contains_key(&(*provider, enemy.champion_id)));
            if error.is_none() && pending && !any_loaded {
                loading.insert(enemy.champion_id);
            }
        }
        Some(build_board(
            picks,
            stats.as_deref(),
            &files,
            &loading,
            self.names.as_deref(),
            error,
            main,
        ))
    }

    fn slug(&self, champion_id: i64) -> Option<String> {
        let id = i32::try_from(champion_id).ok()?;
        let slug = &self.names.as_ref()?.get(&id)?.slug;
        (!slug.is_empty()).then(|| slug.clone())
    }
}

/// One enemy pick's matchup files, from whichever sites have loaded.
#[derive(Clone, Copy, Default)]
struct SourceFiles<'a> {
    lolalytics: Option<&'a ChampionMatchups>,
    ugg: Option<&'a ChampionMatchups>,
}

impl<'a> SourceFiles<'a> {
    fn loaded(self) -> u8 {
        u8::from(self.lolalytics.is_some()) + u8::from(self.ugg.is_some())
    }

    /// Each site's row for `champion_id` in `role`, if it has at least `min_games` of it.
    fn rows(self, role: Role, champion_id: i64, min_games: f64) -> MatchupRows<'a> {
        let find = |file: Option<&'a ChampionMatchups>| {
            file?
                .rows(role)
                .iter()
                .find(|row| row.champion_id == champion_id && row.games >= min_games)
        };
        MatchupRows {
            lolalytics: find(self.lolalytics),
            ugg: find(self.ugg),
            loaded: self.loaded(),
        }
    }
}

/// One matchup's rows from each site, and how many sites loaded at all.
#[derive(Clone, Copy)]
struct MatchupRows<'a> {
    lolalytics: Option<&'a OpponentRow>,
    ugg: Option<&'a OpponentRow>,
    loaded: u8,
}

fn build_board(
    picks: &DraftPicks,
    stats: Option<&PatchStats>,
    files: &HashMap<(Provider, i64), Arc<ChampionMatchups>>,
    loading: &HashSet<i64>,
    names: Option<&HashMap<i32, ChampionInfo>>,
    error: Option<String>,
    main: Provider,
) -> MatchupBoard {
    let roles = stats.map(|stats| assign_roles(&picks.enemies, stats));
    let mut enemies: Vec<EnemyMatchups> = picks
        .enemies
        .iter()
        .enumerate()
        .map(|(index, enemy)| {
            let role = enemy
                .role
                .or_else(|| roles.as_ref().and_then(|roles| roles[index]));
            let record = role
                .zip(stats)
                .and_then(|(role, stats)| stats.record(role, enemy.champion_id));
            let sources = SourceFiles {
                lolalytics: files
                    .get(&(Provider::Lolalytics, enemy.champion_id))
                    .map(Arc::as_ref),
                ugg: files.get(&(Provider::Ugg, enemy.champion_id)).map(Arc::as_ref),
            };
            // Both lists need the role, the patch's records and at least one site's file.
            let ready = match (role, stats) {
                (Some(role), Some(stats)) if sources.loaded() > 0 => Some((role, stats)),
                _ => None,
            };
            let counters = ready
                .map(|(role, stats)| {
                    counters_for(sources, role, enemy.champion_id, stats, names, main)
                })
                .unwrap_or_default();
            let ally = role
                .and_then(|role| picks.allies.iter().find(|ally| ally.role == Some(role)))
                .map(|ally| AllyMatchup {
                    champion: champion_ref(ally.champion_id, names),
                    is_self: ally.is_self,
                    stats: ready.and_then(|(role, stats)| {
                        let rows = sources.rows(role, ally.champion_id, 1.0);
                        combined_stats(rows, main, role, enemy.champion_id, stats)
                    }),
                });
            EnemyMatchups {
                champion: champion_ref(enemy.champion_id, names),
                role,
                role_guessed: enemy.role.is_none() && role.is_some(),
                games: record.map_or(0, |record| record.games.round() as u32),
                win_rate: record.map(|record| record.wins / record.games * 100.0),
                loading: loading.contains(&enemy.champion_id),
                counters,
                ally,
            }
        })
        .collect();
    // Lane order; picks without a role yet stay last, in pick order.
    enemies.sort_by_key(|enemy| (enemy.role.is_none(), enemy.role));

    MatchupBoard {
        provider: main,
        patch: stats.map(|stats| display_patch(&stats.patch)),
        error,
        self_role: picks.self_role,
        allies: picks
            .allies
            .iter()
            .filter_map(|ally| {
                Some(AllyPick {
                    champion: champion_ref(ally.champion_id, names),
                    role: ally.role?,
                    is_self: ally.is_self,
                })
            })
            .collect(),
        enemies,
    }
}

/// The best picks into `enemy_id` in `role`, best first: the ones likeliest to win the game.
///
/// Ranked by each site's matchup win rate, pulled towards what an average pick wins against the
/// enemy by sample size, and averaged over the sites that loaded; a site without enough games of
/// the matchup counts as that average, so a pick both sites back outranks one only one has seen
/// enough of. Everyone is pulled towards the same average: pulling towards each pick's own
/// overall strength let strong champions with few games outrank ones that won more.
fn counters_for(
    sources: SourceFiles,
    role: Role,
    enemy_id: i64,
    stats: &PatchStats,
    names: Option<&HashMap<i32, ChampionInfo>>,
    main: Provider,
) -> Vec<Counter> {
    let enemy_games = stats.record(role, enemy_id).map_or(0.0, |record| record.games);
    let min_games = (enemy_games * COUNTER_GAMES_SHARE)
        .clamp(MIN_SPARSE_COUNTER_GAMES, MIN_COUNTER_GAMES);
    // What an average pick wins against this enemy, in percent.
    let average = stats
        .win_rate(role, enemy_id)
        .map_or(50.0, |enemy| (1.0 - enemy) * 100.0);
    let candidates: BTreeSet<i64> = [sources.lolalytics, sources.ugg]
        .into_iter()
        .flatten()
        .flat_map(|file| file.rows(role))
        .filter(|row| row.games >= min_games && row.champion_id != enemy_id)
        .map(|row| row.champion_id)
        .collect();

    let mut ranked: Vec<(f64, Counter)> = candidates
        .into_iter()
        .filter_map(|champion_id| {
            let rows = sources.rows(role, champion_id, min_games);
            let sides: Vec<SideStats> = [rows.lolalytics, rows.ugg]
                .into_iter()
                .flatten()
                .map(|row| matchup_stats(row, role, enemy_id, stats))
                .collect();
            if sides.is_empty() {
                return None;
            }
            let shrunk = |side: &SideStats| {
                let games = f64::from(side.games);
                (games * side.win_rate + SHRINK_GAMES * average) / (games + SHRINK_GAMES)
            };
            let missing = f64::from(rows.loaded.saturating_sub(sides.len() as u8));
            let score = (sides.iter().map(shrunk).sum::<f64>() + missing * average)
                / f64::from(rows.loaded.max(1));
            let counter = Counter {
                champion: champion_ref(champion_id, names),
                stats: combined_stats(rows, main, role, enemy_id, stats)?,
            };
            Some((score, counter))
        })
        .collect();
    ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
    ranked
        .into_iter()
        .take(COUNTERS_PER_ENEMY)
        .map(|(_, counter)| counter)
        .collect()
}

/// The matchup as both sites see it: the main site's win rate up front, U.GG's leads, and how
/// many sites agree it goes better than expected. None when neither site has it.
fn combined_stats(
    rows: MatchupRows,
    main: Provider,
    role: Role,
    enemy_id: i64,
    stats: &PatchStats,
) -> Option<MatchupStats> {
    let side = |row: Option<&OpponentRow>| row.map(|row| matchup_stats(row, role, enemy_id, stats));
    let (lolalytics, ugg) = (side(rows.lolalytics), side(rows.ugg));
    let shown = match main {
        Provider::Lolalytics => lolalytics.as_ref().or(ugg.as_ref()),
        Provider::Ugg => ugg.as_ref().or(lolalytics.as_ref()),
    }?;
    let source = |side: &Option<SideStats>| {
        side.as_ref().map(|side| SourceStats {
            games: side.games,
            win_rate: side.win_rate,
            win_rate_delta: side.win_rate_delta,
        })
    };
    let leads = ugg.as_ref().and_then(|ugg| ugg.leads);
    let lead = |pick: fn(&PerGameLeads) -> f64| leads.as_ref().map(pick);
    Some(MatchupStats {
        games: shown.games,
        win_rate: shown.win_rate,
        win_rate_delta: shown.win_rate_delta,
        gold_diff_15: lead(|leads| leads.gold),
        xp_diff_15: lead(|leads| leads.xp),
        cs_diff_15: lead(|leads| leads.cs),
        jungle_cs_diff_15: lead(|leads| leads.jungle_cs),
        kill_diff_15: lead(|leads| leads.kills),
        lolalytics: source(&lolalytics),
        ugg: source(&ugg),
        agreement: [&lolalytics, &ugg]
            .into_iter()
            .flatten()
            .filter(|side| edge(side) > 0.0)
            .count() as u8,
        sources: rows.loaded,
    })
}

/// One site's view of a matchup, per game.
#[derive(Clone, Debug, PartialEq)]
struct SideStats {
    games: u32,
    win_rate: f64,
    win_rate_delta: Option<f64>,
    leads: Option<PerGameLeads>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PerGameLeads {
    xp: f64,
    gold: f64,
    cs: f64,
    jungle_cs: f64,
    kills: f64,
}

/// How much better than expected the matchup goes, in win rate points: the Δ, or else the
/// distance from 50% when a champion has no games in the role.
fn edge(side: &SideStats) -> f64 {
    side.win_rate_delta.unwrap_or(side.win_rate - 50.0)
}

/// A matchup from the side of `row`'s champion, which faced `enemy_id` (whose file it's from).
fn matchup_stats(row: &OpponentRow, role: Role, enemy_id: i64, stats: &PatchStats) -> SideStats {
    let per_game = |sum: f64| sum / row.games;
    let win_rate = per_game(row.wins);
    // Even champions split 50/50; each point a champion wins above 50% elsewhere is expected
    // to carry over, and each point its opponent wins takes one away.
    let expected = stats
        .win_rate(role, row.champion_id)
        .zip(stats.win_rate(role, enemy_id))
        .map(|(own, enemy)| 0.5 + own - enemy);
    SideStats {
        games: row.games.round() as u32,
        win_rate: win_rate * 100.0,
        win_rate_delta: row
            .delta
            .or_else(|| expected.map(|expected| (win_rate - expected) * 100.0)),
        leads: row.leads.map(|leads| PerGameLeads {
            xp: per_game(leads.xp),
            gold: per_game(leads.gold),
            cs: per_game(leads.cs),
            jungle_cs: per_game(leads.jungle_cs),
            kills: per_game(leads.kills),
        }),
    }
}

fn champion_ref(champion_id: i64, names: Option<&HashMap<i32, ChampionInfo>>) -> ChampionRef {
    let info = i32::try_from(champion_id)
        .ok()
        .and_then(|id| names?.get(&id));
    ChampionRef {
        id: champion_id,
        name: info.map_or_else(
            || format!("Champion {champion_id}"),
            |info| info.name.clone(),
        ),
        slug: info.map(|info| info.slug.clone()).unwrap_or_default(),
    }
}

/// Each enemy pick's role: the one Riot shows, or else the free role that best fits what the
/// unplaced picks are usually played as, so two picks never share a role.
fn assign_roles(enemies: &[Pick], stats: &PatchStats) -> Vec<Option<Role>> {
    let mut roles: Vec<Option<Role>> = enemies.iter().map(|enemy| enemy.role).collect();
    let free: Vec<Role> = Role::ALL
        .into_iter()
        .filter(|role| !roles.contains(&Some(*role)))
        .collect();
    let unplaced: Vec<usize> = (0..enemies.len())
        .filter(|&index| roles[index].is_none())
        .collect();

    let mut best: Option<(f64, Vec<Role>)> = None;
    for order in permutations(&free) {
        let score: f64 = unplaced
            .iter()
            .zip(&order)
            .map(|(&index, &role)| stats.role_share(enemies[index].champion_id, role).ln())
            .sum();
        if best.as_ref().map_or(true, |(best_score, _)| score > *best_score) {
            best = Some((score, order));
        }
    }
    if let Some((_, order)) = best {
        for (&index, role) in unplaced.iter().zip(order) {
            roles[index] = Some(role);
        }
    }
    roles
}

fn permutations(roles: &[Role]) -> Vec<Vec<Role>> {
    if roles.is_empty() {
        return vec![Vec::new()];
    }
    let mut all = Vec::new();
    for (index, &first) in roles.iter().enumerate() {
        let mut rest = roles.to_vec();
        rest.remove(index);
        for mut order in permutations(&rest) {
            order.insert(0, first);
            all.push(order);
        }
    }
    all
}

/// U.GG names patches by game version ("16_21"); Riot has named them by year ("26.21") since
/// 2025's 15.x.
fn display_patch(patch: &str) -> String {
    match patch.split_once('_') {
        Some((major, minor)) => match major.parse::<u32>() {
            Ok(number) if number >= 15 => format!("{}.{minor}", number + 10),
            _ => format!("{major}.{minor}"),
        },
        None => patch.to_string(),
    }
}

/// One champion's games in one role.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct RoleRecord {
    wins: f64,
    games: f64,
}

/// The patch Reveal reads and each champion's World Master+ record per role.
#[derive(Debug)]
pub struct PatchStats {
    /// U.GG's patch key, e.g. "16_21".
    patch: String,
    matchups_version: String,
    records: HashMap<Role, HashMap<i64, RoleRecord>>,
}

impl PatchStats {
    fn record(&self, role: Role, champion_id: i64) -> Option<RoleRecord> {
        self.records
            .get(&role)?
            .get(&champion_id)
            .copied()
            .filter(|record| record.games > 0.0)
    }

    fn win_rate(&self, role: Role, champion_id: i64) -> Option<f64> {
        self.record(role, champion_id)
            .map(|record| record.wins / record.games)
    }

    /// How much of the champion's play is in the role, smoothed so unplayed roles stay possible.
    fn role_share(&self, champion_id: i64, role: Role) -> f64 {
        let games = |role: Role| {
            self.record(role, champion_id)
                .map_or(0.0, |record| record.games)
        };
        let total: f64 = Role::ALL.into_iter().map(|role| games(role)).sum();
        (games(role) + 1.0) / (total + Role::ALL.len() as f64)
    }
}

/// One opponent's games against a champion in one role, from the opponent's side.
#[derive(Clone, Debug, PartialEq)]
struct OpponentRow {
    champion_id: i64,
    wins: f64,
    games: f64,
    /// U.GG's leads at 15 minutes, summed over the games; Lolalytics has none.
    leads: Option<Leads>,
    /// The provider's own normalized win rate edge in points (Lolalytics' Δ2).
    delta: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Leads {
    xp: f64,
    gold: f64,
    cs: f64,
    jungle_cs: f64,
    kills: f64,
}

/// A champion's World Master+ matchups, by role.
#[derive(Debug, Default)]
pub struct ChampionMatchups {
    rows: HashMap<Role, Vec<OpponentRow>>,
}

impl ChampionMatchups {
    fn rows(&self, role: Role) -> &[OpponentRow] {
        self.rows
            .get(&role)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

/// The newest patch with enough Master+ games, with every champion's record per role.
pub async fn load_patch_stats() -> anyhow::Result<Arc<PatchStats>> {
    if let Some(stats) = cached_patch_stats() {
        return Ok(stats);
    }

    let patches: Vec<String> = serde_json::from_value(fetch_json(PATCHES_URL).await?)?;
    // Only names the file versions, which have been 1.5.0 for years; not worth failing over.
    let versions = fetch_json(API_VERSIONS_URL).await.unwrap_or_else(|error| {
        log_warn!("U.GG file versions are unavailable; assuming {DEFAULT_API_VERSION}: {error}");
        Value::Null
    });

    let mut chosen: Option<(PatchStats, f64)> = None;
    for patch in patches.iter().take(2) {
        let version = api_version(&versions, patch, "champion_ranking");
        let url = format!(
            "{STATS_BASE}/champion_ranking/world/{patch}/ranked_solo_5x5/master_plus/{version}.json"
        );
        let (records, games) = match fetch_json(&url)
            .await
            .and_then(|ranking| parse_ranking(&ranking))
        {
            Ok(parsed) => parsed,
            Err(error) => {
                log_warn!("U.GG has no Master+ ranking for patch {patch}: {error}");
                continue;
            }
        };
        let stats = PatchStats {
            patch: patch.clone(),
            matchups_version: api_version(&versions, patch, "matchups"),
            records,
        };
        if games >= MIN_PATCH_GAMES {
            chosen = Some((stats, games));
            break;
        }
        if chosen.as_ref().map_or(true, |(_, most)| games > *most) {
            chosen = Some((stats, games));
        }
    }

    let (stats, games) = chosen.context("U.GG has no Master+ stats for the latest patches")?;
    log_info!(
        "Using U.GG World Master+ stats for patch {} ({games} games)",
        stats.patch
    );
    let stats = Arc::new(stats);
    *lock(&PATCH_STATS) = Some(Cached::new(stats.clone()));
    Ok(stats)
}

/// One champion's World Master+ matchups on the patch Reveal reads.
pub async fn load_champion(
    stats: Arc<PatchStats>,
    champion_id: i64,
) -> anyhow::Result<Arc<ChampionMatchups>> {
    let key = (stats.patch.clone(), champion_id);
    if let Some(matchups) = cached_champion(&key) {
        return Ok(matchups);
    }

    let url = format!(
        "{STATS_BASE}/matchups/{}/ranked_solo_5x5/{champion_id}/{}.json",
        stats.patch, stats.matchups_version
    );
    let matchups = Arc::new(parse_matchups(&fetch_json(&url).await?));
    lock(&CHAMPION_MATCHUPS)
        .get_or_insert_with(HashMap::new)
        .insert(key, Cached::new(matchups.clone()));
    Ok(matchups)
}

/// One champion's Master+ matchups in one lane from Lolalytics, on its current patch.
pub async fn load_lolalytics(
    slug: String,
    champion_id: i64,
    role: Role,
) -> anyhow::Result<Arc<ChampionMatchups>> {
    let key = (format!("lolalytics:{}", role.lolalytics_lane()), champion_id);
    if let Some(matchups) = cached_champion(&key) {
        return Ok(matchups);
    }

    let url = format!(
        "{LOLALYTICS_COUNTERS_URL}&c={slug}&lane={lane}&vslane={lane}&tier=master_plus&queue=ranked&region=all",
        lane = role.lolalytics_lane()
    );
    let matchups = Arc::new(parse_lolalytics(&fetch_json(&url).await?, role)?);
    lock(&CHAMPION_MATCHUPS)
        .get_or_insert_with(HashMap::new)
        .insert(key, Cached::new(matchups.clone()));
    Ok(matchups)
}

/// The file version U.GG lists for this patch and kind of file.
fn api_version(versions: &Value, patch: &str, kind: &str) -> String {
    versions[patch][kind]
        .as_str()
        .unwrap_or(DEFAULT_API_VERSION)
        .to_string()
}

/// `[{role: [[champion, worst matchups, wins, games, ...], ...]}, bans, updated, games]`.
fn parse_ranking(value: &Value) -> anyhow::Result<(HashMap<Role, HashMap<i64, RoleRecord>>, f64)> {
    let roles = value
        .get(0)
        .and_then(Value::as_object)
        .context("U.GG's champion ranking has no roles")?;
    let records = Role::ALL
        .into_iter()
        .map(|role| {
            let champions: HashMap<i64, RoleRecord> = roles
                .get(role.ranking_key())
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|row| {
                    let champion_id: i64 = row.get(0)?.as_str()?.parse().ok()?;
                    let wins = row.get(2)?.as_f64()?;
                    let games = row.get(3)?.as_f64()?;
                    Some((champion_id, RoleRecord { wins, games }))
                })
                .collect();
            (role, champions)
        })
        .collect();
    let games = value.get(3).and_then(Value::as_f64).unwrap_or_default();
    Ok((records, games))
}

/// `{region: {tier: {role: [[rows...], updated]}}}`; only World Master+ is kept.
fn parse_matchups(value: &Value) -> ChampionMatchups {
    let tier = &value[WORLD][MASTER_PLUS];
    let rows = Role::ALL
        .into_iter()
        .map(|role| {
            let opponents: Vec<OpponentRow> = tier[role.ugg_id()][0]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(parse_row)
                .collect();
            (role, opponents)
        })
        .collect();
    ChampionMatchups { rows }
}

/// `[opponent, its wins, games, xp@15, gold@15, duo gold@15, cs@15, duo cs@15, jungle cs@15,
/// kills@15, ...]`, each lead being the opponent's over the champion the file is about.
fn parse_row(row: &Value) -> Option<OpponentRow> {
    let field = |index: usize| row.get(index).and_then(Value::as_f64);
    let games = field(2).filter(|games| *games > 0.0)?;
    Some(OpponentRow {
        champion_id: field(0)? as i64,
        wins: field(1)?,
        games,
        leads: Some(Leads {
            xp: field(3)?,
            gold: field(4)?,
            cs: field(6)?,
            jungle_cs: field(8)?,
            kills: field(9)?,
        }),
        delta: None,
    })
}

/// `{"counters": [{"cid", "vsWr", "n", "d1", "d2", ...}]}`, from the side of the champion asked
/// about: its win rate against each opponent and Δ2, its normalized edge.
fn parse_lolalytics(value: &Value, role: Role) -> anyhow::Result<ChampionMatchups> {
    let counters = value["counters"]
        .as_array()
        .context("Lolalytics sent no counters")?;
    let opponents = counters
        .iter()
        .filter_map(|counter| {
            let field = |name: &str| counter[name].as_f64();
            let games = field("n").filter(|games| *games > 0.0)?;
            Some(OpponentRow {
                champion_id: field("cid")? as i64,
                wins: (100.0 - field("vsWr")?) / 100.0 * games,
                games,
                leads: None,
                delta: field("d2").map(|delta| -delta),
            })
        })
        .collect();
    Ok(ChampionMatchups {
        rows: HashMap::from([(role, opponents)]),
    })
}

fn http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_default()
    })
}

async fn fetch_json(url: &str) -> anyhow::Result<Value> {
    Ok(http()
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

struct Cached<T> {
    fetched_at: Instant,
    value: Arc<T>,
}

impl<T> Cached<T> {
    fn new(value: Arc<T>) -> Self {
        Self {
            fetched_at: Instant::now(),
            value,
        }
    }

    fn fresh(&self) -> Option<Arc<T>> {
        (self.fetched_at.elapsed() < STATS_MAX_AGE).then(|| self.value.clone())
    }
}

/// Kept for the whole run, so the next Champ Select doesn't download them again.
static PATCH_STATS: Mutex<Option<Cached<PatchStats>>> = Mutex::new(None);
static CHAMPION_MATCHUPS: Mutex<Option<HashMap<(String, i64), Cached<ChampionMatchups>>>> =
    Mutex::new(None);

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn cached_patch_stats() -> Option<Arc<PatchStats>> {
    lock(&PATCH_STATS).as_ref().and_then(Cached::fresh)
}

fn cached_champion(key: &(String, i64)) -> Option<Arc<ChampionMatchups>> {
    lock(&CHAMPION_MATCHUPS)
        .as_ref()
        .and_then(|cache| cache.get(key))
        .and_then(Cached::fresh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const LEE_SIN: i64 = 64;
    const REK_SAI: i64 = 421;
    const ELISE: i64 = 60;
    const KINDRED: i64 = 203;
    const QIYANA: i64 = 246;
    const GRAVES: i64 = 104;
    const DARIUS: i64 = 122;
    const AHRI: i64 = 103;

    fn record(wins: f64, games: f64) -> RoleRecord {
        RoleRecord { wins, games }
    }

    fn patch_stats(records: &[(Role, i64, RoleRecord)]) -> PatchStats {
        let mut stats = PatchStats {
            patch: "16_21".to_string(),
            matchups_version: DEFAULT_API_VERSION.to_string(),
            records: HashMap::new(),
        };
        for &(role, champion_id, record) in records {
            stats
                .records
                .entry(role)
                .or_default()
                .insert(champion_id, record);
        }
        stats
    }

    fn row(champion_id: i64, wins: f64, games: f64) -> OpponentRow {
        OpponentRow {
            champion_id,
            wins,
            games,
            leads: None,
            delta: None,
        }
    }

    const REK_SAI_LEADS: Leads = Leads {
        xp: -678.0,
        gold: 52642.0,
        cs: 152.0,
        jungle_cs: -1000.0,
        kills: 90.0,
    };

    fn names() -> HashMap<i32, ChampionInfo> {
        [
            (LEE_SIN, "Lee Sin", "leesin"),
            (REK_SAI, "Rek'Sai", "reksai"),
            (QIYANA, "Qiyana", "qiyana"),
        ]
        .into_iter()
        .map(|(id, name, slug)| {
            (
                id as i32,
                ChampionInfo {
                    name: name.to_string(),
                    slug: slug.to_string(),
                },
            )
        })
        .collect()
    }

    fn ugg_only(file: &ChampionMatchups) -> SourceFiles<'_> {
        SourceFiles {
            lolalytics: None,
            ugg: Some(file),
        }
    }

    fn with_delta(champion_id: i64, wins: f64, games: f64, delta: f64) -> OpponentRow {
        OpponentRow {
            delta: Some(delta),
            ..row(champion_id, wins, games)
        }
    }

    #[test]
    fn counters_both_sites_back_outrank_ones_only_one_site_has() {
        let stats = patch_stats(&[(Role::Jungle, LEE_SIN, record(24_000.0, 48_000.0))]);
        let ugg = ChampionMatchups {
            rows: HashMap::from([(
                Role::Jungle,
                vec![
                    OpponentRow {
                        leads: Some(REK_SAI_LEADS),
                        ..with_delta(REK_SAI, 340.0, 600.0, 4.0)
                    },
                    with_delta(ELISE, 560.0, 1_000.0, 6.0),
                    with_delta(KINDRED, 520.0, 1_000.0, -2.0),
                ],
            )]),
        };
        let lolalytics = ChampionMatchups {
            rows: HashMap::from([(
                Role::Jungle,
                vec![
                    with_delta(REK_SAI, 380.0, 700.0, 4.0),
                    // Elise has too few games here, so only U.GG backs her.
                    with_delta(ELISE, 100.0, 150.0, 9.0),
                    with_delta(KINDRED, 550.0, 1_000.0, 3.0),
                ],
            )]),
        };
        let sources = SourceFiles {
            lolalytics: Some(&lolalytics),
            ugg: Some(&ugg),
        };

        let counters = counters_for(sources, Role::Jungle, LEE_SIN, &stats, None, Provider::Lolalytics);

        let order: Vec<i64> = counters.iter().map(|counter| counter.champion.id).collect();
        // Kindred wins more on both sites than Elise does on U.GG alone.
        assert_eq!(order, vec![REK_SAI, KINDRED, ELISE]);
        let rek_sai = &counters[0].stats;
        assert_eq!((rek_sai.agreement, rek_sai.sources), (2, 2));
        assert_eq!(rek_sai.games, 700, "the main site's numbers come first");
        assert_eq!(rek_sai.ugg.as_ref().map(|ugg| ugg.games), Some(600));
        assert!(rek_sai.gold_diff_15.is_some(), "leads always come from U.GG");
        let elise = &counters[2].stats;
        assert_eq!((elise.agreement, elise.lolalytics.as_ref()), (1, None));
        assert_eq!(elise.games, 1_000, "U.GG's numbers when Lolalytics lacks the matchup");
        assert_eq!(counters[1].stats.agreement, 1, "the sites disagree on Kindred");
    }

    #[test]
    fn a_strong_pick_that_wins_the_matchup_beats_a_weak_one_doing_a_little_better() {
        // Like Kindred into Jarvan IV: strong everywhere, so only a small Δ, but the higher win rate.
        let stats = patch_stats(&[
            (Role::Jungle, LEE_SIN, record(500.0, 1_000.0)),
            (Role::Jungle, KINDRED, record(540.0, 1_000.0)),
            (Role::Jungle, ELISE, record(460.0, 1_000.0)),
        ]);
        let file = ChampionMatchups {
            rows: HashMap::from([(
                Role::Jungle,
                vec![row(KINDRED, 550.0, 1_000.0), row(ELISE, 500.0, 1_000.0)],
            )]),
        };

        let counters = counters_for(ugg_only(&file), Role::Jungle, LEE_SIN, &stats, None, Provider::Ugg);

        let order: Vec<i64> = counters.iter().map(|counter| counter.champion.id).collect();
        assert_eq!(order, vec![KINDRED, ELISE]);
        assert!(counters[0].stats.win_rate_delta < counters[1].stats.win_rate_delta);
    }

    #[test]
    fn a_strong_champion_with_few_games_doesnt_outrank_one_that_won_more() {
        // Like Nunu and Kindred into Hecarim: Nunu is strong overall but won less here.
        let stats = patch_stats(&[
            (Role::Jungle, LEE_SIN, record(510.0, 1_000.0)),
            (Role::Jungle, KINDRED, record(500.0, 1_000.0)),
            (Role::Jungle, ELISE, record(560.0, 1_000.0)),
        ]);
        let file = ChampionMatchups {
            rows: HashMap::from([(
                Role::Jungle,
                vec![row(KINDRED, 277.0, 543.0), row(ELISE, 187.0, 379.0)],
            )]),
        };

        let counters = counters_for(ugg_only(&file), Role::Jungle, LEE_SIN, &stats, None, Provider::Ugg);

        let order: Vec<i64> = counters.iter().map(|counter| counter.champion.id).collect();
        assert_eq!(order, vec![KINDRED, ELISE]);
    }

    #[test]
    fn keeps_only_world_master_plus_rows_from_a_matchup_file() {
        let file = json!({
            "1": { "14": { "1": [[[ELISE, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]], "t"] } },
            "12": {
                "11": { "1": [[[ELISE, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]], "t"] },
                "14": {
                    "1": [[
                        [REK_SAI, 364, 644, -678, 52642, 0, 152, 0, -1000, 90, 0, 0, 5960, 0, 360547],
                        [ELISE, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
                    ], "2026-10-31T06:12:47Z"]
                }
            }
        });

        let matchups = parse_matchups(&file);

        assert_eq!(
            matchups.rows(Role::Jungle),
            [OpponentRow {
                champion_id: REK_SAI,
                wins: 364.0,
                games: 644.0,
                leads: Some(REK_SAI_LEADS),
                delta: None,
            }],
            "rows without games are dropped"
        );
        assert!(matchups.rows(Role::Top).is_empty());
    }

    #[test]
    fn reads_each_champions_record_per_role_from_the_ranking() {
        let ranking = json!([
            {
                "jungle": [["64", [], 23294, 48152, 1, 2, 3, 4, 5, 6], ["421", [], 10, 0]],
                "supp": [["64", [], 300, 700, 1, 2, 3, 4, 5, 6]]
            },
            {},
            "2026-10-31T05:59:39Z",
            338085.0
        ]);

        let (records, games) = parse_ranking(&ranking).unwrap();
        let stats = PatchStats {
            patch: "16_21".to_string(),
            matchups_version: DEFAULT_API_VERSION.to_string(),
            records,
        };

        assert_eq!(games, 338085.0);
        assert_eq!(
            stats.record(Role::Jungle, LEE_SIN),
            Some(record(23294.0, 48152.0))
        );
        assert_eq!(stats.record(Role::Utility, LEE_SIN), Some(record(300.0, 700.0)));
        assert_eq!(stats.record(Role::Jungle, REK_SAI), None, "no games");
        assert_eq!(stats.record(Role::Top, LEE_SIN), None);
        assert!(parse_ranking(&json!({})).is_err());
    }

    #[test]
    fn matchup_stats_are_per_game_from_the_opponents_side() {
        let stats = patch_stats(&[
            (Role::Jungle, LEE_SIN, record(48.0, 100.0)),
            (Role::Jungle, REK_SAI, record(51.0, 100.0)),
        ]);
        let row = OpponentRow {
            champion_id: REK_SAI,
            wins: 364.0,
            games: 644.0,
            leads: Some(REK_SAI_LEADS),
            delta: None,
        };

        let matchup = matchup_stats(&row, Role::Jungle, LEE_SIN, &stats);

        assert_eq!(matchup.games, 644);
        assert!((matchup.win_rate - 56.52).abs() < 0.01);
        // Rek'Sai (51%) against Lee Sin (48%) is expected to win 53%.
        assert!((matchup.win_rate_delta.unwrap() - 3.52).abs() < 0.01);
        let leads = matchup.leads.unwrap();
        assert!((leads.gold - 81.74).abs() < 0.01);
        assert!((leads.kills - 0.14).abs() < 0.01);
        assert!((leads.jungle_cs + 1.55).abs() < 0.01);

        let unranked = patch_stats(&[(Role::Jungle, LEE_SIN, record(48.0, 100.0))]);
        assert_eq!(
            matchup_stats(&row, Role::Jungle, LEE_SIN, &unranked).win_rate_delta,
            None
        );
    }

    #[test]
    fn ranks_counters_by_their_edge_weighed_by_games() {
        let stats = patch_stats(&[
            (Role::Jungle, LEE_SIN, record(24_000.0, 48_000.0)),
            (Role::Jungle, REK_SAI, record(500.0, 1_000.0)),
            (Role::Jungle, ELISE, record(500.0, 1_000.0)),
            (Role::Jungle, KINDRED, record(500.0, 1_000.0)),
        ]);
        let file = ChampionMatchups {
            rows: HashMap::from([(
                Role::Jungle,
                vec![
                    // Huge edge, but under the 200 games a popular champion's counters need.
                    row(KINDRED, 120.0, 150.0),
                    // +6 points over 250 games ranks below +4 over 3,000.
                    row(REK_SAI, 140.0, 250.0),
                    row(ELISE, 1_620.0, 3_000.0),
                    row(LEE_SIN, 500.0, 1_000.0),
                ],
            )]),
        };

        let counters = counters_for(ugg_only(&file), Role::Jungle, LEE_SIN, &stats, None, Provider::Ugg);

        let order: Vec<i64> = counters.iter().map(|counter| counter.champion.id).collect();
        assert_eq!(order, vec![ELISE, REK_SAI], "thin and mirror matchups are left out");
        assert_eq!(counters[1].champion.name, "Champion 421");
    }

    #[test]
    fn rarely_played_picks_still_get_counters_from_fewer_games() {
        let stats = patch_stats(&[(Role::Top, DARIUS, record(1_400.0, 2_800.0))]);
        let file = ChampionMatchups {
            rows: HashMap::from([(
                Role::Top,
                vec![row(GRAVES, 50.0, 80.0), row(AHRI, 20.0, 40.0)],
            )]),
        };

        let counters = counters_for(ugg_only(&file), Role::Top, DARIUS, &stats, None, Provider::Ugg);

        // 2.5% of 2,800 games is 70, so 80 games is enough but 40 is still too few.
        assert_eq!(counters.len(), 1);
        assert_eq!(counters[0].champion.id, GRAVES);
        assert_eq!(counters[0].stats.win_rate_delta, None, "Graves has no top games");
        assert!((counters[0].stats.win_rate - 62.5).abs() < 0.01);
    }

    #[test]
    fn guesses_hidden_enemy_roles_from_what_each_pick_plays() {
        let stats = patch_stats(&[
            (Role::Jungle, GRAVES, record(450.0, 900.0)),
            (Role::Top, GRAVES, record(50.0, 100.0)),
            (Role::Top, DARIUS, record(500.0, 1_000.0)),
            (Role::Jungle, DARIUS, record(5.0, 10.0)),
            (Role::Middle, AHRI, record(500.0, 1_000.0)),
        ]);
        let pick = |champion_id, role| Pick {
            champion_id,
            role,
            is_self: false,
        };

        assert_eq!(
            assign_roles(
                &[pick(GRAVES, None), pick(DARIUS, None), pick(AHRI, None)],
                &stats
            ),
            vec![Some(Role::Jungle), Some(Role::Top), Some(Role::Middle)]
        );
        // A role Riot shows wins, and nobody else gets it.
        assert_eq!(
            assign_roles(&[pick(GRAVES, Some(Role::Top)), pick(DARIUS, None)], &stats),
            vec![Some(Role::Top), Some(Role::Jungle)]
        );
    }

    #[test]
    fn board_lists_counters_and_your_matchup_for_each_enemy_in_lane_order() {
        let stats = patch_stats(&[
            (Role::Jungle, LEE_SIN, record(480.0, 1_000.0)),
            (Role::Jungle, REK_SAI, record(510.0, 1_000.0)),
            (Role::Jungle, QIYANA, record(500.0, 1_000.0)),
            (Role::Middle, AHRI, record(500.0, 1_000.0)),
        ]);
        let lee_sin = ChampionMatchups {
            rows: HashMap::from([(
                Role::Jungle,
                vec![row(REK_SAI, 364.0, 644.0), row(QIYANA, 1_325.0, 2_655.0)],
            )]),
        };
        let picks = DraftPicks {
            has_roles: true,
            self_role: Some(Role::Jungle),
            allies: vec![Pick {
                champion_id: QIYANA,
                role: Some(Role::Jungle),
                is_self: true,
            }],
            enemies: vec![
                Pick {
                    champion_id: AHRI,
                    role: None,
                    is_self: false,
                },
                Pick {
                    champion_id: LEE_SIN,
                    role: None,
                    is_self: false,
                },
            ],
        };

        let board = build_board(
            &picks,
            Some(&stats),
            &HashMap::from([((Provider::Ugg, LEE_SIN), Arc::new(lee_sin))]),
            &HashSet::from([AHRI]),
            Some(&names()),
            None,
            Provider::Lolalytics,
        );

        assert_eq!(board.patch.as_deref(), Some("26.21"));
        assert_eq!(board.self_role, Some(Role::Jungle));
        assert_eq!(
            board.allies,
            vec![AllyPick {
                champion: ChampionRef {
                    id: QIYANA,
                    name: "Qiyana".to_string(),
                    slug: "qiyana".to_string(),
                },
                role: Role::Jungle,
                is_self: true,
            }]
        );
        let [jungle, middle] = board.enemies.as_slice() else {
            panic!("expected two enemies, got {:?}", board.enemies);
        };
        assert_eq!(
            (jungle.champion.slug.as_str(), jungle.role, jungle.role_guessed),
            ("leesin", Some(Role::Jungle), true)
        );
        assert_eq!((jungle.games, jungle.loading), (1_000, false));
        assert_eq!(jungle.counters[0].champion.name, "Rek'Sai");
        let ally = jungle.ally.as_ref().unwrap();
        assert!(ally.is_self);
        assert_eq!(ally.champion.name, "Qiyana");
        assert_eq!(ally.stats.as_ref().unwrap().games, 2_655);

        assert_eq!((middle.role, middle.loading), (Some(Role::Middle), true));
        assert!(middle.counters.is_empty());
        assert_eq!(middle.ally, None);
    }

    #[test]
    fn board_shows_enemies_without_roles_until_the_stats_load() {
        let picks = DraftPicks {
            has_roles: true,
            enemies: vec![Pick {
                champion_id: LEE_SIN,
                role: None,
                is_self: false,
            }],
            ..Default::default()
        };

        let board = build_board(
            &picks,
            None,
            &HashMap::new(),
            &HashSet::from([LEE_SIN]),
            None,
            None,
            Provider::Lolalytics,
        );

        assert_eq!(board.patch, None);
        assert_eq!(board.enemies[0].role, None);
        assert!(!board.enemies[0].role_guessed);
        assert!(board.enemies[0].loading);
        assert_eq!(board.enemies[0].champion.name, "Champion 64");
    }

    #[test]
    fn no_board_without_lanes_or_enemy_picks() {
        let lookups = MatchupLookups::default();
        let enemy = Pick {
            champion_id: LEE_SIN,
            role: None,
            is_self: false,
        };
        let aram = DraftPicks {
            has_roles: false,
            enemies: vec![enemy],
            ..Default::default()
        };
        let waiting = DraftPicks {
            has_roles: true,
            ..Default::default()
        };

        assert_eq!(lookups.board(&aram, Provider::Lolalytics), None);
        assert_eq!(lookups.board(&waiting, Provider::Lolalytics), None);
    }

    #[test]
    fn a_dragged_pick_takes_its_lane_and_the_old_one_is_guessed_again() {
        let enemy = |champion_id, role| Pick {
            champion_id,
            role,
            is_self: false,
        };
        let mut picks = DraftPicks {
            has_roles: true,
            enemies: vec![enemy(GRAVES, None), enemy(DARIUS, Some(Role::Jungle))],
            ..Default::default()
        };
        clear_role_overrides();
        set_role_override(DARIUS, Role::Top);
        set_role_override(GRAVES, Role::Top);

        apply_role_overrides(&mut picks);

        assert_eq!(picks.enemies[0].role, Some(Role::Top), "the latest drop wins the lane");
        assert_eq!(picks.enemies[1].role, Some(Role::Jungle), "Darius keeps what Riot showed");
        clear_role_overrides();
    }

    #[test]
    fn names_patches_like_riot_does() {
        assert_eq!(display_patch("16_21"), "26.21");
        assert_eq!(display_patch("15_1"), "25.1");
        assert_eq!(display_patch("14_23"), "14.23");
        assert_eq!(display_patch("latest"), "latest");
    }

    #[test]
    fn reads_league_positions() {
        assert_eq!(Role::from_position("utility"), Some(Role::Utility));
        assert_eq!(Role::from_position("MIDDLE"), Some(Role::Middle));
        assert_eq!(Role::from_position(""), None);
        assert_eq!(
            serde_json::to_value(Role::Utility).unwrap(),
            json!("utility")
        );
    }

    /// Reads today's real files: `cargo test live_ugg -- --ignored --nocapture`.
    #[test]
    #[ignore = "downloads U.GG's live stats"]
    fn live_ugg_stats_give_counters() {
        tauri::async_runtime::block_on(async {
            let stats = load_patch_stats().await.expect("patch stats");
            let record = stats.record(Role::Jungle, LEE_SIN).expect("Lee Sin jungle record");
            println!(
                "Patch {}: Lee Sin jungle {:.2}% in {} games",
                display_patch(&stats.patch),
                record.wins / record.games * 100.0,
                record.games
            );
            assert!(record.games > 1_000.0);

            let lee_sin = load_champion(stats.clone(), LEE_SIN)
                .await
                .expect("Lee Sin matchups");
            let counters =
                counters_for(ugg_only(&lee_sin), Role::Jungle, LEE_SIN, &stats, None, Provider::Ugg);
            for counter in &counters {
                println!(
                    "  {} {:.2}% (delta {:+.2}) in {} games, kills {:+.2}, CS {:+.1}, gold {:+.0}",
                    counter.champion.id,
                    counter.stats.win_rate,
                    counter.stats.win_rate_delta.unwrap_or_default(),
                    counter.stats.games,
                    counter.stats.kill_diff_15.unwrap_or_default(),
                    counter.stats.cs_diff_15.unwrap_or_default()
                        + counter.stats.jungle_cs_diff_15.unwrap_or_default(),
                    counter.stats.gold_diff_15.unwrap_or_default()
                );
            }
            assert_eq!(counters.len(), COUNTERS_PER_ENEMY);
            assert!(counters.iter().all(|counter| counter.stats.games >= 200));
        });
    }

    /// Weighs both sites' real stats: `cargo test live_both -- --ignored --nocapture`.
    #[test]
    #[ignore = "downloads U.GG's and Lolalytics' live stats"]
    fn live_both_sites_rank_counters_together() {
        tauri::async_runtime::block_on(async {
            let stats = load_patch_stats().await.expect("patch stats");
            let ugg = load_champion(stats.clone(), LEE_SIN).await.expect("U.GG");
            let lolalytics = load_lolalytics("leesin".to_string(), LEE_SIN, Role::Jungle)
                .await
                .expect("Lolalytics");
            let sources = SourceFiles {
                lolalytics: Some(&lolalytics),
                ugg: Some(&ugg),
            };
            let counters =
                counters_for(sources, Role::Jungle, LEE_SIN, &stats, None, Provider::Lolalytics);
            for counter in &counters {
                let site = |side: &Option<SourceStats>| {
                    side.as_ref().map_or("-".to_string(), |side| {
                        format!(
                            "{:.1}% d{:+.1} n{}",
                            side.win_rate,
                            side.win_rate_delta.unwrap_or_default(),
                            side.games
                        )
                    })
                };
                println!(
                    "  {:>4}  agree {}/{}  lola {:<22} ugg {}",
                    counter.champion.id,
                    counter.stats.agreement,
                    counter.stats.sources,
                    site(&counter.stats.lolalytics),
                    site(&counter.stats.ugg)
                );
            }
            assert_eq!(counters.len(), COUNTERS_PER_ENEMY);
        });
    }

    /// Reads Lolalytics' real API: `cargo test live_lolalytics -- --ignored --nocapture`.
    #[test]
    #[ignore = "downloads Lolalytics' live stats"]
    fn live_lolalytics_gives_counters() {
        tauri::async_runtime::block_on(async {
            let lee_sin = load_lolalytics("leesin".to_string(), LEE_SIN, Role::Jungle)
                .await
                .expect("Lee Sin matchups");
            let sources = SourceFiles {
                lolalytics: Some(&lee_sin),
                ugg: None,
            };
            let counters = counters_for(
                sources,
                Role::Jungle,
                LEE_SIN,
                &patch_stats(&[]),
                None,
                Provider::Lolalytics,
            );
            for counter in &counters {
                println!(
                    "  {} {:.2}% (delta {:+.2}) in {} games",
                    counter.champion.id,
                    counter.stats.win_rate,
                    counter.stats.win_rate_delta.unwrap_or_default(),
                    counter.stats.games
                );
            }
            assert!(lee_sin.rows(Role::Jungle).len() > 20);
            assert_eq!(counters.len(), COUNTERS_PER_ENEMY);
        });
    }

    #[test]
    fn reads_lolalytics_counters_from_the_counter_picks_side() {
        let response = json!({
            "stats": { "cid": 64, "lane": "jungle" },
            "counters": [
                { "cid": 421, "vsWr": 46.64, "n": 714, "d1": -0.12, "d2": -3.07 },
                { "cid": 60, "vsWr": 50.0, "n": 0, "d2": 0.0 }
            ]
        });

        let matchups = parse_lolalytics(&response, Role::Jungle).unwrap();
        let [rek_sai] = matchups.rows(Role::Jungle) else {
            panic!("rows without games are dropped");
        };
        let stats = matchup_stats(rek_sai, Role::Jungle, LEE_SIN, &patch_stats(&[]));

        assert_eq!(stats.games, 714);
        assert!((stats.win_rate - 53.36).abs() < 0.01, "Lee Sin won 46.64%");
        assert_eq!(stats.win_rate_delta, Some(3.07), "Lolalytics' Δ2, flipped");
        assert_eq!(stats.leads, None);
        assert!(matchups.rows(Role::Top).is_empty());
        assert!(parse_lolalytics(&json!({}), Role::Jungle).is_err());
    }

    #[test]
    fn providers_split_their_downloads_like_their_sites_do() {
        assert_eq!(Provider::Ugg.file_key(64, Role::Jungle), "ugg:64");
        assert_eq!(Provider::Ugg.file_key(64, Role::Top), "ugg:64");
        assert_eq!(
            Provider::Lolalytics.file_key(64, Role::Utility),
            "lolalytics:64:support"
        );
        assert_eq!(Provider::from_setting("ugg"), Provider::Ugg);
        assert_eq!(Provider::from_setting("leagueofgraphs"), Provider::Lolalytics);
        assert_eq!(Provider::default(), Provider::Lolalytics);
        assert_eq!(serde_json::to_value(Provider::Ugg).unwrap(), json!("ugg"));
    }

    #[test]
    fn uses_the_listed_file_versions() {
        let versions = json!({ "16_21": { "matchups": "1.6.0" } });
        assert_eq!(api_version(&versions, "16_21", "matchups"), "1.6.0");
        assert_eq!(api_version(&versions, "16_21", "champion_ranking"), "1.5.0");
        assert_eq!(api_version(&Value::Null, "16_21", "matchups"), "1.5.0");
    }
}
