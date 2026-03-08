use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::llm_provider::{ChatMessage, LlmError, LlmProvider, LlmResponse, ModelDescriptor};

const GEMINI_API_BASE: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent";

pub struct GoogleGeminiProvider {
    client: Client,
    base_url: String,
}

impl GoogleGeminiProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: GEMINI_API_BASE.to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    fn build_url(&self, model_id: &str, api_key: &str) -> String {
        let url = self.base_url.replace("{model}", model_id);
        format!("{url}?key={api_key}")
    }
}

impl Default for GoogleGeminiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction>,
    contents: Vec<GeminiContent>,
}

#[derive(Debug, Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: GeminiUsageMetadata,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: u32,
}

fn extract_system_instruction(messages: &[ChatMessage]) -> Option<GeminiSystemInstruction> {
    messages
        .iter()
        .find(|message| message.role == "system")
        .map(|message| GeminiSystemInstruction {
            parts: vec![GeminiPart {
                text: message.content.clone(),
            }],
        })
}

fn map_role_to_gemini(role: &str) -> String {
    if role == "assistant" {
        "model".to_string()
    } else {
        role.to_string()
    }
}

fn build_gemini_contents(messages: Vec<ChatMessage>) -> Vec<GeminiContent> {
    messages
        .into_iter()
        .filter(|message| message.role != "system")
        .map(|message| GeminiContent {
            role: map_role_to_gemini(&message.role),
            parts: vec![GeminiPart {
                text: message.content,
            }],
        })
        .collect()
}

fn build_gemini_request(messages: Vec<ChatMessage>) -> GeminiRequest {
    let system_instruction = extract_system_instruction(&messages);
    let contents = build_gemini_contents(messages);

    GeminiRequest {
        system_instruction,
        contents,
    }
}

#[async_trait]
impl LlmProvider for GoogleGeminiProvider {
    fn provider_id(&self) -> &str {
        "google_gemini"
    }

    fn display_name(&self) -> &str {
        "Google Gemini"
    }

    fn available_models(&self) -> Vec<ModelDescriptor> {
        vec![
            ModelDescriptor {
                id: "gemini-2.5-pro".to_string(),
                display_name: "Gemini 2.5 Pro".to_string(),
                context_window: 1_000_000,
            },
            ModelDescriptor {
                id: "gemini-2.0-flash".to_string(),
                display_name: "Gemini 2.0 Flash".to_string(),
                context_window: 1_000_000,
            },
            ModelDescriptor {
                id: "gemini-2.5-flash".to_string(),
                display_name: "Gemini 2.5 Flash".to_string(),
                context_window: 1_000_000,
            },
        ]
    }

    async fn send_message(
        &self,
        messages: Vec<ChatMessage>,
        model_id: &str,
        api_key: &str,
    ) -> Result<LlmResponse, LlmError> {
        let url = self.build_url(model_id, api_key);
        let request_body = build_gemini_request(messages);

        let http_response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|error| LlmError::NetworkError(error.to_string()))?;

        let status = http_response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
            || status == reqwest::StatusCode::BAD_REQUEST
        {
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

        let gemini_response: GeminiResponse = http_response
            .json()
            .await
            .map_err(|error| LlmError::ProviderError(error.to_string()))?;

        let content = gemini_response
            .candidates
            .into_iter()
            .next()
            .and_then(|candidate| candidate.content.parts.into_iter().next())
            .map(|part| part.text)
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            input_tokens: gemini_response.usage_metadata.prompt_token_count,
            output_tokens: gemini_response.usage_metadata.candidates_token_count,
        })
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool, LlmError> {
        let url = self.build_url("gemini-2.0-flash", key);
        let request_body = build_gemini_request(vec![ChatMessage {
            role: "user".to_string(),
            content: "Hi".to_string(),
        }]);

        let http_response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|error| LlmError::NetworkError(error.to_string()))?;

        let status = http_response.status();

        if status == reqwest::StatusCode::BAD_REQUEST
            || status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Ok(false);
        }

        Ok(status.is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn successful_gemini_response() -> serde_json::Value {
        serde_json::json!({
            "candidates": [
                {
                    "content": {
                        "role": "model",
                        "parts": [
                            {
                                "text": "Hello, adventurer!"
                            }
                        ]
                    },
                    "finishReason": "STOP"
                }
            ],
            "usageMetadata": {
                "promptTokenCount": 10,
                "candidatesTokenCount": 5,
                "totalTokenCount": 15
            }
        })
    }

    #[tokio::test]
    async fn send_message_parses_successful_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path_regex(r".*generateContent.*"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(successful_gemini_response()),
            )
            .mount(&mock_server)
            .await;

        let provider = GoogleGeminiProvider::with_base_url(format!(
            "{}/v1beta/models/{{model}}:generateContent",
            mock_server.uri()
        ));

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let response = provider
            .send_message(messages, "gemini-2.0-flash", "test-key")
            .await
            .expect("send_message should succeed");

        assert_eq!(response.content, "Hello, adventurer!");
        assert_eq!(response.input_tokens, 10);
        assert_eq!(response.output_tokens, 5);
    }

    #[test]
    fn system_message_is_converted_to_system_instruction() {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an oracle.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Speak!".to_string(),
            },
        ];

        let request = build_gemini_request(messages);

        assert!(request.system_instruction.is_some());
        assert_eq!(request.contents.len(), 1);
        assert_eq!(request.contents[0].role, "user");
    }

    #[test]
    fn assistant_role_is_mapped_to_model_role() {
        assert_eq!(map_role_to_gemini("assistant"), "model");
        assert_eq!(map_role_to_gemini("user"), "user");
    }

    #[test]
    fn available_models_returns_three_gemini_models() {
        let provider = GoogleGeminiProvider::new();
        let models = provider.available_models();
        assert_eq!(models.len(), 3);
        assert!(models.iter().any(|m| m.id == "gemini-2.5-pro"));
        assert!(models.iter().any(|m| m.id == "gemini-2.0-flash"));
        assert!(models.iter().any(|m| m.id == "gemini-2.5-flash"));
    }
}
