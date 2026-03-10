use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::llm_provider::{ChatMessage, LlmError, LlmProvider, LlmResponse, ModelDescriptor};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const CONTEXT_MAX_TOKENS: u32 = 4096;
const VALIDATION_MAX_TOKENS: u32 = 1;

pub struct OpenAiProvider {
    client: Client,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: OPENAI_API_URL.to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
}

impl Default for OpenAiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

fn build_openai_messages(messages: Vec<ChatMessage>) -> Vec<OpenAiMessage> {
    messages
        .into_iter()
        .map(|message| OpenAiMessage {
            role: message.role,
            content: message.content,
        })
        .collect()
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn provider_id(&self) -> &str {
        "openai"
    }

    fn display_name(&self) -> &str {
        "OpenAI"
    }

    fn available_models(&self) -> Vec<ModelDescriptor> {
        vec![
            ModelDescriptor {
                id: "gpt-4o".to_string(),
                display_name: "GPT-4o".to_string(),
                context_window: 128_000,
            },
            ModelDescriptor {
                id: "gpt-4o-mini".to_string(),
                display_name: "GPT-4o Mini".to_string(),
                context_window: 128_000,
            },
            ModelDescriptor {
                id: "gpt-4.1".to_string(),
                display_name: "GPT-4.1".to_string(),
                context_window: 1_047_576,
            },
            ModelDescriptor {
                id: "o4-mini".to_string(),
                display_name: "o4-mini".to_string(),
                context_window: 200_000,
            },
        ]
    }

    async fn send_message(
        &self,
        messages: Vec<ChatMessage>,
        model_id: &str,
        api_key: &str,
    ) -> Result<LlmResponse, LlmError> {
        let request_body = OpenAiRequest {
            model: model_id.to_string(),
            messages: build_openai_messages(messages),
            max_tokens: CONTEXT_MAX_TOKENS,
        };

        let http_response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {api_key}"))
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

        let openai_response: OpenAiResponse = http_response
            .json()
            .await
            .map_err(|error| LlmError::ProviderError(error.to_string()))?;

        let content = openai_response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            input_tokens: openai_response.usage.prompt_tokens,
            output_tokens: openai_response.usage.completion_tokens,
        })
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool, LlmError> {
        let validation_messages = vec![OpenAiMessage {
            role: "user".to_string(),
            content: "Hi".to_string(),
        }];

        let request_body = OpenAiRequest {
            model: "gpt-4o-mini".to_string(),
            messages: validation_messages,
            max_tokens: VALIDATION_MAX_TOKENS,
        };

        let http_response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {key}"))
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
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn successful_openai_response() -> serde_json::Value {
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
            .and(path("/v1/chat/completions"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(successful_openai_response()),
            )
            .mount(&mock_server)
            .await;

        let provider =
            OpenAiProvider::with_base_url(format!("{}/v1/chat/completions", mock_server.uri()));

        let messages = vec![ChatMessage::user("Hello")];

        let response = provider
            .send_message(messages, "gpt-4o", "test-key")
            .await
            .expect("send_message should succeed");

        assert_eq!(response.content, "Hello, adventurer!");
        assert_eq!(response.input_tokens, 10);
        assert_eq!(response.output_tokens, 5);
    }

    #[test]
    fn available_models_returns_four_models() {
        let provider = OpenAiProvider::new();
        let models = provider.available_models();
        assert_eq!(models.len(), 4);
        assert!(models.iter().any(|m| m.id == "gpt-4o"));
        assert!(models.iter().any(|m| m.id == "gpt-4o-mini"));
        assert!(models.iter().any(|m| m.id == "gpt-4.1"));
        assert!(models.iter().any(|m| m.id == "o4-mini"));
    }
}
