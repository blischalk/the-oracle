use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::domain::settings::AppSettings;
use crate::AppState;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    state
        .settings_service
        .get_settings()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_settings(settings: AppSettings, state: State<AppState>) -> Result<(), String> {
    state
        .settings_service
        .save_settings(&settings)
        .map_err(|error| error.to_string())
}

/// Creates the user RPG systems directory if it doesn't exist, then opens it
/// in the OS file manager so the user can drop in custom YAML files.
#[tauri::command]
pub fn open_user_systems_folder(app: AppHandle) -> Result<String, String> {
    let user_systems_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("rpg-systems");

    std::fs::create_dir_all(&user_systems_dir).map_err(|e| e.to_string())?;

    let path_str = user_systems_dir.to_string_lossy().to_string();
    app.opener()
        .open_path(&path_str, None::<&str>)
        .map_err(|e| e.to_string())?;

    Ok(path_str)
}
