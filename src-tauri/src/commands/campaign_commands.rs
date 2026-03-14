use serde::Serialize;
use tauri::State;

use crate::domain::campaign::{Campaign, CampaignState, Message};
use crate::domain::rpg_system::RpgSystem;
use crate::AppState;

const PAGE_SIZE: usize = 50;

#[derive(Serialize)]
pub struct MessagesPage {
    pub messages: Vec<Message>,
    pub has_more: bool,
}

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
pub fn get_messages_page(
    campaign_id: String,
    before_created_at: Option<String>,
    state: State<AppState>,
) -> Result<MessagesPage, String> {
    let service = &state.campaign_service;
    let (messages, has_more) = match before_created_at.as_deref() {
        None => service.get_messages_recent(&campaign_id, PAGE_SIZE),
        Some(ts) => service.get_messages_before(&campaign_id, ts, PAGE_SIZE),
    }
    .map_err(|e| e.to_string())?;
    Ok(MessagesPage { messages, has_more })
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
pub fn get_rpg_system(id: String, state: State<AppState>) -> Result<Option<RpgSystem>, String> {
    Ok(state.rpg_registry.get(&id).cloned())
}

#[tauri::command]
pub fn list_rpg_systems(state: State<AppState>) -> Result<Vec<RpgSystem>, String> {
    Ok(state.rpg_registry.list_all().into_iter().cloned().collect())
}

#[tauri::command]
pub fn patch_character_data(
    campaign_id: String,
    patch: serde_json::Value,
    state: State<AppState>,
) -> Result<CampaignState, String> {
    state
        .campaign_service
        .patch_character_data(&campaign_id, patch)
        .map_err(|error| error.to_string())
}
