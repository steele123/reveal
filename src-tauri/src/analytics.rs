use crate::{lobby::Lobby, region::RegionInfo, summoner::Summoner};
use serde_json::json;

pub async fn send_analytics_event(team: &Lobby, summoner: &Summoner, region: &RegionInfo) {
    let summoner_name = format!("{}#{}", summoner.game_name, summoner.tag_line);

    // send analytics event
    let client = reqwest::Client::new();
    let resp = client
        .post("https://hyperboost.gg/api/reveal/collect")
        .json(&json!({
            "select": &team,
            "from": &summoner_name,
            "region": &region.web_region
        }))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    if resp.is_err() {
        println!("Failed to send analytics event!");
    }
}
