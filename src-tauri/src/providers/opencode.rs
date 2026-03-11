use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::llm_provider::{ChatMessage, LlmError, LlmProvider, LlmResponse, ModelDescriptor};

const OPENCODE_COMPLETIONS_URL: &str = "https://opencode.ai/zen/v1/chat/completions";
const OPENCODE_MODELS_URL: &str = "https://opencode.ai/zen/v1/models";

pub struct OpenCodeProvider {
    client: Client,
    completions_url: String,
    models_url: String,
}

impl OpenCodeProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            completions_url: OPENCODE_COMPLETIONS_URL.to_string(),
            models_url: OPENCODE_MODELS_URL.to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(base_url: String) -> Self {
        // base_url pattern: "http://server/zen/v1/chat/completions"
        // models_url replaces the completions suffix with the models path.
        let models_url = base_url.replace("/chat/completions", "/models");
        Self {
            client: Client::new(),
            completions_url: base_url,
            models_url,
        }
    }
}

impl Default for OpenCodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
struct OpenCodeRequest {
    model: String,
    messages: Vec<OpenCodeMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenCodeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenCodeResponse {
    choices: Vec<OpenCodeChoice>,
    usage: OpenCodeUsage,
}

#[derive(Debug, Deserialize)]
struct OpenCodeChoice {
    message: OpenCodeMessage,
}

#[derive(Debug, Deserialize)]
struct OpenCodeUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

fn build_opencode_messages(messages: Vec<ChatMessage>) -> Vec<OpenCodeMessage> {
    messages
        .into_iter()
        .map(|message| OpenCodeMessage {
            role: message.role,
            content: message.content,
        })
        .collect()
}

#[async_trait]
impl LlmProvider for OpenCodeProvider {
    fn provider_id(&self) -> &str {
        "opencode"
    }

    fn display_name(&self) -> &str {
        "OpenCode"
    }

    fn available_models(&self) -> Vec<ModelDescriptor> {
        vec![
            ModelDescriptor {
                id: "big-pickle".to_string(),
                display_name: "Big Pickle (free)".to_string(),
                context_window: 200_000,
            },
            ModelDescriptor {
                id: "minimax-m2.5".to_string(),
                display_name: "MiniMax M2.5".to_string(),
                context_window: 1_000_000,
            },
            ModelDescriptor {
                id: "kimi-k2.5".to_string(),
                display_name: "Kimi K2.5".to_string(),
                context_window: 131_072,
            },
            ModelDescriptor {
                id: "qwen3-coder".to_string(),
                display_name: "Qwen3 Coder".to_string(),
                context_window: 131_072,
            },
        ]
    }

    async fn send_message(
        &self,
        messages: Vec<ChatMessage>,
        model_id: &str,
        api_key: &str,
    ) -> Result<LlmResponse, LlmError> {
        let request_body = OpenCodeRequest {
            model: model_id.to_string(),
            messages: build_opencode_messages(messages),
        };

        let http_response = self
            .client
            .post(&self.completions_url)
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

        let opencode_response: OpenCodeResponse = http_response
            .json()
            .await
            .map_err(|error| LlmError::ProviderError(error.to_string()))?;

        let content = opencode_response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            input_tokens: opencode_response.usage.prompt_tokens,
            output_tokens: opencode_response.usage.completion_tokens,
        })
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool, LlmError> {
        let http_response = self
            .client
            .get(&self.models_url)
            .header("Authorization", format!("Bearer {key}"))
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

    fn successful_response() -> serde_json::Value {
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
            .and(path("/zen/v1/chat/completions"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(successful_response()))
            .mount(&mock_server)
            .await;

        let provider = OpenCodeProvider::with_base_url(format!(
            "{}/zen/v1/chat/completions",
            mock_server.uri()
        ));

        let response = provider
            .send_message(vec![ChatMessage::user("Hello")], "big-pickle", "test-key")
            .await
            .expect("send_message should succeed");

        assert_eq!(response.content, "Hello, adventurer!");
        assert_eq!(response.input_tokens, 10);
        assert_eq!(response.output_tokens, 5);
    }

    #[tokio::test]
    async fn send_message_returns_invalid_key_error_on_401() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/zen/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let provider = OpenCodeProvider::with_base_url(format!(
            "{}/zen/v1/chat/completions",
            mock_server.uri()
        ));

        let err = provider
            .send_message(vec![ChatMessage::user("Hello")], "big-pickle", "bad-key")
            .await
            .expect_err("should fail with invalid key");

        assert!(matches!(err, LlmError::InvalidApiKey));
    }

    #[tokio::test]
    async fn send_message_returns_rate_limited_on_429() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/zen/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;

        let provider = OpenCodeProvider::with_base_url(format!(
            "{}/zen/v1/chat/completions",
            mock_server.uri()
        ));

        let err = provider
            .send_message(vec![ChatMessage::user("Hello")], "big-pickle", "test-key")
            .await
            .expect_err("should fail with rate limited");

        assert!(matches!(err, LlmError::RateLimited));
    }

    #[tokio::test]
    async fn validate_api_key_returns_true_on_200() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/zen/v1/models"))
            .and(header("Authorization", "Bearer valid-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": []
            })))
            .mount(&mock_server)
            .await;

        let provider = OpenCodeProvider::with_base_url(format!(
            "{}/zen/v1/chat/completions",
            mock_server.uri()
        ));

        assert!(provider.validate_api_key("valid-key").await.unwrap());
    }

    #[tokio::test]
    async fn validate_api_key_returns_false_on_401() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/zen/v1/models"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let provider = OpenCodeProvider::with_base_url(format!(
            "{}/zen/v1/chat/completions",
            mock_server.uri()
        ));

        assert!(!provider.validate_api_key("bad-key").await.unwrap());
    }

    #[test]
    fn available_models_returns_four_models() {
        let provider = OpenCodeProvider::new();
        let models = provider.available_models();
        assert_eq!(models.len(), 4);
        assert!(models.iter().any(|m| m.id == "big-pickle"));
        assert!(models.iter().any(|m| m.id == "minimax-m2.5"));
        assert!(models.iter().any(|m| m.id == "kimi-k2.5"));
        assert!(models.iter().any(|m| m.id == "qwen3-coder"));
    }
}
