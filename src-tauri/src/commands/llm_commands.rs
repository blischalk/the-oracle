use tauri::{AppHandle, State};

use crate::domain::campaign::{Message, MessageRole};
use crate::providers::llm_provider::LlmResponse;
use crate::services::llm_service::ProviderInfo;
use crate::AppState;

#[tauri::command]
pub async fn send_chat_message(
    campaign_id: String,
    user_message: String,
    provider_id: String,
    model_id: String,
    state: State<'_, AppState>,
    _app: AppHandle,
) -> Result<LlmResponse, String> {
    let user_domain_message = Message::new(
        campaign_id.clone(),
        MessageRole::User,
        user_message.clone(),
    );

    state
        .campaign_service
        .save_message(&user_domain_message)
        .map_err(|error| error.to_string())?;

    let rpg_system_id = state
        .campaign_service
        .get_campaign(&campaign_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Campaign not found: {campaign_id}"))?
        .rpg_system_id;

    let rpg_system = state
        .rpg_registry
        .get(&rpg_system_id)
        .ok_or_else(|| format!("RPG system not found: {rpg_system_id}"))?;

    let context_messages = state
        .campaign_service
        .build_llm_context(&campaign_id, rpg_system)
        .map_err(|error| error.to_string())?;

    let llm_response = state
        .llm_service
        .send_message(&provider_id, &model_id, context_messages)
        .await
        .map_err(|error| error.to_string())?;

    let total_tokens = llm_response.input_tokens + llm_response.output_tokens;
    let assistant_domain_message =
        Message::new(campaign_id, MessageRole::Assistant, llm_response.content.clone())
            .with_token_count(total_tokens);

    state
        .campaign_service
        .save_message(&assistant_domain_message)
        .map_err(|error| error.to_string())?;

    Ok(llm_response)
}

#[tauri::command]
pub fn list_providers(state: State<AppState>) -> Result<Vec<ProviderInfo>, String> {
    Ok(state.llm_service.available_providers())
}

#[tauri::command]
pub async fn validate_api_key(
    provider_id: String,
    key: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    state
        .llm_service
        .validate_key(&provider_id, &key)
        .await
        .map_err(|error| error.to_string())
}
