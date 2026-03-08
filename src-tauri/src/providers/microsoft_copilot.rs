use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::llm_provider::{ChatMessage, LlmError, LlmProvider, LlmResponse, ModelDescriptor};

const COPILOT_API_BASE: &str =
    "https://api.cognitive.microsoft.com/openai/deployments/{model}/chat/completions?api-version=2024-02-01";
const CONTEXT_MAX_TOKENS: u32 = 4096;
const VALIDATION_MAX_TOKENS: u32 = 1;

pub struct MicrosoftCopilotProvider {
    client: Client,
    base_url: String,
}

impl MicrosoftCopilotProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: COPILOT_API_BASE.to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    fn build_url(&self, model_id: &str) -> String {
        self.base_url.replace("{model}", model_id)
    }
}

impl Default for MicrosoftCopilotProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
struct CopilotRequest {
    messages: Vec<CopilotMessage>,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct CopilotMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct CopilotResponse {
    choices: Vec<CopilotChoice>,
    usage: CopilotUsage,
}

#[derive(Debug, Deserialize)]
struct CopilotChoice {
    message: CopilotMessage,
}

#[derive(Debug, Deserialize)]
struct CopilotUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

fn build_copilot_messages(messages: Vec<ChatMessage>) -> Vec<CopilotMessage> {
    messages
        .into_iter()
        .map(|message| CopilotMessage {
            role: message.role,
            content: message.content,
        })
        .collect()
}

#[async_trait]
impl LlmProvider for MicrosoftCopilotProvider {
    fn provider_id(&self) -> &str {
        "microsoft_copilot"
    }

    fn display_name(&self) -> &str {
        "Microsoft Copilot"
    }

    fn available_models(&self) -> Vec<ModelDescriptor> {
        vec![
            ModelDescriptor {
                id: "gpt-4o".to_string(),
                display_name: "GPT-4o (Copilot)".to_string(),
                context_window: 128_000,
            },
            ModelDescriptor {
                id: "gpt-4".to_string(),
                display_name: "GPT-4 (Copilot)".to_string(),
                context_window: 8_192,
            },
        ]
    }

    async fn send_message(
        &self,
        messages: Vec<ChatMessage>,
        model_id: &str,
        api_key: &str,
    ) -> Result<LlmResponse, LlmError> {
        let url = self.build_url(model_id);
        let request_body = CopilotRequest {
            messages: build_copilot_messages(messages),
            max_tokens: CONTEXT_MAX_TOKENS,
        };

        let http_response = self
            .client
            .post(&url)
            .header("api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|error| LlmError::NetworkError(error.to_string()))?;

        let status = http_response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(LlmError::InvalidApiKey);
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(LlmError::RateLimited);
        }

        if !status.is_success() {
            let error_body = http_response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!(
                "HTTP {status}: {error_body}"
            )));
        }

        let copilot_response: CopilotResponse = http_response
            .json()
            .await
            .map_err(|error| LlmError::ProviderError(error.to_string()))?;

        let content = copilot_response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            input_tokens: copilot_response.usage.prompt_tokens,
            output_tokens: copilot_response.usage.completion_tokens,
        })
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool, LlmError> {
        let url = self.build_url("gpt-4o");
        let request_body = CopilotRequest {
            messages: vec![CopilotMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            max_tokens: VALIDATION_MAX_TOKENS,
        };

        let http_response = self
            .client
            .post(&url)
            .header("api-key", key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|error| LlmError::NetworkError(error.to_string()))?;

        let status = http_response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Ok(false);
        }

        Ok(status.is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn successful_copilot_response() -> serde_json::Value {
        serde_json::json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello, adventurer!"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        })
    }

    #[tokio::test]
    async fn send_message_parses_successful_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path_regex(r".*chat/completions.*"))
            .and(header("api-key", "test-key"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(successful_copilot_response()),
            )
            .mount(&mock_server)
            .await;

        let provider = MicrosoftCopilotProvider::with_base_url(format!(
            "{}/openai/deployments/{{model}}/chat/completions?api-version=2024-02-01",
            mock_server.uri()
        ));

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let response = provider
            .send_message(messages, "gpt-4o", "test-key")
            .await
            .expect("send_message should succeed");

        assert_eq!(response.content, "Hello, adventurer!");
        assert_eq!(response.input_tokens, 10);
        assert_eq!(response.output_tokens, 5);
    }

    #[test]
    fn available_models_returns_two_copilot_models() {
        let provider = MicrosoftCopilotProvider::new();
        let models = provider.available_models();
        assert_eq!(models.len(), 2);
        assert!(models.iter().any(|m| m.id == "gpt-4o"));
        assert!(models.iter().any(|m| m.id == "gpt-4"));
    }
}
