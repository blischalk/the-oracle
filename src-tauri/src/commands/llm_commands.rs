use chrono::Utc;
use tauri::{AppHandle, State};

use crate::domain::campaign::{CampaignState, Message, MessageRole};
use crate::providers::llm_provider::{ChatMessage, LlmResponse};
use crate::services::campaign_service::GreetingKind;
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
    let user_domain_message =
        Message::new(campaign_id.clone(), MessageRole::User, user_message.clone());

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

    let tools = crate::tools::definitions::build_tool_definitions(rpg_system);

    let mut campaign_state = state
        .campaign_service
        .get_campaign_state(&campaign_id)
        .map_err(|e| e.to_string())?;

    let context_messages = state
        .campaign_service
        .build_llm_context(&campaign_id, rpg_system)
        .map_err(|error| error.to_string())?;

    let llm_response = state
        .llm_service
        .send_message_with_tool_loop(
            &provider_id,
            &model_id,
            context_messages,
            tools,
            &mut campaign_state,
            rpg_system,
        )
        .await
        .map_err(|error| error.to_string())?;

    // Save updated campaign state (character sheet changes from tools)
    state
        .campaign_service
        .save_campaign_state(&campaign_state)
        .map_err(|e| e.to_string())?;

    let total_tokens = llm_response.input_tokens + llm_response.output_tokens;
    let assistant_domain_message = Message::new(
        campaign_id,
        MessageRole::Assistant,
        llm_response.content.clone(),
    )
    .with_token_count(total_tokens);

    state
        .campaign_service
        .save_message(&assistant_domain_message)
        .map_err(|error| error.to_string())?;

    Ok(llm_response)
}

/// Requests an opening message from the Game Master: character-creation guidance for new campaigns,
/// or a recap and next-decision prompt when resuming.
#[tauri::command]
pub async fn request_gm_greeting(
    campaign_id: String,
    kind: String,
    provider_id: String,
    model_id: String,
    state: State<'_, AppState>,
) -> Result<LlmResponse, String> {
    let greeting_kind = match kind.as_str() {
        "new" => GreetingKind::NewCampaign,
        "resume" => GreetingKind::ResumeCampaign,
        other => {
            return Err(format!(
                "Unknown greeting kind: {other}. Use 'new' or 'resume'."
            ))
        }
    };

    let rpg_system_id = state
        .campaign_service
        .get_campaign(&campaign_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Campaign not found: {campaign_id}"))?
        .rpg_system_id;

    let rpg_system = state
        .rpg_registry
        .get(&rpg_system_id)
        .ok_or_else(|| format!("RPG system not found: {rpg_system_id}"))?;

    let tools = crate::tools::definitions::build_tool_definitions(rpg_system);

    let mut campaign_state = state
        .campaign_service
        .get_campaign_state(&campaign_id)
        .map_err(|e| e.to_string())?;

    let context_messages = state
        .campaign_service
        .build_greeting_context(&campaign_id, rpg_system, greeting_kind)
        .map_err(|e| e.to_string())?;

    let llm_response = state
        .llm_service
        .send_message_with_tool_loop(
            &provider_id,
            &model_id,
            context_messages,
            tools,
            &mut campaign_state,
            rpg_system,
        )
        .await
        .map_err(|e| e.to_string())?;

    // Save updated campaign state (character sheet changes from tools)
    state
        .campaign_service
        .save_campaign_state(&campaign_state)
        .map_err(|e| e.to_string())?;

    let total_tokens = llm_response.input_tokens + llm_response.output_tokens;
    let assistant_message = Message::new(
        campaign_id,
        MessageRole::Assistant,
        llm_response.content.clone(),
    )
    .with_token_count(total_tokens);

    state
        .campaign_service
        .save_message(&assistant_message)
        .map_err(|e| e.to_string())?;

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
    let key = key.trim();
    if key.is_empty() {
        return Err("API key cannot be empty.".to_string());
    }
    state
        .llm_service
        .validate_key(&provider_id, key)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn suggest_campaign_name(
    campaign_id: String,
    provider_id: String,
    model_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let messages = state
        .campaign_service
        .get_messages(&campaign_id)
        .map_err(|e| e.to_string())?;

    if messages.len() < 2 {
        return Ok(());
    }

    let excerpt: String = messages
        .iter()
        .take(6)
        .map(|m| {
            let role = match m.role {
                MessageRole::User => "Player",
                MessageRole::Assistant => "Game Master",
                MessageRole::System => "System",
            };
            format!("{role}: {}", m.content)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        "Based on this opening of an RPG adventure, suggest a short campaign title (2 to 6 words). \
         Reply with only the title, no quotes or punctuation.\n\n{excerpt}"
    );

    let chat_message = ChatMessage::user(prompt);

    let response = state
        .llm_service
        .send_message(&provider_id, &model_id, vec![chat_message])
        .await
        .map_err(|e| e.to_string())?;

    let name = response
        .content
        .lines()
        .next()
        .unwrap_or(&response.content)
        .trim()
        .trim_matches(|c: char| c == '"' || c == '\'');

    if name.is_empty() || name.len() > 100 {
        return Ok(());
    }

    state
        .campaign_service
        .update_campaign_name(&campaign_id, name)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Extracts the player character's name and stats from the campaign conversation
/// and saves them to campaign state so the profile UI can display them.
#[tauri::command]
pub async fn extract_character_data(
    campaign_id: String,
    provider_id: String,
    model_id: String,
    state: State<'_, AppState>,
) -> Result<CampaignState, String> {
    let campaign = state
        .campaign_service
        .get_campaign(&campaign_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Campaign not found: {campaign_id}"))?;

    let rpg_system = state
        .rpg_registry
        .get(&campaign.rpg_system_id)
        .ok_or_else(|| format!("RPG system not found: {}", campaign.rpg_system_id))?;

    let messages = state
        .campaign_service
        .get_messages(&campaign_id)
        .map_err(|e| e.to_string())?;

    if messages.is_empty() {
        return state
            .campaign_service
            .get_campaign_state(&campaign_id)
            .map_err(|e| e.to_string());
    }

    let field_names: Vec<&str> = rpg_system
        .character_fields
        .iter()
        .map(|f| f.name.as_str())
        .collect();
    let field_list = field_names.join(", ");

    let conversation: String = messages
        .iter()
        .take(30)
        .map(|m| {
            let role = match m.role {
                MessageRole::User => "Player",
                MessageRole::Assistant => "Game Master",
                MessageRole::System => "System",
            };
            format!("{role}: {}", m.content)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        "From the RPG conversation below, extract the player character's current name and stats. \
         Reply with ONLY a single JSON object, no markdown, no code block, no explanation. \
         Use exactly these keys where known: {field_list}. \
         Use character_name for the character's name. \
         Omit any key whose value is not stated in the conversation. \
         Numbers must be numeric (e.g. 14 not \"14\").\n\nConversation:\n{conversation}"
    );

    let chat_message = ChatMessage::user(prompt);

    let response = state
        .llm_service
        .send_message(&provider_id, &model_id, vec![chat_message])
        .await
        .map_err(|e| e.to_string())?;

    let json_str = response
        .content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let parsed: serde_json::Map<String, serde_json::Value> = match serde_json::from_str(json_str) {
        Ok(serde_json::Value::Object(m)) => m,
        Ok(_) => {
            return state
                .campaign_service
                .get_campaign_state(&campaign_id)
                .map_err(|e| e.to_string())
        }
        Err(_) => {
            return state
                .campaign_service
                .get_campaign_state(&campaign_id)
                .map_err(|e| e.to_string())
        }
    };

    let valid_keys: std::collections::HashSet<&str> =
        rpg_system.character_fields.iter().map(|f| f.name.as_str()).collect();
    let mut character_data = serde_json::Map::new();
    for (k, v) in parsed {
        if valid_keys.contains(k.as_str()) {
            character_data.insert(k, v);
        }
    }

    if character_data.is_empty() {
        return state
            .campaign_service
            .get_campaign_state(&campaign_id)
            .map_err(|e| e.to_string());
    }

    let mut current_state = state
        .campaign_service
        .get_campaign_state(&campaign_id)
        .map_err(|e| e.to_string())?;

    if let Some(existing) = current_state.character_data.as_object_mut() {
        for (k, v) in character_data {
            existing.insert(k, v);
        }
    } else {
        current_state.character_data = serde_json::Value::Object(character_data);
    }

    current_state.updated_at = Utc::now();
    state
        .campaign_service
        .save_campaign_state(&current_state)
        .map_err(|e| e.to_string())?;

    Ok(current_state)
}
