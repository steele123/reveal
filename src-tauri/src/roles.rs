//! Guessing which hidden Champ Select slot belongs to which teammate.
//!
//! In ranked, Riot blanks each slot's puuid, so chat names can't be linked to roles directly.
//! Each teammate's recent match history usually gives them away: the positions they play, the
//! summoner spells they take (players keep the same spells *and* key order), and the champions
//! they play. Every slot is scored against every unplaced teammate and the best overall
//! assignment wins, so two teammates never get the same slot.

use crate::{champions::ChampionMain, ranked::RecentForm};
use serde_json::Value;
use shaco::rest::RESTClient;
use std::collections::HashMap;

const SUMMONERS_RIFT_MAP_ID: i64 = 11;
const SOLO_DUO_QUEUE_ID: i64 = 420;
const HISTORY_GAMES: u32 = 20;

/// What a player's recent games say about them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerHistory {
    /// Solo/Duo results, used for the displayed win rate.
    pub recent: RecentForm,
    /// Summoner's Rift games, which the position/spell/champion counts below are taken from.
    pub games: u32,
    pub positions: HashMap<String, u32>,
    /// Exact (D, F) spell pairs; order matters because players keep their key bindings.
    pub spells: HashMap<(i64, i64), u32>,
    pub champions: HashMap<i64, u32>,
}

pub async fn get_player_history(
    remoting_client: &RESTClient,
    puuid: &str,
) -> anyhow::Result<PlayerHistory> {
    let history = remoting_client
        .get(format!(
            "/lol-match-history/v1/products/lol/{puuid}/matches?begIndex=0&endIndex={}",
            HISTORY_GAMES - 1
        ))
        .await?;
    if history["games"].is_null() {
        anyhow::bail!("match history unavailable: {}", history["message"]);
    }
    Ok(parse_player_history(&history))
}

/// The match list only includes the requested player, so `participants[0]` is them.
fn parse_player_history(history: &Value) -> PlayerHistory {
    let mut profile = PlayerHistory::default();
    for game in history["games"]["games"].as_array().into_iter().flatten() {
        let player = &game["participants"][0];
        if game["queueId"].as_i64() == Some(SOLO_DUO_QUEUE_ID) {
            match player["stats"]["win"].as_bool() {
                Some(true) => profile.recent.wins += 1,
                Some(false) => profile.recent.losses += 1,
                None => {}
            }
        }
        if game["mapId"].as_i64() != Some(SUMMONERS_RIFT_MAP_ID) {
            continue;
        }
        profile.games += 1;
        if let Some(position) = position_from_timeline(&player["timeline"]) {
            *profile.positions.entry(position.to_string()).or_default() += 1;
        }
        if let (Some(d), Some(f)) = (player["spell1Id"].as_i64(), player["spell2Id"].as_i64()) {
            *profile.spells.entry((d, f)).or_default() += 1;
        }
        if let Some(champion) = player["championId"].as_i64().filter(|id| *id > 0) {
            *profile.champions.entry(champion).or_default() += 1;
        }
    }
    profile
}

/// Maps match history's lane/role to Champ Select's assigned positions.
fn position_from_timeline(timeline: &Value) -> Option<&'static str> {
    match timeline["lane"].as_str()? {
        "TOP" => Some("top"),
        "JUNGLE" => Some("jungle"),
        "MIDDLE" | "MID" => Some("middle"),
        "BOTTOM" | "BOT" if timeline["role"].as_str() == Some("DUO_SUPPORT") => Some("utility"),
        "BOTTOM" | "BOT" => Some("bottom"),
        _ => None,
    }
}

/// A Champ Select slot whose player is hidden.
#[derive(Clone, Debug, PartialEq)]
pub struct HiddenSlot {
    pub cell_id: u32,
    pub position: String,
    pub spells: (i64, i64),
    /// Locked champion, or the hovered one before lock-in; 0 if none.
    pub champion_id: i64,
    pub autofilled: bool,
}

/// A teammate from chat who isn't matched to a slot yet.
pub struct Candidate<'a> {
    pub history: Option<&'a PlayerHistory>,
    pub mains: &'a [ChampionMain],
}

/// How well a teammate fits a slot. Each signal is a share of their recent games, so a player
/// who always takes the slot's spells and position scores about 2.
fn score(candidate: &Candidate, slot: &HiddenSlot) -> f64 {
    let Some(history) = candidate.history.filter(|history| history.games > 0) else {
        return 0.0;
    };
    let games = f64::from(history.games);
    let share = |count: Option<&u32>| f64::from(count.copied().unwrap_or(0)) / games;

    // Autofilled players are off their usual role, so their history says little here.
    let position_weight = if slot.autofilled { 0.25 } else { 1.0 };
    let position = position_weight * share(history.positions.get(&slot.position));

    let (d, f) = slot.spells;
    let spells = share(history.spells.get(&(d, f))) + 0.25 * share(history.spells.get(&(f, d)));

    let champion = if slot.champion_id > 0 {
        let main_bonus = if candidate
            .mains
            .iter()
            .any(|main| i64::from(main.champion_id) == slot.champion_id)
        {
            0.5
        } else {
            0.0
        };
        2.0 * share(history.champions.get(&slot.champion_id)) + main_bonus
    } else {
        0.0
    };

    position + spells + champion
}

/// Returns, for each candidate, the index of the slot it most likely belongs to (or `None` when
/// there are more candidates than slots). Tries every assignment; at most 5 players, so 120.
pub fn assign(candidates: &[Candidate], slots: &[HiddenSlot]) -> Vec<Option<usize>> {
    let scores: Vec<Vec<f64>> = candidates
        .iter()
        .map(|candidate| slots.iter().map(|slot| score(candidate, slot)).collect())
        .collect();

    let mut best = (f64::MIN, vec![None; candidates.len()]);
    let mut current = vec![None; candidates.len()];
    let mut used = vec![false; slots.len()];
    search(0, 0.0, &scores, &mut used, &mut current, &mut best);
    best.1
}

fn search(
    index: usize,
    total: f64,
    scores: &[Vec<f64>],
    used: &mut [bool],
    current: &mut Vec<Option<usize>>,
    best: &mut (f64, Vec<Option<usize>>),
) {
    if index == scores.len() {
        if total > best.0 {
            *best = (total, current.clone());
        }
        return;
    }
    let free_slots = used.iter().filter(|used| !**used).count();
    let remaining_candidates = scores.len() - index;
    // Leave this candidate unplaced only when there aren't enough slots for everyone.
    if remaining_candidates > free_slots {
        current[index] = None;
        search(index + 1, total, scores, used, current, best);
    }
    for slot in 0..used.len() {
        if used[slot] {
            continue;
        }
        used[slot] = true;
        current[index] = Some(slot);
        search(
            index + 1,
            total + scores[index][slot],
            scores,
            used,
            current,
            best,
        );
        used[slot] = false;
    }
    current[index] = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn game(
        queue: i64,
        map: i64,
        lane: &str,
        role: &str,
        spells: (i64, i64),
        champion: i64,
        win: bool,
    ) -> Value {
        json!({
            "queueId": queue, "mapId": map,
            "participants": [{
                "championId": champion, "spell1Id": spells.0, "spell2Id": spells.1,
                "stats": { "win": win }, "timeline": { "lane": lane, "role": role }
            }]
        })
    }

    #[test]
    fn parses_positions_spells_champions_and_solo_record() {
        let history = parse_player_history(&json!({ "games": { "games": [
            game(420, 11, "BOTTOM", "DUO_SUPPORT", (4, 7), 99, true),
            game(420, 11, "BOTTOM", "DUO_SUPPORT", (4, 7), 99, false),
            game(440, 11, "JUNGLE", "NONE", (11, 4), 64, true),
            game(450, 12, "MIDDLE", "SOLO", (4, 32), 103, true),
        ]}}));
        assert_eq!(history.recent, RecentForm { wins: 1, losses: 1 });
        assert_eq!(history.games, 3, "ARAM doesn't count toward positions");
        assert_eq!(history.positions.get("utility"), Some(&2));
        assert_eq!(history.positions.get("jungle"), Some(&1));
        assert_eq!(history.spells.get(&(4, 7)), Some(&2));
        assert_eq!(history.champions.get(&99), Some(&2));
        assert_eq!(history.positions.get("middle"), None);
    }

    fn profile(position: &str, spells: (i64, i64), champion: i64) -> PlayerHistory {
        let mut history = PlayerHistory {
            games: 10,
            ..Default::default()
        };
        history.positions.insert(position.to_string(), 8);
        history.spells.insert(spells, 9);
        history.champions.insert(champion, 5);
        history
    }

    fn slot(cell_id: u32, position: &str, spells: (i64, i64), champion_id: i64) -> HiddenSlot {
        HiddenSlot {
            cell_id,
            position: position.to_string(),
            spells,
            champion_id,
            autofilled: false,
        }
    }

    /// Mirrors a real ranked lobby: jungle on Smite, ADC on Barrier, support on Heal, and a
    /// player whose history says "jungle" but who took Teleport, so belongs in top.
    #[test]
    fn assigns_a_real_lobby_by_role_and_spells() {
        let jungle = profile("jungle", (11, 4), 64);
        let adc = profile("bottom", (4, 21), 22);
        let support = profile("utility", (4, 7), 99);
        let mut top = profile("jungle", (4, 12), 23);
        top.positions.insert("jungle".to_string(), 5);
        top.positions.insert("top".to_string(), 3);
        let histories = [&support, &top, &jungle, &adc];
        let candidates: Vec<Candidate> = histories
            .iter()
            .map(|history| Candidate {
                history: Some(history),
                mains: &[],
            })
            .collect();
        let slots = [
            slot(5, "utility", (4, 7), 0),
            slot(6, "top", (4, 12), 0),
            slot(7, "jungle", (11, 4), 0),
            slot(9, "bottom", (4, 21), 0),
        ];

        let assignment = assign(&candidates, &slots);

        let cells: Vec<u32> = assignment
            .iter()
            .map(|slot_index| slots[slot_index.unwrap()].cell_id)
            .collect();
        assert_eq!(cells, vec![5, 6, 7, 9]);
    }

    #[test]
    fn a_hovered_main_outweighs_a_spell_tie() {
        let mut lee = profile("jungle", (11, 4), 64);
        lee.positions.clear();
        let mut other = profile("jungle", (11, 4), 1);
        other.positions.clear();
        let lee_mains = [ChampionMain {
            champion_id: 64,
            name: "Lee Sin".into(),
            level: 30,
            points: 300_000,
        }];
        let candidates = [
            Candidate {
                history: Some(&other),
                mains: &[],
            },
            Candidate {
                history: Some(&lee),
                mains: &lee_mains,
            },
        ];
        let slots = [slot(1, "jungle", (11, 4), 64), slot(2, "top", (11, 4), 0)];

        assert_eq!(assign(&candidates, &slots), vec![Some(1), Some(0)]);
    }

    #[test]
    fn handles_missing_history_and_more_players_than_slots() {
        let jungle = profile("jungle", (11, 4), 64);
        let candidates = [
            Candidate {
                history: None,
                mains: &[],
            },
            Candidate {
                history: Some(&jungle),
                mains: &[],
            },
        ];
        let slots = [slot(7, "jungle", (11, 4), 0)];

        assert_eq!(assign(&candidates, &slots), vec![None, Some(0)]);
    }
}
