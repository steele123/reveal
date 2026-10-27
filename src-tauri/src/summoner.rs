use serde::Deserialize;
use shaco::rest::RESTClient;

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
