use crate::utils::MULTI_PROVIDERS;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tauri::AppHandle;
use tokio::sync::Mutex;

pub const DEFAULT_AUTO_OPEN_DELAY_SECONDS: u32 = 6;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub auto_open: bool,
    #[serde(default = "default_auto_open_delay_seconds")]
    pub auto_open_delay_seconds: u32,
    pub auto_accept: bool,
    pub accept_delay: u32,
    #[serde(default)]
    pub pause_queue_after_dodge: bool,
    #[serde(default = "default_provider")]
    pub multi_provider: String,
    /// Set once the user has answered the one-time "join our Discord" prompt.
    #[serde(default)]
    pub discord_prompt_seen: bool,
    /// What clicking a teammate does: "details" (Reveal popup) or "opgg" (open their profile).
    #[serde(default = "default_player_click_action")]
    pub player_click_action: String,
    /// Which site's win rates the Counters tab shows first: "lolalytics", "ugg" or
    /// "laddereye". All are read.
    #[serde(default = "default_counters_provider")]
    pub counters_provider: String,
    /// Queue again a little while after a game ends.
    #[serde(default)]
    pub auto_requeue: bool,
    /// How long to wait before queueing again after a game; 0 is right away.
    #[serde(default = "default_requeue_delay_seconds")]
    pub requeue_delay_seconds: u32,
    /// Leave and rejoin the queue once the wait passes League's estimate.
    #[serde(default)]
    pub requeue_past_estimate: bool,
    /// How long to wait between leaving a long queue and joining it again; 0 is right away.
    #[serde(default = "default_requeue_delay_seconds")]
    pub long_queue_requeue_delay_seconds: u32,
    /// How long a pause after a dodge lasts before Reveal queues again; 0 waits for you.
    #[serde(default)]
    pub dodge_pause_seconds: u32,
    /// In the last seconds of your ban, ban the first available of `ban_champions` unless you
    /// already chose one.
    #[serde(default)]
    pub auto_ban: bool,
    #[serde(default)]
    pub ban_champions: Vec<i64>,
    /// How many seconds before the ban timer runs out to ban; 1 means the last second.
    #[serde(default = "default_seconds_left")]
    pub auto_ban_seconds_left: u32,
    /// In the last seconds of your pick, lock in your hover, or else the first available of
    /// your role's `pick_champions`.
    #[serde(default)]
    pub auto_pick: bool,
    /// Champion ids in order of preference, by League position ("top" ... "utility").
    #[serde(default)]
    pub pick_champions: HashMap<String, Vec<i64>>,
    /// How many seconds before the pick timer runs out to lock in; 1 means the last second.
    #[serde(default = "default_seconds_left")]
    pub auto_pick_seconds_left: u32,
    #[serde(default)]
    pub notifications: NotificationSettings,
}

/// Which alerts go where. Alerts are listed by `notify::Alert::key`, kept as strings so a config
/// written by a newer Reveal with more alerts still loads.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct NotificationSettings {
    /// Shown as Windows notifications.
    pub desktop: Vec<String>,
    /// Sent to `ntfy_topic` on ntfy.sh, for the ntfy phone app.
    pub phone: Vec<String>,
    pub ntfy_topic: String,
    /// Posted to the channel of `discord_webhook`.
    pub discord: Vec<String>,
    pub discord_webhook: String,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        use crate::notify::Alert;
        let keys = |alerts: &[Alert]| alerts.iter().map(|alert| alert.key().to_string()).collect();
        // League already alerts on this PC when a match is found, so only away-from-PC channels
        // start with it. Phone and Discord stay off until a topic or webhook is set.
        let away = keys(&[Alert::MatchFound, Alert::YourTurn, Alert::Dodge]);
        Self {
            desktop: keys(&[Alert::AutoAction, Alert::Dodge]),
            phone: away.clone(),
            ntfy_topic: String::new(),
            discord: away,
            discord_webhook: String::new(),
        }
    }
}

/// Leaves the topic and webhook out: anyone who has either can read or post the alerts.
impl std::fmt::Debug for NotificationSettings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NotificationSettings")
            .field("desktop", &self.desktop)
            .field("phone", &self.phone)
            .field("ntfy_topic_set", &!self.ntfy_topic.is_empty())
            .field("discord", &self.discord)
            .field("discord_webhook_set", &!self.discord_webhook.is_empty())
            .finish()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_open: true,
            auto_open_delay_seconds: DEFAULT_AUTO_OPEN_DELAY_SECONDS,
            auto_accept: false,
            accept_delay: 2_000,
            pause_queue_after_dodge: false,
            multi_provider: default_provider(),
            discord_prompt_seen: false,
            player_click_action: default_player_click_action(),
            counters_provider: default_counters_provider(),
            auto_requeue: false,
            requeue_delay_seconds: default_requeue_delay_seconds(),
            requeue_past_estimate: false,
            long_queue_requeue_delay_seconds: default_requeue_delay_seconds(),
            dodge_pause_seconds: 0,
            auto_ban: false,
            ban_champions: Vec::new(),
            auto_ban_seconds_left: default_seconds_left(),
            auto_pick: false,
            pick_champions: HashMap::new(),
            auto_pick_seconds_left: default_seconds_left(),
            notifications: NotificationSettings::default(),
        }
    }
}

impl Config {
    /// Replaces a removed multi-search provider with the default. Returns true if it changed.
    pub fn normalize_provider(&mut self) -> bool {
        if MULTI_PROVIDERS.contains(&self.multi_provider.as_str()) {
            return false;
        }
        self.multi_provider = default_provider();
        true
    }
}

fn default_requeue_delay_seconds() -> u32 {
    30
}

fn default_auto_open_delay_seconds() -> u32 {
    DEFAULT_AUTO_OPEN_DELAY_SECONDS
}

fn default_player_click_action() -> String {
    "details".to_string()
}

fn default_provider() -> String {
    "opgg".to_string()
}

fn default_seconds_left() -> u32 {
    1
}

fn default_counters_provider() -> String {
    "lolalytics".to_string()
}

pub struct AppConfig(pub Mutex<Config>);

impl AppConfig {
    pub fn new(config: Config) -> Self {
        Self(Mutex::new(config))
    }
}

fn config_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let config_dir = app_handle
        .path_resolver()
        .app_config_dir()
        .context("Reveal has no application config directory")?;
    std::fs::create_dir_all(&config_dir).context("failed to create config directory")?;
    Ok(config_dir.join("config.json"))
}

pub fn load(app_handle: &AppHandle) -> Result<Config> {
    let config_path = config_path(app_handle)?;
    log_info!("Loading config from {}", config_path.display());
    load_from(&config_path)
}

/// Loads the config, replacing a missing or unreadable file with defaults so Reveal always starts.
/// An unreadable file is kept next to it as `config.json.bad` for troubleshooting.
fn load_from(config_path: &Path) -> Result<Config> {
    if !config_path.exists() {
        let config = Config::default();
        write_atomic(config_path, &config)?;
        log_info!("Created a default config");
        return Ok(config);
    }

    let parsed = std::fs::read_to_string(config_path)
        .context("failed to read config")
        .and_then(|json| serde_json::from_str::<Config>(&json).context("failed to parse config"));
    let mut config = match parsed {
        Ok(config) => config,
        Err(error) => {
            log_error!("Config is unreadable; resetting to defaults: {error:#}");
            if let Err(error) = std::fs::rename(config_path, config_path.with_extension("json.bad"))
            {
                log_warn!("Failed to back up the unreadable config: {error}");
            }
            let config = Config::default();
            write_atomic(config_path, &config)?;
            return Ok(config);
        }
    };

    if config.normalize_provider() {
        log_warn!("Saved multi-search provider is no longer supported; using OP.GG");
        write_atomic(config_path, &config)?;
    }
    log_info!("Config loaded successfully");
    Ok(config)
}

/// Writes to a temporary file and renames it over the config, so a crash mid-write
/// can never leave a truncated config behind.
fn write_atomic(config_path: &Path, config: &Config) -> Result<()> {
    let json = serde_json::to_string(config).context("failed to serialize config")?;
    let temp_path = config_path.with_extension("json.tmp");
    std::fs::write(&temp_path, json).context("failed to write config")?;
    std::fs::rename(&temp_path, config_path).context("failed to replace config")?;
    Ok(())
}

pub async fn save(app_handle: &AppHandle, config: &Config) -> Result<()> {
    let config_path = config_path(app_handle)?;
    let config = config.clone();
    tauri::async_runtime::spawn_blocking(move || write_atomic(&config_path, &config))
        .await
        .context("config save task failed")??;
    log_info!("Config saved successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_configs_receive_the_default_auto_open_delay() {
        let config: Config = serde_json::from_str(
            r#"{
                "autoOpen": true,
                "autoAccept": false,
                "acceptDelay": 2000,
                "multiProvider": "opgg"
            }"#,
        )
        .unwrap();

        assert_eq!(
            config.auto_open_delay_seconds,
            DEFAULT_AUTO_OPEN_DELAY_SECONDS
        );
        assert!(!config.pause_queue_after_dodge);
        assert!(!config.discord_prompt_seen);
        assert_eq!(config.player_click_action, "details");
        assert_eq!(config.counters_provider, "lolalytics");
        assert!(!config.auto_requeue && !config.requeue_past_estimate);
        assert_eq!(config.requeue_delay_seconds, 30);
        assert_eq!(config.long_queue_requeue_delay_seconds, 30);
        assert_eq!(config.dodge_pause_seconds, 0);
        assert!(!config.auto_ban && !config.auto_pick);
        assert!(config.ban_champions.is_empty() && config.pick_champions.is_empty());
    }

    #[test]
    fn automation_settings_round_trip_with_the_frontend_names() {
        let config: Config = serde_json::from_str(
            r#"{
                "autoOpen": true,
                "autoAccept": true,
                "acceptDelay": 2000,
                "countersProvider": "lolalytics",
                "autoRequeue": true,
                "requeuePastEstimate": true,
                "autoBan": true,
                "banChampions": [64, 421],
                "autoPick": true,
                "pickChampions": { "jungle": [59, 104] }
            }"#,
        )
        .unwrap();
        assert_eq!(config.counters_provider, "lolalytics");
        assert!(config.auto_requeue && config.requeue_past_estimate);
        assert_eq!(config.ban_champions, vec![64, 421]);
        assert_eq!(config.pick_champions["jungle"], vec![59, 104]);
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#""pickChampions":{"jungle":[59,104]}"#));
    }

    fn temp_config_path(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("reveal-config-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("config.json")
    }

    #[test]
    fn corrupt_config_resets_to_defaults_and_keeps_a_backup() {
        let path = temp_config_path("corrupt");
        std::fs::write(&path, "{\"autoOpen\": tru").unwrap();

        let config = load_from(&path).unwrap();

        assert_eq!(config.multi_provider, "opgg");
        assert!(config.auto_open);
        assert_eq!(
            std::fs::read_to_string(path.with_extension("json.bad")).unwrap(),
            "{\"autoOpen\": tru"
        );
        let saved: Config = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved.multi_provider, "opgg");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn missing_config_is_created_and_saved_values_round_trip() {
        let path = temp_config_path("missing");

        let mut config = load_from(&path).unwrap();
        assert!(path.exists());

        config.multi_provider = "fow".to_string();
        write_atomic(&path, &config).unwrap();
        assert_eq!(load_from(&path).unwrap().multi_provider, "fow");
        assert!(!path.with_extension("json.tmp").exists());
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn discord_prompt_seen_round_trips_with_the_frontend_name() {
        let config: Config = serde_json::from_str(
            r#"{
                "autoOpen": true,
                "autoAccept": false,
                "acceptDelay": 2000,
                "multiProvider": "opgg",
                "discordPromptSeen": true
            }"#,
        )
        .unwrap();
        assert!(config.discord_prompt_seen);
        assert!(serde_json::to_string(&config)
            .unwrap()
            .contains(r#""discordPromptSeen":true"#));
    }

    #[test]
    fn removed_providers_fall_back_to_opgg() {
        for provider in ["deeplol", "tracker"] {
            let mut config = Config {
                multi_provider: provider.to_string(),
                ..Config::default()
            };
            assert!(config.normalize_provider());
            assert_eq!(config.multi_provider, "opgg");
        }

        let mut config = Config {
            multi_provider: "fow".to_string(),
            ..Config::default()
        };
        assert!(!config.normalize_provider());
        assert_eq!(config.multi_provider, "fow");
    }
}
