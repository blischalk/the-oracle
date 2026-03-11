use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::campaign::CampaignState;
use crate::domain::rpg_system::RpgSystem;
use crate::keychain::keychain_service::KeychainService;
use crate::providers::anthropic::AnthropicProvider;
use crate::providers::google_gemini::GoogleGeminiProvider;
use crate::providers::llm_provider::{
    ChatMessage, LlmError, LlmProvider, LlmResponse, ModelDescriptor, ToolDefinition, ToolResult,
};
use crate::providers::microsoft_copilot::MicrosoftCopilotProvider;
use crate::providers::ollama::OllamaProvider;
use crate::providers::openai::OpenAiProvider;
use crate::providers::opencode::OpenCodeProvider;

const MAX_TOOL_ITERATIONS: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub display_name: String,
    pub models: Vec<ModelDescriptor>,
}

pub struct LlmService {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    keychain: Arc<KeychainService>,
}

impl LlmService {
    pub fn new(keychain: Arc<KeychainService>) -> Self {
        let mut providers: HashMap<String, Arc<dyn LlmProvider>> = HashMap::new();

        let anthropic = Arc::new(AnthropicProvider::new());
        providers.insert(anthropic.provider_id().to_string(), anthropic);

        let openai = Arc::new(OpenAiProvider::new());
        providers.insert(openai.provider_id().to_string(), openai);

        let gemini = Arc::new(GoogleGeminiProvider::new());
        providers.insert(gemini.provider_id().to_string(), gemini);

        let copilot = Arc::new(MicrosoftCopilotProvider::new());
        providers.insert(copilot.provider_id().to_string(), copilot);

        let ollama = Arc::new(OllamaProvider::new());
        providers.insert(ollama.provider_id().to_string(), ollama);

        let opencode = Arc::new(OpenCodeProvider::new());
        providers.insert(opencode.provider_id().to_string(), opencode);

        Self {
            providers,
            keychain,
        }
    }

    pub async fn send_message(
        &self,
        provider_id: &str,
        model_id: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<LlmResponse, LlmError> {
        let provider = self.find_provider(provider_id)?;
        let api_key = self.get_api_key_for_provider(provider_id)?;
        provider.send_message(messages, model_id, &api_key).await
    }

    pub async fn send_message_with_tool_loop(
        &self,
        provider_id: &str,
        model_id: &str,
        mut messages: Vec<ChatMessage>,
        tools: Vec<ToolDefinition>,
        campaign_state: &mut CampaignState,
        rpg_system: &RpgSystem,
    ) -> Result<LlmResponse, LlmError> {
        let provider = self.find_provider(provider_id)?;
        let api_key = self.get_api_key_for_provider(provider_id)?;

        // If provider doesn't support tools, fall back to plain send
        if !provider.supports_tools() || tools.is_empty() {
            return self.send_message(provider_id, model_id, messages).await;
        }

        let mut total_input_tokens = 0u32;
        let mut total_output_tokens = 0u32;
        let mut final_text = String::new();

        for _ in 0..MAX_TOOL_ITERATIONS {
            let turn = provider
                .send_message_with_tools(messages.clone(), &tools, model_id, &api_key)
                .await?;

            total_input_tokens += turn.input_tokens;
            total_output_tokens += turn.output_tokens;

            if !turn.text.is_empty() {
                final_text = turn.text.clone();
            }

            if turn.is_final() {
                break;
            }

            // Append the assistant turn (with tool calls) to the message history
            messages.push(ChatMessage::assistant_with_tool_calls(
                turn.text.clone(),
                turn.tool_calls.clone(),
            ));

            // Execute each tool call
            let mut executor = crate::tools::executor::ToolExecutor {
                campaign_state,
                rpg_system,
            };
            let results: Vec<ToolResult> = turn
                .tool_calls
                .iter()
                .map(|call| executor.execute(call))
                .collect();

            // Append the tool results
            messages.push(ChatMessage::tool_results_message(results));
        }

        Ok(LlmResponse {
            content: final_text,
            input_tokens: total_input_tokens,
            output_tokens: total_output_tokens,
        })
    }

    pub fn available_providers(&self) -> Vec<ProviderInfo> {
        self.providers
            .values()
            .map(|provider| ProviderInfo {
                id: provider.provider_id().to_string(),
                display_name: provider.display_name().to_string(),
                models: provider.available_models(),
            })
            .collect()
    }

    pub async fn validate_key(&self, provider_id: &str, key: &str) -> Result<bool, LlmError> {
        let provider = self.find_provider(provider_id)?;
        provider.validate_api_key(key).await
    }

    fn find_provider(&self, provider_id: &str) -> Result<&Arc<dyn LlmProvider>, LlmError> {
        self.providers
            .get(provider_id)
            .ok_or_else(|| LlmError::ModelNotFound(provider_id.to_string()))
    }

    fn get_api_key_for_provider(&self, provider_id: &str) -> Result<String, LlmError> {
        if provider_id == "ollama" {
            return Ok(String::new());
        }
        self.keychain
            .get_api_key(provider_id)
            .map_err(|e| LlmError::ProviderError(e.to_string()))?
            .ok_or(LlmError::InvalidApiKey)
    }
}
