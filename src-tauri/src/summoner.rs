use serde::{Deserialize, Serialize};
use shaco::rest::RESTClient;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summoner {
    pub game_name: String,
    pub tag_line: String,
}

pub async fn get_current_summoner(remoting_client: &RESTClient) -> anyhow::Result<Summoner> {
    let summoner: Summoner = serde_json::from_value(
        remoting_client
            .get("/lol-summoner/v1/current-summoner".to_string())
            .await?,
    )?;

    Ok(summoner)
}
