use crate::{
    app_state::{Dodge, Lcu},
    champ_select::ChampSelectSession,
    config::{self, AppConfig, Config},
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
    println!("App Ready!");
    let lcu = lcu.0.lock().await;
    let cfg = cfg.0.lock().await;

    println!("LCU State: {}", lcu.connected);
    println!("Config: {:?}", cfg);

    app_handle
        .emit_all("lcu_state_update", lcu.connected)
        .map_err(|error| error.to_string())?;

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
    println!("Setting Config: {:?}", new_cfg);
    let mut stored_config = cfg.0.lock().await;
    config::save(&app_handle, &new_cfg)
        .await
        .map_err(|error| error.to_string())?;
    *stored_config = new_cfg;

    Ok(())
}

#[tauri::command]
pub async fn open_opgg_link(app_handle: AppHandle) -> CommandResult<()> {
    println!("Manually opening Multi Link...");
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

    let region = match region_info.web_region.as_str() {
        "SG2" => "SG",
        _ => &region_info.web_region,
    };

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

#[tauri::command]
pub async fn dodge(app_handle: AppHandle) -> CommandResult<()> {
    let lcu_info = current_lcu_info(&app_handle).await?;
    let remoting_client = RESTClient::new(lcu_info, true).map_err(|error| error.to_string())?;

    println!("Attempting to quit champ select...");
    remoting_client
        .post(
            "/lol-login/v1/session/invoke?destination=lcdsServiceProxy&method=call&args=[\"\",\"teambuilder-draft\",\"quitV2\",\"\"]".to_string(),
            serde_json::json!({}),
        )
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn enable_dodge(app_handle: AppHandle) -> CommandResult<()> {
    let lcu_info = current_lcu_info(&app_handle).await?;
    let remoting_client = RESTClient::new(lcu_info, true).map_err(|error| error.to_string())?;

    let dodge_state = app_handle.state::<Dodge>();
    let mut dodge_state = dodge_state.0.lock().await;

    if dodge_state.enabled.is_some() {
        dodge_state.enabled = None;
        return Ok(());
    }

    let champ_select = serde_json::from_value::<ChampSelectSession>(
        remoting_client
            .get("/lol-champ-select/v1/session".to_string())
            .await
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    dodge_state.enabled = Some(champ_select.game_id);
    Ok(())
}

async fn current_lcu_info(app_handle: &AppHandle) -> CommandResult<LCUClientInfo> {
    let lcu = app_handle.state::<Lcu>();
    let lcu = lcu.0.lock().await;
    lcu.data
        .clone()
        .ok_or_else(|| "League Client is not connected".to_string())
}
