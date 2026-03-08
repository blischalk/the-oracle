use tauri::State;

use crate::domain::campaign::{Campaign, CampaignState, Message};
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
pub fn get_campaign(id: String, state: State<AppState>) -> Result<Option<Campaign>, String> {
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
pub fn delete_campaign(id: String, state: State<AppState>) -> Result<(), String> {
    state
        .campaign_service
        .delete_campaign(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_campaign_name(
    id: String,
    name: String,
    state: State<AppState>,
) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Campaign name cannot be empty".to_string());
    }
    state
        .campaign_service
        .update_campaign_name(&id, name)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_messages(campaign_id: String, state: State<AppState>) -> Result<Vec<Message>, String> {
    state
        .campaign_service
        .get_messages(&campaign_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_campaign_state(
    campaign_id: String,
    state: State<AppState>,
) -> Result<CampaignState, String> {
    state
        .campaign_service
        .get_campaign_state(&campaign_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_rpg_system(
    id: String,
    state: State<AppState>,
) -> Result<Option<crate::domain::rpg_system::RpgSystem>, String> {
    Ok(state.rpg_registry.get(&id).cloned())
}
