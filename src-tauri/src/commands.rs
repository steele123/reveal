use crate::{
    app_state::{Lcu, QueuePause},
    config::{self, AppConfig, Config},
    legacy_install::{self, LegacyInstall},
    lobby::get_lobby_info,
    region::RegionInfo,
    utils::display_champ_select,
};
use shaco::rest::{LCUClientInfo, RESTClient};
use tauri::{AppHandle, Manager};

type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub async fn app_ready(
    app_handle: AppHandle,
    lcu: tauri::State<'_, Lcu>,
    cfg: tauri::State<'_, AppConfig>,
) -> CommandResult<Config> {
    log_info!("Frontend reported ready");
    let lcu = lcu.0.lock().await;
    let cfg = cfg.0.lock().await;

    log_info!("Initial League connection state: {}", lcu.connected);
    log_info!(
        "Active settings: auto_open={}, auto_open_delay_seconds={}, auto_accept={}, accept_delay_ms={}, provider={}",
        cfg.auto_open,
        cfg.auto_open_delay_seconds,
        cfg.auto_accept,
        cfg.accept_delay,
        cfg.multi_provider
    );

    app_handle
        .emit_all("lcu_state_update", lcu.connected)
        .map_err(|error| error.to_string())?;

    let pause = app_handle.state::<QueuePause>();
    let paused = pause.0.lock().await.paused();
    app_handle
        .emit_all("queue_pause_update", paused)
        .map_err(|error| error.to_string())?;
    // Catch up on anything sent before the window was listening, e.g. a Champ Select that was
    // already running when Reveal opened.
    crate::events::replay(&app_handle);

    Ok(cfg.clone())
}

#[tauri::command]
pub async fn get_lcu_state(lcu: tauri::State<'_, Lcu>) -> CommandResult<bool> {
    let lcu = lcu.0.lock().await;
    Ok(lcu.connected)
}

#[tauri::command]
pub async fn get_config(cfg: tauri::State<'_, AppConfig>) -> CommandResult<Config> {
    let cfg = cfg.0.lock().await;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn set_config(
    cfg: tauri::State<'_, AppConfig>,
    new_cfg: Config,
    app_handle: AppHandle,
) -> CommandResult<()> {
    log_info!(
        "Saving settings: auto_open={}, auto_open_delay_seconds={}, auto_accept={}, accept_delay_ms={}, provider={}",
        new_cfg.auto_open,
        new_cfg.auto_open_delay_seconds,
        new_cfg.auto_accept,
        new_cfg.accept_delay,
        new_cfg.multi_provider
    );
    let mut new_cfg = new_cfg;
    new_cfg.normalize_provider();
    let mut stored_config = cfg.0.lock().await;
    config::save(&app_handle, &new_cfg)
        .await
        .map_err(|error| error.to_string())?;
    *stored_config = new_cfg;
    let pause_enabled = stored_config.pause_queue_after_dodge;
    drop(stored_config);

    if !pause_enabled {
        let pause = app_handle.state::<QueuePause>();
        let pause_changed = pause.0.lock().await.disable();
        if let Some(paused) = pause_changed {
            app_handle
                .emit_all("queue_pause_update", paused)
                .map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn open_opgg_link(app_handle: AppHandle) -> CommandResult<()> {
    log_info!("Manual multi-search open requested");
    let lcu_info = current_lcu_info(&app_handle).await?;
    let app_client = RESTClient::new(lcu_info, false).map_err(|error| error.to_string())?;

    let config = {
        let config = app_handle.state::<AppConfig>();
        let value = config.0.lock().await.clone();
        value
    };

    let team = get_lobby_info(&app_client)
        .await
        .map_err(|error| error.to_string())?;
    let region_info: RegionInfo = serde_json::from_value(
        app_client
            .get("/riotclient/region-locale".to_string())
            .await
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    let region = &region_info.web();

    display_champ_select(&team, region, &config.multi_provider)
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_lcu_info(lcu: tauri::State<'_, Lcu>) -> CommandResult<LCUClientInfo> {
    let lcu = lcu.0.lock().await;
    lcu.data
        .clone()
        .ok_or_else(|| "League Client is not connected".to_string())
}

/// Every champion's id and name in the client's language, for the ban and pick lists.
#[tauri::command]
pub async fn get_champion_list(app_handle: AppHandle) -> CommandResult<Vec<ChampionName>> {
    let lcu_info = current_lcu_info(&app_handle).await?;
    let remoting_client = RESTClient::new(lcu_info, true).map_err(|error| error.to_string())?;
    let names = crate::champions::get_champion_names(&remoting_client)
        .await
        .map_err(|error| format!("{error:#}"))?;
    let mut champions: Vec<ChampionName> = names
        .into_iter()
        .map(|(id, name)| ChampionName { id, name })
        .collect();
    champions.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(champions)
}

#[derive(serde::Serialize)]
pub struct ChampionName {
    id: i32,
    name: String,
}

/// The player signed in to the League Client, with their ranks, for the idle screen.
#[tauri::command]
pub async fn get_own_profile(app_handle: AppHandle) -> CommandResult<crate::summoner::OwnProfile> {
    let lcu_info = current_lcu_info(&app_handle).await?;
    let remoting_client = RESTClient::new(lcu_info, true).map_err(|error| error.to_string())?;
    crate::summoner::get_own_profile(&remoting_client)
        .await
        .map_err(|error| format!("{error:#}"))
}

async fn current_lcu_info(app_handle: &AppHandle) -> CommandResult<LCUClientInfo> {
    let lcu = app_handle.state::<Lcu>();
    let lcu = lcu.0.lock().await;
    lcu.data
        .clone()
        .ok_or_else(|| "League Client is not connected".to_string())
}

#[tauri::command]
pub async fn get_legacy_install() -> CommandResult<Option<LegacyInstall>> {
    let install = tauri::async_runtime::spawn_blocking(legacy_install::find)
        .await
        .map_err(|error| error.to_string())?;
    if let Some(install) = &install {
        log_info!(
            "Found old MSI install v{} at {}",
            install.version,
            install.location
        );
    }
    Ok(install)
}

#[tauri::command]
pub async fn remove_legacy_install() -> CommandResult<bool> {
    let Some(install) = tauri::async_runtime::spawn_blocking(legacy_install::find)
        .await
        .map_err(|error| error.to_string())?
    else {
        return Ok(true);
    };

    log_info!("Removing old MSI install v{}", install.version);
    tauri::async_runtime::spawn_blocking(move || legacy_install::remove(&install))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_log_folder() -> CommandResult<()> {
    let directory =
        crate::logging::log_directory().ok_or_else(|| "Logging is not available".to_string())?;
    log_info!("Opening the log folder");
    open::that(directory).map_err(|error| error.to_string())
}

/// Puts an enemy pick in another lane, after the user dragged it there.
#[tauri::command]
pub fn set_enemy_role(champion_id: i64, role: String) -> CommandResult<()> {
    let role = crate::matchups::Role::from_position(&role)
        .ok_or_else(|| format!("Unknown role {role}"))?;
    log_info!("Enemy pick {champion_id} moved to {role:?}");
    crate::matchups::set_role_override(champion_id, role);
    Ok(())
}

/// Sends a sample alert to one channel ("desktop", "phone" or "discord") with the settings on
/// screen.
#[tauri::command]
pub async fn send_test_notification(
    app_handle: AppHandle,
    channel: String,
    settings: crate::config::NotificationSettings,
) -> CommandResult<()> {
    let channel = crate::notify::Channel::from_setting(&channel)
        .ok_or_else(|| format!("Unknown notification channel {channel}"))?;
    log_info!("Sending a test notification to {channel:?}");
    crate::notify::send_test(&app_handle, channel, &settings)
        .await
        .map_err(|error| format!("{error:#}"))
}

/// The window's summary of a revealed team (it knows who was seen before), as an alert. The
/// phone and Discord alerts list `players` and link to the team's multi-search.
#[tauri::command]
pub fn notify_lobby_revealed(
    app_handle: AppHandle,
    body: String,
    players: Vec<crate::notify::NoticePlayer>,
    link: Option<String>,
) {
    let notice = crate::notify::Notice::new(
        crate::notify::Alert::LobbyRevealed,
        "Your team is revealed",
        body,
    )
    .with_players(players)
    .with_link(link);
    crate::notify::send(&app_handle, notice);
}

#[tauri::command]
pub fn write_frontend_log(level: String, message: String) {
    let message = crate::logging::sanitize_frontend_message(&message);
    match level.as_str() {
        "error" => log_error!("Frontend: {message}"),
        "warn" => log_warn!("Frontend: {message}"),
        _ => log_info!("Frontend: {message}"),
    }
}

/// Hides the window instead of quitting, and says so, so Reveal doesn't seem to have closed.
#[tauri::command]
pub fn hide_to_tray(window: tauri::Window) {
    if let Err(error) = window.hide() {
        log_error!("Couldn't hide the window: {error}");
        return;
    }
    log_info!("Window hidden; Reveal keeps running in the tray");
    let identifier = &window.app_handle().config().tauri.bundle.identifier;
    if let Err(error) = tauri::api::notification::Notification::new(identifier)
        .title("Reveal is still running")
        .body("It keeps watching League from the tray. Right-click the tray icon to quit.")
        .show()
    {
        log_error!("Couldn't show the tray notice: {error}");
    }
}
