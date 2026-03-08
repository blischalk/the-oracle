use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub id: String,
    pub display_name: String,
    pub context_window: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub is_final: bool,
}

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("API key is missing or invalid")]
    InvalidApiKey,
    #[error("Rate limit exceeded")]
    RateLimited,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Model not found: {0}")]
    ModelNotFound(String),
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn available_models(&self) -> Vec<ModelDescriptor>;
    async fn send_message(
        &self,
        messages: Vec<ChatMessage>,
        model_id: &str,
        api_key: &str,
    ) -> Result<LlmResponse, LlmError>;
    async fn validate_api_key(&self, key: &str) -> Result<bool, LlmError>;
}
