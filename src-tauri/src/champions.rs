use serde::{Deserialize, Serialize};
use shaco::rest::RESTClient;
use std::collections::HashMap;

pub const MAINS_PER_PLAYER: usize = 3;

/// One of a player's most-played champions, by mastery points.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionMain {
    pub champion_id: i32,
    pub name: String,
    pub level: u32,
    pub points: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MasteryEntry {
    champion_id: i32,
    #[serde(default)]
    champion_level: u32,
    #[serde(default)]
    champion_points: u32,
}

#[derive(Deserialize)]
struct ChampionSummary {
    id: i32,
    name: String,
    /// The language-independent name, e.g. "MonkeyKing" for Wukong.
    #[serde(default)]
    alias: String,
}

/// A champion's name in the client's language, and the name stats sites use in URLs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChampionInfo {
    pub name: String,
    pub slug: String,
}

async fn get_champion_summary(
    remoting_client: &RESTClient,
) -> anyhow::Result<Vec<ChampionSummary>> {
    // A few hundred champions take longer than the client's default 500 ms on a busy client.
    let summary: Vec<ChampionSummary> = serde_json::from_value(
        remoting_client
            .get_with_timeout(
                "/lol-game-data/assets/v1/champion-summary.json".to_string(),
                crate::lookup::LOOKUP_TIMEOUT,
            )
            .await?,
    )?;
    Ok(summary
        .into_iter()
        // Game modes add copies from 60000 up, and -1 is "None".
        .filter(|champion| champion.id > 0 && champion.id < 10_000)
        .collect())
}

/// Champion names by id, from the League Client's local game data.
pub async fn get_champion_names(
    remoting_client: &RESTClient,
) -> anyhow::Result<HashMap<i32, String>> {
    Ok(get_champion_summary(remoting_client)
        .await?
        .into_iter()
        .map(|champion| (champion.id, champion.name))
        .collect())
}

/// Champion names and URL slugs by id, from the League Client's local game data.
pub async fn get_champion_info(
    remoting_client: &RESTClient,
) -> anyhow::Result<HashMap<i32, ChampionInfo>> {
    Ok(get_champion_summary(remoting_client)
        .await?
        .into_iter()
        .map(|champion| {
            let info = ChampionInfo {
                slug: champion_slug(&champion.alias),
                name: champion.name,
            };
            (champion.id, info)
        })
        .collect())
}

/// "LeeSin" -> "leesin", "MonkeyKing" -> "wukong": how U.GG, Lolalytics and League of Graphs
/// name champions in their URLs.
pub fn champion_slug(alias: &str) -> String {
    match alias {
        "MonkeyKing" => "wukong".to_string(),
        _ => alias.to_lowercase(),
    }
}

pub async fn get_top_mains(
    remoting_client: &RESTClient,
    puuid: &str,
    names: &HashMap<i32, String>,
) -> anyhow::Result<Vec<ChampionMain>> {
    let mastery: Vec<MasteryEntry> = serde_json::from_value(
        remoting_client
            .get_with_timeout(
                format!("/lol-champion-mastery/v1/{puuid}/champion-mastery"),
                crate::lookup::LOOKUP_TIMEOUT,
            )
            .await?,
    )?;
    Ok(top_mains(mastery, names))
}

fn top_mains(mut mastery: Vec<MasteryEntry>, names: &HashMap<i32, String>) -> Vec<ChampionMain> {
    mastery.sort_by(|a, b| b.champion_points.cmp(&a.champion_points));
    mastery
        .into_iter()
        .filter(|entry| entry.champion_points > 0)
        .take(MAINS_PER_PLAYER)
        .map(|entry| ChampionMain {
            name: names
                .get(&entry.champion_id)
                .cloned()
                .unwrap_or_else(|| format!("Champion {}", entry.champion_id)),
            champion_id: entry.champion_id,
            level: entry.champion_level,
            points: entry.champion_points,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(champion_id: i32, champion_points: u32) -> MasteryEntry {
        MasteryEntry {
            champion_id,
            champion_level: 10,
            champion_points,
        }
    }

    #[test]
    fn picks_the_top_three_by_mastery_points_with_names() {
        let names = HashMap::from([(157, "Yasuo".to_string()), (92, "Riven".to_string())]);
        let mains = top_mains(
            vec![
                entry(1, 50),
                entry(92, 394_816),
                entry(238, 384_110),
                entry(157, 406_821),
                entry(2, 0),
            ],
            &names,
        );

        assert_eq!(
            mains
                .iter()
                .map(|main| main.champion_id)
                .collect::<Vec<_>>(),
            vec![157, 92, 238]
        );
        assert_eq!(mains[0].name, "Yasuo");
        assert_eq!(mains[2].name, "Champion 238");
    }

    #[test]
    fn slugs_champions_like_stats_sites_do() {
        assert_eq!(champion_slug("LeeSin"), "leesin");
        assert_eq!(champion_slug("JarvanIV"), "jarvaniv");
        assert_eq!(champion_slug("MonkeyKing"), "wukong");
        assert_eq!(champion_slug("Nunu"), "nunu");
        assert_eq!(champion_slug(""), "");
    }

    #[test]
    fn skips_champions_with_no_points() {
        let mains = top_mains(vec![entry(1, 0), entry(2, 10)], &HashMap::new());
        assert_eq!(mains.len(), 1);
        assert_eq!(mains[0].champion_id, 2);
    }
}
