use serde_json::json;

/// Reports that a lobby was revealed. The payload is deliberately anonymous:
/// no Riot IDs, PUUIDs, or other player identifiers are sent.
pub async fn send_analytics_event(
    participant_count: usize,
    region: &str,
    provider: &str,
    app_version: &str,
) {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://lobbyreveal.app/api/reveal/v2/collect")
        .json(&json!({
            "region": region,
            "participantCount": participant_count,
            "provider": provider,
            "appVersion": app_version
        }))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    if let Err(error) = resp {
        log_warn!("Analytics request failed: {error}");
    }
}
