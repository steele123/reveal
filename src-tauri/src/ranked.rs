use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaco::rest::RESTClient;

/// A player's current Solo/Duo standing, read from the League Client's local ranked API.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankSummary {
    /// "IRON" through "CHALLENGER", or "UNRANKED".
    pub tier: String,
    /// "I" through "IV"; absent for Master and above, which have no divisions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub division: Option<String>,
    pub league_points: u32,
    pub wins: u32,
    pub losses: u32,
    /// Still playing placement games this split.
    pub provisional: bool,
}

/// Solo/Duo results from a player's recent match history. League's ranked stats report other
/// players' losses as 0, so this is the only reliable win rate for teammates.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentForm {
    pub wins: u32,
    pub losses: u32,
}

const SOLO_DUO_QUEUE_ID: i64 = 420;
const RECENT_GAMES: u32 = 20;

pub async fn get_recent_solo_form(
    remoting_client: &RESTClient,
    puuid: &str,
) -> anyhow::Result<RecentForm> {
    let history = remoting_client
        .get(format!(
            "/lol-match-history/v1/products/lol/{puuid}/matches?begIndex=0&endIndex={}",
            RECENT_GAMES - 1
        ))
        .await?;
    if history["games"].is_null() {
        anyhow::bail!("match history unavailable: {}", history["message"]);
    }
    Ok(parse_recent_solo_form(&history))
}

/// The match list only includes the requested player, so `participants[0]` is them.
fn parse_recent_solo_form(history: &Value) -> RecentForm {
    let mut form = RecentForm { wins: 0, losses: 0 };
    for game in history["games"]["games"].as_array().into_iter().flatten() {
        if game["queueId"].as_i64() != Some(SOLO_DUO_QUEUE_ID) {
            continue;
        }
        match game["participants"][0]["stats"]["win"].as_bool() {
            Some(true) => form.wins += 1,
            Some(false) => form.losses += 1,
            None => {}
        }
    }
    form
}

pub async fn get_solo_rank(
    remoting_client: &RESTClient,
    puuid: &str,
) -> anyhow::Result<RankSummary> {
    let stats = remoting_client
        .get(format!("/lol-ranked/v1/ranked-stats/{puuid}"))
        .await?;
    Ok(parse_solo_rank(&stats))
}

fn parse_solo_rank(stats: &Value) -> RankSummary {
    let solo = &stats["queueMap"]["RANKED_SOLO_5x5"];
    let number = |key: &str| solo[key].as_u64().unwrap_or(0) as u32;
    let tier = solo["tier"].as_str().unwrap_or_default();
    let provisional = solo["isProvisional"].as_bool().unwrap_or(false);

    if tier.is_empty() || tier == "NONE" {
        return RankSummary {
            tier: "UNRANKED".to_string(),
            division: None,
            league_points: 0,
            wins: number("wins"),
            losses: number("losses"),
            provisional,
        };
    }

    let division = solo["division"]
        .as_str()
        .filter(|division| !division.is_empty() && *division != "NA")
        .map(str::to_string);

    RankSummary {
        tier: tier.to_string(),
        division,
        league_points: number("leaguePoints"),
        wins: number("wins"),
        losses: number("losses"),
        provisional,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_a_divisioned_solo_rank() {
        let rank = parse_solo_rank(&json!({
            "queueMap": { "RANKED_SOLO_5x5": {
                "tier": "DIAMOND", "division": "II", "leaguePoints": 45,
                "wins": 70, "losses": 50, "isProvisional": false
            }}
        }));
        assert_eq!(rank.tier, "DIAMOND");
        assert_eq!(rank.division.as_deref(), Some("II"));
        assert_eq!((rank.league_points, rank.wins, rank.losses), (45, 70, 50));
    }

    #[test]
    fn apex_tiers_have_no_division() {
        let rank = parse_solo_rank(&json!({
            "queueMap": { "RANKED_SOLO_5x5": {
                "tier": "MASTER", "division": "NA", "leaguePoints": 155,
                "wins": 376, "losses": 328, "isProvisional": false
            }}
        }));
        assert_eq!(rank.tier, "MASTER");
        assert_eq!(rank.division, None);
        assert_eq!(rank.league_points, 155);
    }

    #[test]
    fn counts_only_solo_duo_results_from_match_history() {
        let game = |queue: i64, win: bool| json!({ "queueId": queue, "participants": [{ "stats": { "win": win } }] });
        let form = parse_recent_solo_form(&json!({ "games": { "games": [
            game(420, true), game(420, false), game(420, true), game(440, false), game(450, true)
        ]}}));
        assert_eq!(form, RecentForm { wins: 2, losses: 1 });
        assert_eq!(
            parse_recent_solo_form(&json!({})),
            RecentForm { wins: 0, losses: 0 }
        );
    }

    #[test]
    fn missing_or_empty_solo_queue_is_unranked() {
        for stats in [
            json!({ "queueMap": { "RANKED_SOLO_5x5": { "tier": "", "division": "NA" } } }),
            json!({ "queueMap": {} }),
            json!({}),
        ] {
            let rank = parse_solo_rank(&stats);
            assert_eq!(rank.tier, "UNRANKED");
            assert_eq!(rank.division, None);
        }
    }
}
