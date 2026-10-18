use crate::utils::MULTI_PROVIDERS;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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
