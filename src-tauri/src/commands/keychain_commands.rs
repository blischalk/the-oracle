use tauri::State;

use crate::AppState;

#[tauri::command]
pub fn save_api_key(
    provider_id: String,
    key: String,
    state: State<AppState>,
) -> Result<(), String> {
    let key = key.trim();
    state
        .keychain_service
        .save_api_key(&provider_id, key)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_api_key(provider_id: String, state: State<AppState>) -> Result<Option<String>, String> {
    state
        .keychain_service
        .get_api_key(&provider_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_api_key(provider_id: String, state: State<AppState>) -> Result<(), String> {
    state
        .keychain_service
        .delete_api_key(&provider_id)
        .map_err(|error| error.to_string())
}
