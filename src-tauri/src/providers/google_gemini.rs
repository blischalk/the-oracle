use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::llm_provider::{ChatMessage, LlmError, LlmProvider, LlmResponse, ModelDescriptor};

const GEMINI_API_BASE: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent";

const GEMINI_MODELS_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models";

pub struct GoogleGeminiProvider {
    client: Client,
    base_url: String,
    models_url: String,
}

impl GoogleGeminiProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: GEMINI_API_BASE.to_string(),
            models_url: GEMINI_MODELS_URL.to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(base_url: String) -> Self {
        // base_url pattern: "http://server/v1beta/models/{model}:generateContent"
        // models_url strips the per-model suffix to get the list endpoint.
        let models_url = base_url.replace("/{model}:generateContent", "");
        Self {
            client: Client::new(),
            models_url,
            base_url,
        }
    }

    fn build_generate_url(&self, model_id: &str, api_key: &str) -> String {
        let url = self.base_url.replace("{model}", model_id);
        format!("{url}?key={api_key}")
    }

    fn build_models_url(&self, api_key: &str) -> String {
        format!("{}?key={api_key}", self.models_url)
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

/// Envelope returned by the Gemini API on error.
#[derive(Debug, Deserialize)]
struct GeminiErrorEnvelope {
    error: GeminiErrorBody,
}

#[derive(Debug, Deserialize)]
struct GeminiErrorBody {
    message: String,
}

/// Returns true when a Gemini error body is specifically about the API key
/// being invalid, as opposed to other 400-class errors (wrong model ID,
/// malformed request, quota exhausted on a per-project basis, etc.).
fn is_invalid_key_error(body: &str) -> bool {
    let lower = body.to_lowercase();
    lower.contains("api key not valid")
        || lower.contains("api_key_invalid")
        || lower.contains("invalid api key")
}

fn extract_error_message(body: &str) -> String {
    serde_json::from_str::<GeminiErrorEnvelope>(body)
        .map(|e| e.error.message)
        .unwrap_or_else(|_| body.to_string())
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
                id: "gemini-2.5-flash".to_string(),
                display_name: "Gemini 2.5 Flash".to_string(),
                context_window: 1_000_000,
            },
            ModelDescriptor {
                id: "gemini-2.0-flash".to_string(),
                display_name: "Gemini 2.0 Flash".to_string(),
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
        let url = self.build_generate_url(model_id, api_key);
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

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(LlmError::RateLimited);
        }

        if !status.is_success() {
            let body = http_response.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::UNAUTHORIZED || is_invalid_key_error(&body) {
                return Err(LlmError::InvalidApiKey);
            }
            return Err(LlmError::ProviderError(format!(
                "HTTP {status}: {}",
                extract_error_message(&body)
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

    /// Validates the key by calling the models list endpoint — a lightweight
    /// GET that returns 200 for any valid key and a clear error for invalid ones,
    /// avoiding the ambiguous 400 responses that chat completions can return.
    async fn validate_api_key(&self, key: &str) -> Result<bool, LlmError> {
        let url = self.build_models_url(key);

        let http_response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|error| LlmError::NetworkError(error.to_string()))?;

        let status = http_response.status();

        if status.is_success() {
            return Ok(true);
        }

        let body = http_response.text().await.unwrap_or_default();

        // A valid key that hits a permission boundary (e.g. project billing not
        // enabled) should not be reported as invalid — surface the real reason.
        if status == reqwest::StatusCode::UNAUTHORIZED || is_invalid_key_error(&body) {
            return Ok(false);
        }

        // Any other non-success (403 quota, 429, 5xx) means the key itself is
        // probably fine but something else is wrong.
        Err(LlmError::ProviderError(format!(
            "HTTP {status}: {}",
            extract_error_message(&body)
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, path_regex, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn successful_gemini_response() -> serde_json::Value {
        serde_json::json!({
            "candidates": [
                {
                    "content": {
                        "role": "model",
                        "parts": [{ "text": "Hello, adventurer!" }]
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

    fn invalid_key_error_body() -> serde_json::Value {
        serde_json::json!({
            "error": {
                "code": 400,
                "message": "API key not valid. Please pass a valid API key.",
                "status": "INVALID_ARGUMENT"
            }
        })
    }

    fn model_not_found_error_body() -> serde_json::Value {
        serde_json::json!({
            "error": {
                "code": 404,
                "message": "models/gemini-bad-model is not found for API version v1beta.",
                "status": "NOT_FOUND"
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

        let response = provider
            .send_message(vec![ChatMessage::user("Hello")], "gemini-2.0-flash", "test-key")
            .await
            .expect("send_message should succeed");

        assert_eq!(response.content, "Hello, adventurer!");
        assert_eq!(response.input_tokens, 10);
        assert_eq!(response.output_tokens, 5);
    }

    #[tokio::test]
    async fn send_message_returns_invalid_key_error_when_body_says_so() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path_regex(r".*generateContent.*"))
            .respond_with(
                ResponseTemplate::new(400).set_body_json(invalid_key_error_body()),
            )
            .mount(&mock_server)
            .await;

        let provider = GoogleGeminiProvider::with_base_url(format!(
            "{}/v1beta/models/{{model}}:generateContent",
            mock_server.uri()
        ));

        let err = provider
            .send_message(vec![ChatMessage::user("Hello")], "gemini-2.0-flash", "bad-key")
            .await
            .expect_err("should fail with invalid key");

        assert!(matches!(err, LlmError::InvalidApiKey));
    }

    #[tokio::test]
    async fn send_message_returns_provider_error_for_non_key_400() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path_regex(r".*generateContent.*"))
            .respond_with(
                ResponseTemplate::new(404).set_body_json(model_not_found_error_body()),
            )
            .mount(&mock_server)
            .await;

        let provider = GoogleGeminiProvider::with_base_url(format!(
            "{}/v1beta/models/{{model}}:generateContent",
            mock_server.uri()
        ));

        let err = provider
            .send_message(vec![ChatMessage::user("Hello")], "gemini-bad-model", "valid-key")
            .await
            .expect_err("should fail with provider error");

        assert!(matches!(err, LlmError::ProviderError(_)));
    }

    #[tokio::test]
    async fn validate_api_key_returns_true_when_models_endpoint_succeeds() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1beta/models"))
            .and(query_param("key", "valid-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "models": []
            })))
            .mount(&mock_server)
            .await;

        let provider = GoogleGeminiProvider::with_base_url(format!(
            "{}/v1beta/models/{{model}}:generateContent",
            mock_server.uri()
        ));

        assert!(provider.validate_api_key("valid-key").await.unwrap());
    }

    #[tokio::test]
    async fn validate_api_key_returns_false_for_invalid_key_body() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1beta/models"))
            .respond_with(
                ResponseTemplate::new(400).set_body_json(invalid_key_error_body()),
            )
            .mount(&mock_server)
            .await;

        let provider = GoogleGeminiProvider::with_base_url(format!(
            "{}/v1beta/models/{{model}}:generateContent",
            mock_server.uri()
        ));

        assert!(!provider.validate_api_key("bad-key").await.unwrap());
    }

    #[test]
    fn system_message_is_converted_to_system_instruction() {
        let messages = vec![
            ChatMessage::system("You are an oracle."),
            ChatMessage::user("Speak!"),
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
        assert!(models.iter().any(|m| m.display_name == "Gemini 2.5 Pro"));
        assert!(models.iter().any(|m| m.id == "gemini-2.0-flash"));
        assert!(models.iter().any(|m| m.display_name == "Gemini 2.5 Flash"));
    }

    #[test]
    fn is_invalid_key_error_matches_known_google_messages() {
        assert!(is_invalid_key_error("API key not valid. Please pass a valid API key."));
        assert!(is_invalid_key_error(r#"{"error":{"message":"API key not valid"}}"#));
        assert!(!is_invalid_key_error("models/gemini-bad is not found"));
        assert!(!is_invalid_key_error("Request contains an invalid argument."));
    }
}
