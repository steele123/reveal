use serde_json::Value;
use shaco::rest::RESTClient;

/// The current queue's display name, e.g. "Ranked Solo/Duo", "Draft Pick" or "Custom game".
pub async fn current_queue_name(remoting_client: &RESTClient) -> Option<String> {
    let session = remoting_client
        .get("/lol-gameflow/v1/session".to_string())
        .await
        .ok()?;
    queue_name(&session["gameData"]["queue"])
}

fn queue_name(queue: &Value) -> Option<String> {
    if queue["isCustom"].as_bool() == Some(true) {
        return Some("Custom game".to_string());
    }
    // `description` is the most specific ("Draft Pick" where `name` is just "Normal").
    ["description", "name", "shortName"]
        .iter()
        .filter_map(|key| queue[*key].as_str())
        .find(|name| !name.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn prefers_the_specific_description() {
        assert_eq!(
            queue_name(&json!({ "name": "Normal", "description": "Draft Pick" })).as_deref(),
            Some("Draft Pick")
        );
        assert_eq!(
            queue_name(&json!({ "name": "Ranked 5s", "description": "" })).as_deref(),
            Some("Ranked 5s")
        );
        assert_eq!(
            queue_name(&json!({ "isCustom": true, "description": "x" })).as_deref(),
            Some("Custom game")
        );
        assert_eq!(queue_name(&json!({})), None);
    }
}
