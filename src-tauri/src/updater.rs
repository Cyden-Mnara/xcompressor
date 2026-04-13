//! Application updater commands and status mapping.

use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;
use url::Url;

use crate::AppUpdateStatus;

fn configured_updater(app: &AppHandle) -> Result<Option<tauri_plugin_updater::Updater>, String> {
    let endpoint = option_env!("XCOMPRESSOR_UPDATER_ENDPOINT")
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let pubkey = option_env!("XCOMPRESSOR_UPDATER_PUBKEY")
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let (Some(endpoint), Some(pubkey)) = (endpoint, pubkey) else {
        return Ok(None);
    };

    app.updater_builder()
        .endpoints(vec![Url::parse(endpoint).map_err(|error| {
            format!("Invalid updater endpoint {endpoint}: {error}")
        })?])
        .map_err(|error| format!("Failed to configure updater endpoints: {error}"))?
        .pubkey(pubkey)
        .build()
        .map(Some)
        .map_err(|error| format!("Failed to build updater: {error}"))
}

fn update_status(
    current_version: String,
    available_version: Option<String>,
    notes: Option<String>,
    pub_date: Option<String>,
    update_ready: bool,
    update_installed: bool,
    message: impl Into<String>,
) -> AppUpdateStatus {
    AppUpdateStatus {
        configured: true,
        current_version,
        available_version,
        notes,
        pub_date,
        update_ready,
        update_installed,
        message: message.into(),
    }
}

fn updater_not_configured_status(app: &AppHandle) -> AppUpdateStatus {
    AppUpdateStatus {
        configured: false,
        current_version: app.package_info().version.to_string(),
        available_version: None,
        notes: None,
        pub_date: None,
        update_ready: false,
        update_installed: false,
        message: "Updater is not configured for this build.".into(),
    }
}

#[tauri::command]
pub(crate) async fn check_for_app_update(app: AppHandle) -> Result<AppUpdateStatus, String> {
    let Some(updater) = configured_updater(&app)? else {
        return Ok(updater_not_configured_status(&app));
    };

    let current_version = app.package_info().version.to_string();
    let update = updater
        .check()
        .await
        .map_err(|error| format!("Failed to check for updates: {error}"))?;

    Ok(match update {
        Some(update) => update_status(
            current_version,
            Some(update.version.clone()),
            update.body.clone(),
            update.date.map(|date| date.to_string()),
            true,
            false,
            format!("Version {} is available.", update.version),
        ),
        None => update_status(
            current_version,
            None,
            None,
            None,
            false,
            false,
            "You already have the latest version.".to_string(),
        ),
    })
}

#[tauri::command]
pub(crate) async fn install_app_update(app: AppHandle) -> Result<AppUpdateStatus, String> {
    let Some(updater) = configured_updater(&app)? else {
        return Ok(updater_not_configured_status(&app));
    };

    let current_version = app.package_info().version.to_string();
    let Some(update) = updater
        .check()
        .await
        .map_err(|error| format!("Failed to check for updates: {error}"))?
    else {
        return Ok(update_status(
            current_version,
            None,
            None,
            None,
            false,
            false,
            "No update is available right now.",
        ));
    };

    let available_version = update.version.clone();
    let notes = update.body.clone();
    let pub_date = update.date.map(|date| date.to_string());

    update
        .download_and_install(|_chunk_length, _content_length| {}, || {})
        .await
        .map_err(|error| format!("Failed to install update {available_version}: {error}"))?;

    Ok(update_status(
        current_version,
        Some(available_version),
        notes,
        pub_date,
        false,
        true,
        "Update installed. Restart the app to use the new version.",
    ))
}
