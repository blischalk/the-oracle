use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::keychain::keychain_service::KeychainService;
use crate::providers::anthropic::AnthropicProvider;
use crate::providers::google_gemini::GoogleGeminiProvider;
use crate::providers::llm_provider::{
    ChatMessage, LlmError, LlmProvider, LlmResponse, ModelDescriptor,
};
use crate::providers::microsoft_copilot::MicrosoftCopilotProvider;
use crate::providers::ollama::OllamaProvider;
use crate::providers::openai::OpenAiProvider;

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

        let api_key = if provider_id == "ollama" {
            String::new()
        } else {
            self.keychain
                .get_api_key(provider_id)
                .map_err(|error| LlmError::ProviderError(error.to_string()))?
                .ok_or(LlmError::InvalidApiKey)?
        };

        provider.send_message(messages, model_id, &api_key).await
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
}
