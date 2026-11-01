//! Alerts for things that happen while you might not be watching: on this PC (Windows
//! notifications), on your phone (the ntfy app) and in a Discord channel (a webhook). Each alert
//! is turned on per channel in the settings. Only the "team revealed" alert names players: it
//! lists your team on the phone and in Discord, so turning it on there is the opt-in.

use crate::config::{AppConfig, NotificationSettings};
use anyhow::Context;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{sync::OnceLock, time::Duration};
use tauri::{AppHandle, Manager};

const NTFY_URL: &str = "https://ntfy.sh";
const DISCORD_HOSTS: [&str; 4] = [
    "discord.com",
    "discordapp.com",
    "ptb.discord.com",
    "canary.discord.com",
];
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alert {
    AutoQueue,
    MatchFound,
    YourTurn,
    AutoAction,
    Dodge,
    LobbyRevealed,
    /// The settings' "Send test" button; never turned off.
    Test,
}

impl Alert {
    /// The name the settings list it under.
    pub fn key(self) -> &'static str {
        match self {
            Alert::AutoQueue => "autoQueue",
            Alert::MatchFound => "matchFound",
            Alert::YourTurn => "yourTurn",
            Alert::AutoAction => "autoAction",
            Alert::Dodge => "dodge",
            Alert::LobbyRevealed => "lobbyRevealed",
            Alert::Test => "test",
        }
    }

    /// ntfy shows the first tag that names an emoji next to the title.
    fn ntfy_tag(self) -> &'static str {
        match self {
            Alert::AutoQueue => "repeat",
            Alert::MatchFound => "crossed_swords",
            Alert::YourTurn => "hourglass_flowing_sand",
            Alert::AutoAction => "robot",
            Alert::Dodge => "door",
            Alert::LobbyRevealed => "eyes",
            Alert::Test => "bell",
        }
    }

    /// The things to act on right away buzz harder on the phone.
    fn urgent(self) -> bool {
        matches!(self, Alert::MatchFound | Alert::YourTurn)
    }

    fn discord_color(self) -> u32 {
        match self {
            Alert::AutoQueue => 0x06b6d4,
            Alert::MatchFound => 0x3b82f6,
            Alert::YourTurn => 0xf59e0b,
            Alert::AutoAction => 0x10b981,
            Alert::Dodge => 0xef4444,
            Alert::LobbyRevealed => 0x8b5cf6,
            Alert::Test => 0x64748b,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Desktop,
    Phone,
    Discord,
}

impl Channel {
    pub fn from_setting(setting: &str) -> Option<Self> {
        match setting {
            "desktop" => Some(Channel::Desktop),
            "phone" => Some(Channel::Phone),
            "discord" => Some(Channel::Discord),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    pub alert: Alert,
    pub title: String,
    pub body: String,
    /// Listed on the phone and in Discord, but not in the short Windows notification.
    pub players: Vec<NoticePlayer>,
    /// Opened by tapping the phone alert or the Discord title.
    pub link: Option<String>,
}

/// One player in an alert: their Riot ID and what to know about them.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct NoticePlayer {
    pub name: String,
    pub detail: String,
}

impl Notice {
    pub fn new(alert: Alert, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            alert,
            title: title.into(),
            body: body.into(),
            players: Vec::new(),
            link: None,
        }
    }

    pub fn with_players(mut self, players: Vec<NoticePlayer>) -> Self {
        self.players = players;
        self
    }

    /// Only web links are kept, so an alert can't open anything else.
    pub fn with_link(mut self, link: Option<String>) -> Self {
        self.link = link.filter(|link| link.starts_with("https://"));
        self
    }
}

/// Discord allows 25 fields per embed; a lobby never comes close.
const MAX_DISCORD_FIELDS: usize = 25;

/// The channels `alert` goes to: those it's turned on for, once they're set up.
pub fn channels(settings: &NotificationSettings, alert: Alert) -> Vec<Channel> {
    let on = |alerts: &[String]| alert == Alert::Test || alerts.iter().any(|key| key == alert.key());
    let mut channels = Vec::new();
    if on(&settings.desktop) {
        channels.push(Channel::Desktop);
    }
    if on(&settings.phone) && !settings.ntfy_topic.trim().is_empty() {
        channels.push(Channel::Phone);
    }
    if on(&settings.discord) && !settings.discord_webhook.trim().is_empty() {
        channels.push(Channel::Discord);
    }
    channels
}

/// Whether any channel would send `alert`, to skip working out ones nobody gets.
pub fn wanted(settings: &NotificationSettings, alert: Alert) -> bool {
    !channels(settings, alert).is_empty()
}

/// Sends the notice to every channel that has its alert on, in the background.
pub fn send(app_handle: &AppHandle, notice: Notice) {
    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let settings = current_settings(&app_handle).await;
        for channel in channels(&settings, notice.alert) {
            match deliver(&app_handle, &settings, channel, &notice).await {
                Ok(()) => log_info!("Sent the {} alert to {channel:?}", notice.alert.key()),
                Err(error) => log_warn!(
                    "Couldn't send the {} alert to {channel:?}: {error:#}",
                    notice.alert.key()
                ),
            }
        }
    });
}

/// Sends a sample alert to one channel with the settings as they are on screen (maybe not saved
/// yet), so the settings can show whether it's set up right.
pub async fn send_test(
    app_handle: &AppHandle,
    channel: Channel,
    settings: &NotificationSettings,
) -> anyhow::Result<()> {
    let notice = Notice::new(
        Alert::Test,
        "Reveal notifications work",
        "Alerts from Reveal will show up here like this.",
    );
    deliver(app_handle, settings, channel, &notice).await
}

async fn current_settings(app_handle: &AppHandle) -> NotificationSettings {
    let config = app_handle.state::<AppConfig>();
    let settings = config.0.lock().await.notifications.clone();
    settings
}

async fn deliver(
    app_handle: &AppHandle,
    settings: &NotificationSettings,
    channel: Channel,
    notice: &Notice,
) -> anyhow::Result<()> {
    match channel {
        Channel::Desktop => {
            tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
                .title(&notice.title)
                .body(&notice.body)
                .show()
                .context("Windows didn't show the notification")
        }
        Channel::Phone => {
            let topic = ntfy_topic(&settings.ntfy_topic)?;
            post(NTFY_URL, &ntfy_message(topic, notice)).await
        }
        Channel::Discord => {
            let webhook = discord_webhook(&settings.discord_webhook)?;
            post(webhook.as_str(), &discord_message(notice)).await
        }
    }
}

/// An ntfy.sh topic: what the phone app subscribes to. Anyone who knows it can read the alerts,
/// so Reveal suggests a long random one.
fn ntfy_topic(topic: &str) -> anyhow::Result<&str> {
    let topic = topic.trim();
    let valid = (1..=64).contains(&topic.len())
        && topic
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));
    anyhow::ensure!(
        valid,
        "the ntfy topic can only use letters, digits, - and _ (up to 64)"
    );
    Ok(topic)
}

fn ntfy_message(topic: &str, notice: &Notice) -> Value {
    let mut message = notice.body.clone();
    if !notice.players.is_empty() {
        message.push('\n');
        for player in &notice.players {
            message.push('\n');
            message.push_str(&player.name);
            if !player.detail.is_empty() {
                message.push_str(" · ");
                message.push_str(&player.detail);
            }
        }
    }
    let mut published = json!({
        "topic": topic,
        "title": notice.title,
        "message": message,
        "tags": [notice.alert.ntfy_tag()],
        "priority": if notice.alert.urgent() { 4 } else { 3 },
    });
    if let Some(link) = &notice.link {
        published["click"] = json!(link);
    }
    published
}

/// A Discord webhook URL, from a channel's Integrations settings. Only Discord's own hosts are
/// accepted, so the setting can't send alerts anywhere else.
fn discord_webhook(webhook: &str) -> anyhow::Result<reqwest::Url> {
    let url = reqwest::Url::parse(webhook.trim()).context("the Discord webhook isn't a URL")?;
    let valid = url.scheme() == "https"
        && url.host_str().is_some_and(|host| DISCORD_HOSTS.contains(&host))
        && url.path().starts_with("/api/webhooks/");
    anyhow::ensure!(
        valid,
        "the Discord webhook should start with https://discord.com/api/webhooks/"
    );
    Ok(url)
}

fn discord_message(notice: &Notice) -> Value {
    let fields: Vec<Value> = notice
        .players
        .iter()
        .take(MAX_DISCORD_FIELDS)
        .map(|player| {
            // Discord turns down empty field values; a zero-width space shows nothing.
            let value = if player.detail.is_empty() { "\u{200b}" } else { player.detail.as_str() };
            json!({ "name": player.name, "value": value, "inline": true })
        })
        .collect();
    let mut embed = json!({
        "title": notice.title,
        "description": notice.body,
        "color": notice.alert.discord_color(),
        "fields": fields,
    });
    if let Some(link) = &notice.link {
        embed["url"] = json!(link);
    }
    json!({
        "username": "Reveal",
        // Alerts never ping anyone, whatever their text says.
        "allowed_mentions": { "parse": [] },
        "embeds": [embed],
    })
}

fn http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(concat!("Reveal/", env!("CARGO_PKG_VERSION")))
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_default()
    })
}

/// Posts the JSON. Errors leave the URL out, since a webhook URL is enough to post as Reveal.
async fn post(url: &str, body: &Value) -> anyhow::Result<()> {
    http()
        .post(url)
        .json(body)
        .send()
        .await
        .map_err(reqwest::Error::without_url)?
        .error_for_status()
        .map_err(reqwest::Error::without_url)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> NotificationSettings {
        NotificationSettings {
            ntfy_topic: "reveal-test-topic".to_string(),
            discord_webhook: "https://discord.com/api/webhooks/1/abc".to_string(),
            ..NotificationSettings::default()
        }
    }

    #[test]
    fn sends_each_alert_where_it_is_turned_on() {
        let settings = settings();
        assert_eq!(
            channels(&settings, Alert::MatchFound),
            vec![Channel::Phone, Channel::Discord],
            "League already shows match found on this PC"
        );
        assert_eq!(
            channels(&settings, Alert::AutoAction),
            vec![Channel::Desktop]
        );
        assert!(channels(&settings, Alert::LobbyRevealed).is_empty());
        assert_eq!(channels(&settings, Alert::Test).len(), 3);
    }

    #[test]
    fn skips_channels_that_are_not_set_up() {
        let settings = NotificationSettings::default();
        assert_eq!(channels(&settings, Alert::MatchFound), vec![]);
        assert!(!wanted(&settings, Alert::YourTurn));
        assert!(wanted(&settings, Alert::Dodge));
    }

    #[test]
    fn accepts_only_plain_ntfy_topics() {
        assert_eq!(ntfy_topic(" reveal-Ab_12 ").unwrap(), "reveal-Ab_12");
        assert!(ntfy_topic("").is_err());
        assert!(ntfy_topic("has space").is_err());
        assert!(ntfy_topic("https://ntfy.sh/topic").is_err());
        assert!(ntfy_topic(&"a".repeat(65)).is_err());
    }

    #[test]
    fn accepts_only_discord_webhooks() {
        assert!(discord_webhook("https://discord.com/api/webhooks/123/token").is_ok());
        assert!(discord_webhook("https://canary.discord.com/api/webhooks/123/token").is_ok());
        assert!(discord_webhook("http://discord.com/api/webhooks/123/token").is_err());
        assert!(discord_webhook("https://discord.com.evil.example/api/webhooks/1/t").is_err());
        assert!(discord_webhook("https://discord.com/channels/1/2").is_err());
        assert!(discord_webhook("not a url").is_err());
    }

    #[test]
    fn builds_messages_each_service_understands() {
        let notice = Notice::new(Alert::MatchFound, "Match found", "Accept it in League");
        let ntfy = ntfy_message("topic", &notice);
        assert_eq!(ntfy["title"], "Match found");
        assert_eq!(ntfy["priority"], 4);
        assert_eq!(ntfy["tags"][0], "crossed_swords");

        let discord = discord_message(&notice);
        assert_eq!(discord["embeds"][0]["description"], "Accept it in League");
        assert_eq!(discord["allowed_mentions"]["parse"], json!([]));
    }

    /// Publishes to a throwaway topic and checks how ntfy read it:
    /// `cargo test live_ntfy -- --ignored --nocapture`.
    #[test]
    #[ignore = "publishes to ntfy.sh"]
    fn live_ntfy_delivers_alerts() {
        tauri::async_runtime::block_on(async {
            let topic = format!("reveal-test-{}", std::process::id());
            let notice = Notice::new(Alert::Test, "Reveal notifications work", "Test from cargo test")
                .with_players(vec![NoticePlayer {
                    name: "Paper Lantern#NA1".to_string(),
                    detail: "Top · Diamond II".to_string(),
                }]);
            let published: Value = http()
                .post(NTFY_URL)
                .json(&ntfy_message(&topic, &notice))
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .expect("ntfy accepts the message")
                .json()
                .await
                .unwrap();
            println!("{published}");
            assert_eq!(published["topic"], topic.as_str());
            assert_eq!(published["title"], "Reveal notifications work");
            assert_eq!(published["tags"][0], "bell");
            assert!(published["message"]
                .as_str()
                .unwrap()
                .ends_with("Paper Lantern#NA1 · Top · Diamond II"));
        });
    }

    #[test]
    fn lists_the_lobby_on_the_phone_and_in_discord() {
        let link = "https://op.gg/lol/multisearch/na?summoners=a";
        let notice = Notice::new(Alert::LobbyRevealed, "Your team is revealed", "Nobody new.")
            .with_players(vec![
                NoticePlayer {
                    name: "Paper Lantern#NA1".to_string(),
                    detail: "Top · Diamond II".to_string(),
                },
                NoticePlayer {
                    name: "River Walker#SUP".to_string(),
                    detail: String::new(),
                },
            ])
            .with_link(Some(link.to_string()));

        let ntfy = ntfy_message("topic", &notice);
        assert_eq!(
            ntfy["message"],
            "Nobody new.\n\nPaper Lantern#NA1 · Top · Diamond II\nRiver Walker#SUP"
        );
        assert_eq!(ntfy["click"], link);

        let embed = &discord_message(&notice)["embeds"][0];
        assert_eq!(embed["fields"][0]["name"], "Paper Lantern#NA1");
        assert_eq!(embed["fields"][0]["value"], "Top · Diamond II");
        assert_eq!(embed["fields"][1]["value"], "\u{200b}");
        assert_eq!(embed["url"], link);

        let plain = Notice::new(Alert::Dodge, "Dodge", "Back in queue.")
            .with_link(Some("file:///C:/Windows".to_string()));
        assert_eq!(plain.link, None, "only web links");
        assert!(discord_message(&plain)["embeds"][0].get("url").is_none());
        assert!(ntfy_message("topic", &plain).get("click").is_none());
    }

    #[test]
    fn hides_the_topic_and_webhook_when_logged() {
        let logged = format!("{:?}", settings());
        assert!(!logged.contains("reveal-test-topic"));
        assert!(!logged.contains("webhooks"));
    }
}
