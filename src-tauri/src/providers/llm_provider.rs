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
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub tool_name: String,
    pub content: serde_json::Value,
    #[serde(default)]
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderTurn {
    pub text: String,
    pub tool_calls: Vec<ToolCall>,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl ProviderTurn {
    pub fn is_final(&self) -> bool {
        self.tool_calls.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    #[serde(default)]
    pub tool_results: Vec<ToolResult>,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            ..Default::default()
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            ..Default::default()
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            ..Default::default()
        }
    }

    pub fn assistant_with_tool_calls(text: String, calls: Vec<ToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: text,
            tool_calls: calls,
            ..Default::default()
        }
    }

    pub fn tool_results_message(results: Vec<ToolResult>) -> Self {
        Self {
            role: "tool".to_string(),
            content: String::new(),
            tool_results: results,
            ..Default::default()
        }
    }
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

    fn supports_tools(&self) -> bool {
        true
    }

    async fn send_message_with_tools(
        &self,
        messages: Vec<ChatMessage>,
        tools: &[ToolDefinition],
        model_id: &str,
        api_key: &str,
    ) -> Result<ProviderTurn, LlmError> {
        let _ = tools; // ignored in base implementation
        let response = self.send_message(messages, model_id, api_key).await?;
        Ok(ProviderTurn {
            text: response.content,
            tool_calls: vec![],
            input_tokens: response.input_tokens,
            output_tokens: response.output_tokens,
        })
    }
}
