use tauri::State;

use crate::domain::campaign::{Campaign, Message};
use crate::AppState;

#[tauri::command]
pub fn list_campaigns(state: State<AppState>) -> Result<Vec<Campaign>, String> {
    state
        .campaign_service
        .list_campaigns()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_campaign(
    name: String,
    rpg_system_id: String,
    state: State<AppState>,
) -> Result<Campaign, String> {
    state
        .campaign_service
        .create_campaign(&name, &rpg_system_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_campaign(
    id: String,
    state: State<AppState>,
) -> Result<Option<Campaign>, String> {
    state
        .campaign_service
        .get_campaign(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn archive_campaign(id: String, state: State<AppState>) -> Result<(), String> {
    state
        .campaign_service
        .archive_campaign(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_messages(
    campaign_id: String,
    state: State<AppState>,
) -> Result<Vec<Message>, String> {
    state
        .campaign_service
        .get_messages(&campaign_id)
        .map_err(|error| error.to_string())
}
