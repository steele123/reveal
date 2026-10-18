use crate::utils::MULTI_PROVIDERS;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
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

fn default_auto_open_delay_seconds() -> u32 {
    DEFAULT_AUTO_OPEN_DELAY_SECONDS
}

fn default_provider() -> String {
    "opgg".to_string()
}

pub struct AppConfig(pub Mutex<Config>);

impl AppConfig {
    pub fn new(config: Config) -> Self {
        Self(Mutex::new(config))
    }
}

pub fn load(app_handle: &AppHandle) -> Result<Config> {
    let config_dir = app_handle
        .path_resolver()
        .app_config_dir()
        .context("Reveal has no application config directory")?;
    std::fs::create_dir_all(&config_dir).context("failed to create config directory")?;

    let config_path = config_dir.join("config.json");
    log_info!("Loading config from {}", config_path.display());
    if !config_path.exists() {
        let config = Config::default();
        let json = serde_json::to_string(&config).context("failed to serialize default config")?;
        std::fs::write(&config_path, json).context("failed to write default config")?;
        log_info!("Created a default config");
        return Ok(config);
    }

    let json = std::fs::read_to_string(&config_path).context("failed to read config")?;
    let mut config: Config = serde_json::from_str(&json).context("failed to parse config")?;
    if config.normalize_provider() {
        log_warn!("Saved multi-search provider is no longer supported; using OP.GG");
        let json = serde_json::to_string(&config).context("failed to serialize config")?;
        std::fs::write(&config_path, json).context("failed to write config")?;
    }
    log_info!("Config loaded successfully");
    Ok(config)
}

pub async fn save(app_handle: &AppHandle, config: &Config) -> Result<()> {
    let config_dir = app_handle
        .path_resolver()
        .app_config_dir()
        .context("Reveal has no application config directory")?;
    let config_path = config_dir.join("config.json");
    let json = serde_json::to_string(config).context("failed to serialize config")?;

    tokio::fs::write(config_path, json)
        .await
        .context("failed to write config")?;
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
