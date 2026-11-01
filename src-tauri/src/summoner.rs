use crate::ranked::RankSummary;
use serde::{Deserialize, Serialize};
use shaco::rest::RESTClient;

/// The player signed in to the League Client, for the idle screen.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnProfile {
    pub game_name: String,
    pub game_tag: String,
    pub puuid: String,
    pub profile_icon_id: i32,
    pub level: u32,
    pub solo: RankSummary,
    pub flex: RankSummary,
}

/// The signed-in player's Riot ID, icon, level and ranks.
pub async fn get_own_profile(remoting_client: &RESTClient) -> anyhow::Result<OwnProfile> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Summoner {
        game_name: String,
        tag_line: String,
        puuid: String,
        profile_icon_id: i32,
        summoner_level: u32,
    }

    let summoner: Summoner = serde_json::from_value(
        remoting_client
            .get("/lol-summoner/v1/current-summoner".to_string())
            .await?,
    )?;
    let (solo, flex) = crate::ranked::get_ranks(remoting_client, &summoner.puuid).await?;
    Ok(OwnProfile {
        game_name: summoner.game_name,
        game_tag: summoner.tag_line,
        puuid: summoner.puuid,
        profile_icon_id: summoner.profile_icon_id,
        level: summoner.summoner_level,
        solo,
        flex,
    })
}

/// A player's profile icon id, from the League Client's local summoner API.
pub async fn get_profile_icon_id(remoting_client: &RESTClient, puuid: &str) -> anyhow::Result<i32> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Summoner {
        profile_icon_id: i32,
    }

    let summoner: Summoner = serde_json::from_value(
        remoting_client
            .get_with_timeout(
                format!("/lol-summoner/v2/summoners/puuid/{puuid}"),
                crate::lookup::LOOKUP_TIMEOUT,
            )
            .await?,
    )?;
    Ok(summoner.profile_icon_id)
}
