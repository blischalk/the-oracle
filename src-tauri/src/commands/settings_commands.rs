use tauri::State;

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
