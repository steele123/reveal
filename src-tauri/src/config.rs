use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub auto_open: bool,
    pub auto_accept: bool,
    pub accept_delay: u32,
    #[serde(default = "default_provider")]
    pub multi_provider: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_open: true,
            auto_accept: false,
            accept_delay: 2_000,
            multi_provider: default_provider(),
        }
    }
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
    if !config_path.exists() {
        let config = Config::default();
        let json = serde_json::to_string(&config).context("failed to serialize default config")?;
        std::fs::write(&config_path, json).context("failed to write default config")?;
        return Ok(config);
    }

    let json = std::fs::read_to_string(&config_path).context("failed to read config")?;
    serde_json::from_str(&json).context("failed to parse config")
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
        .context("failed to write config")
}
