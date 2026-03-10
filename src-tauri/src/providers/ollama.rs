use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::llm_provider::{ChatMessage, LlmError, LlmProvider, LlmResponse, ModelDescriptor};

const OLLAMA_API_URL: &str = "http://localhost:11434/api/chat";

pub struct OllamaProvider {
    client: Client,
    base_url: String,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: OLLAMA_API_URL.to_string(),
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

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

fn build_ollama_messages(messages: Vec<ChatMessage>) -> Vec<OllamaMessage> {
    messages
        .into_iter()
        .map(|message| OllamaMessage {
            role: message.role,
            content: message.content,
        })
        .collect()
}

fn default_models() -> Vec<ModelDescriptor> {
    vec![
        ModelDescriptor {
            id: "llama3.2:latest".to_string(),
            display_name: "Llama 3.2".to_string(),
            context_window: 32_000,
        },
        ModelDescriptor {
            id: "mistral:latest".to_string(),
            display_name: "Mistral".to_string(),
            context_window: 32_000,
        },
        ModelDescriptor {
            id: "deepseek-r1:latest".to_string(),
            display_name: "DeepSeek R1".to_string(),
            context_window: 32_000,
        },
    ]
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn provider_id(&self) -> &str {
        "ollama"
    }

    fn display_name(&self) -> &str {
        "Ollama (Local)"
    }

    fn available_models(&self) -> Vec<ModelDescriptor> {
        default_models()
    }

    async fn send_message(
        &self,
        messages: Vec<ChatMessage>,
        model_id: &str,
        _api_key: &str,
    ) -> Result<LlmResponse, LlmError> {
        let request_body = OllamaRequest {
            model: model_id.to_string(),
            messages: build_ollama_messages(messages),
            stream: false,
        };

        let http_response = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|error| LlmError::NetworkError(error.to_string()))?;

        let status = http_response.status();

        if !status.is_success() {
            let error_body = http_response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!(
                "HTTP {status}: {error_body}"
            )));
        }

        let ollama_response: OllamaResponse = http_response
            .json()
            .await
            .map_err(|error| LlmError::ProviderError(error.to_string()))?;

        Ok(LlmResponse {
            content: ollama_response.message.content,
            input_tokens: 0,
            output_tokens: 0,
        })
    }

    async fn validate_api_key(&self, _key: &str) -> Result<bool, LlmError> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn successful_ollama_response() -> serde_json::Value {
        serde_json::json!({
            "model": "llama3.2:latest",
            "message": {
                "role": "assistant",
                "content": "Hello, adventurer!"
            },
            "done": true
        })
    }

    #[tokio::test]
    async fn send_message_parses_successful_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(successful_ollama_response()),
            )
            .mount(&mock_server)
            .await;

        let provider =
            OllamaProvider::with_base_url(format!("{}/api/chat", mock_server.uri()));

        let messages = vec![ChatMessage::user("Hello")];

        let response = provider
            .send_message(messages, "llama3.2:latest", "")
            .await
            .expect("send_message should succeed");

        assert_eq!(response.content, "Hello, adventurer!");
        assert_eq!(response.input_tokens, 0);
        assert_eq!(response.output_tokens, 0);
    }

    #[tokio::test]
    async fn validate_api_key_always_returns_true() {
        let provider = OllamaProvider::new();
        let result = provider
            .validate_api_key("")
            .await
            .expect("validate_api_key should not fail");
        assert!(result);
    }

    #[test]
    fn available_models_returns_default_list() {
        let provider = OllamaProvider::new();
        let models = provider.available_models();
        assert_eq!(models.len(), 3);
        assert!(models.iter().any(|m| m.id == "llama3.2:latest"));
        assert!(models.iter().any(|m| m.id == "mistral:latest"));
        assert!(models.iter().any(|m| m.id == "deepseek-r1:latest"));
    }
}
