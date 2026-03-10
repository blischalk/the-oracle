use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::llm_provider::{
    ChatMessage, LlmError, LlmProvider, LlmResponse, ModelDescriptor, ProviderTurn, ToolCall,
    ToolDefinition,
};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const VALIDATION_MAX_TOKENS: u32 = 1;
const CONTEXT_MAX_TOKENS: u32 = 4096;

pub struct AnthropicProvider {
    client: Client,
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ── Simple request/response (no tools) ──────────────────────────────────────

#[derive(Debug, Serialize)]
struct SimpleRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<SimpleMessage>,
}

#[derive(Debug, Serialize)]
struct SimpleMessage {
    role: String,
    content: String,
}

// ── Tool-aware request/response ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ToolRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<AnthropicTool>,
}

/// A message sent to Anthropic whose content may be a plain string or a list
/// of typed content blocks (text, tool_use, tool_result).
#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: AnthropicContent,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum AnthropicContent {
    Text(String),
    Blocks(Vec<AnthropicBlock>),
}

/// A single content block inside a message.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        is_error: bool,
    },
}

#[derive(Debug, Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

// ── Response types (shared) ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ResponseBlock>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ResponseBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

// ── Conversion helpers ────────────────────────────────────────────────────────

fn extract_system_prompt(messages: &[ChatMessage]) -> Option<String> {
    messages
        .iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.clone())
}

fn to_simple_messages(messages: Vec<ChatMessage>) -> Vec<SimpleMessage> {
    messages
        .into_iter()
        .filter(|m| m.role != "system")
        .map(|m| SimpleMessage {
            role: m.role,
            content: m.content,
        })
        .collect()
}

/// Convert our internal `ChatMessage` to an `AnthropicMessage` that supports
/// tool_use and tool_result content blocks.
fn to_anthropic_message(message: ChatMessage) -> Option<AnthropicMessage> {
    match message.role.as_str() {
        "system" => None, // extracted separately

        // Assistant message may carry tool-call blocks alongside text.
        "assistant" => {
            if message.tool_calls.is_empty() {
                Some(AnthropicMessage {
                    role: "assistant".to_string(),
                    content: AnthropicContent::Text(message.content),
                })
            } else {
                let mut blocks: Vec<AnthropicBlock> = Vec::new();
                if !message.content.is_empty() {
                    blocks.push(AnthropicBlock::Text {
                        text: message.content,
                    });
                }
                for call in message.tool_calls {
                    blocks.push(AnthropicBlock::ToolUse {
                        id: call.id,
                        name: call.tool_name,
                        input: call.arguments,
                    });
                }
                Some(AnthropicMessage {
                    role: "assistant".to_string(),
                    content: AnthropicContent::Blocks(blocks),
                })
            }
        }

        // Tool results arrive as role "tool"; Anthropic expects role "user"
        // with tool_result content blocks.
        "tool" => {
            let blocks: Vec<AnthropicBlock> = message
                .tool_results
                .into_iter()
                .map(|result| AnthropicBlock::ToolResult {
                    tool_use_id: result.call_id,
                    content: result.content.to_string(),
                    is_error: result.is_error,
                })
                .collect();

            if blocks.is_empty() {
                None
            } else {
                Some(AnthropicMessage {
                    role: "user".to_string(),
                    content: AnthropicContent::Blocks(blocks),
                })
            }
        }

        // Plain user messages.
        _ => Some(AnthropicMessage {
            role: message.role,
            content: AnthropicContent::Text(message.content),
        }),
    }
}

fn tool_definition_to_anthropic(tool: &ToolDefinition) -> AnthropicTool {
    AnthropicTool {
        name: tool.name.clone(),
        description: tool.description.clone(),
        input_schema: tool.parameters.clone(),
    }
}

fn extract_provider_turn(response: AnthropicResponse) -> ProviderTurn {
    let mut text_parts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();

    for block in response.content {
        match block {
            ResponseBlock::Text { text } => text_parts.push(text),
            ResponseBlock::ToolUse { id, name, input } => {
                tool_calls.push(ToolCall {
                    id,
                    tool_name: name,
                    arguments: input,
                });
            }
            ResponseBlock::Unknown => {}
        }
    }

    ProviderTurn {
        text: text_parts.join(""),
        tool_calls,
        input_tokens: response.usage.input_tokens,
        output_tokens: response.usage.output_tokens,
    }
}

fn extract_text_from_response(response: AnthropicResponse) -> String {
    response
        .content
        .into_iter()
        .filter_map(|block| {
            if let ResponseBlock::Text { text } = block {
                Some(text)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

// ── HTTP helpers ──────────────────────────────────────────────────────────────

async fn post_to_anthropic(
    client: &Client,
    api_key: &str,
    body: &impl Serialize,
) -> Result<AnthropicResponse, LlmError> {
    let http_response = client
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(body)
        .send()
        .await
        .map_err(|e| LlmError::NetworkError(e.to_string()))?;

    let status = http_response.status();

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(LlmError::InvalidApiKey);
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(LlmError::RateLimited);
    }
    if !status.is_success() {
        let body = http_response.text().await.unwrap_or_default();
        return Err(LlmError::ProviderError(format!("HTTP {status}: {body}")));
    }

    http_response
        .json::<AnthropicResponse>()
        .await
        .map_err(|e| LlmError::ProviderError(e.to_string()))
}

// ── LlmProvider implementation ────────────────────────────────────────────────

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn provider_id(&self) -> &str {
        "anthropic"
    }

    fn display_name(&self) -> &str {
        "Anthropic"
    }

    fn available_models(&self) -> Vec<ModelDescriptor> {
        vec![
            ModelDescriptor {
                id: "claude-opus-4-6".to_string(),
                display_name: "Claude Opus 4.6".to_string(),
                context_window: 200_000,
            },
            ModelDescriptor {
                id: "claude-sonnet-4-6".to_string(),
                display_name: "Claude Sonnet 4.6".to_string(),
                context_window: 200_000,
            },
            ModelDescriptor {
                id: "claude-haiku-4-5-20251001".to_string(),
                display_name: "Claude Haiku 4.5".to_string(),
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
        let system = extract_system_prompt(&messages);
        let request = SimpleRequest {
            model: model_id.to_string(),
            max_tokens: CONTEXT_MAX_TOKENS,
            system,
            messages: to_simple_messages(messages),
        };

        let response = post_to_anthropic(&self.client, api_key, &request).await?;
        let input_tokens = response.usage.input_tokens;
        let output_tokens = response.usage.output_tokens;
        let content = extract_text_from_response(response);

        Ok(LlmResponse {
            content,
            input_tokens,
            output_tokens,
        })
    }

    async fn send_message_with_tools(
        &self,
        messages: Vec<ChatMessage>,
        tools: &[ToolDefinition],
        model_id: &str,
        api_key: &str,
    ) -> Result<ProviderTurn, LlmError> {
        let system = extract_system_prompt(&messages);
        let anthropic_messages: Vec<AnthropicMessage> = messages
            .into_iter()
            .filter_map(to_anthropic_message)
            .collect();

        let request = ToolRequest {
            model: model_id.to_string(),
            max_tokens: CONTEXT_MAX_TOKENS,
            system,
            messages: anthropic_messages,
            tools: tools.iter().map(tool_definition_to_anthropic).collect(),
        };

        let response = post_to_anthropic(&self.client, api_key, &request).await?;
        Ok(extract_provider_turn(response))
    }

    async fn validate_api_key(&self, key: &str) -> Result<bool, LlmError> {
        let request = SimpleRequest {
            model: "claude-haiku-4-5-20251001".to_string(),
            max_tokens: VALIDATION_MAX_TOKENS,
            system: None,
            messages: vec![SimpleMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
        };

        let http_response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        let status = http_response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            let body = http_response.text().await.unwrap_or_default();
            let message = if body.is_empty() {
                "Invalid API key. Check the key at console.anthropic.com or try generating a new one.".to_string()
            } else {
                format!("Invalid API key: {body}")
            };
            return Err(LlmError::ProviderError(message));
        }

        if status.is_success() {
            return Ok(true);
        }

        let body = http_response.text().await.unwrap_or_default();
        Err(LlmError::ProviderError(format!("HTTP {status}: {body}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::llm_provider::ToolResult;

    fn text_response_json() -> serde_json::Value {
        serde_json::json!({
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": "Hello, adventurer!"}],
            "model": "claude-opus-4-6",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        })
    }

    fn tool_use_response_json() -> serde_json::Value {
        serde_json::json!({
            "id": "msg_456",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "text", "text": "Let me roll that for you."},
                {"type": "tool_use", "id": "toolu_01", "name": "roll_dice", "input": {"notation": "2d6"}}
            ],
            "model": "claude-sonnet-4-6",
            "stop_reason": "tool_use",
            "usage": {"input_tokens": 20, "output_tokens": 15}
        })
    }

    #[test]
    fn parses_text_only_response() {
        let raw: AnthropicResponse =
            serde_json::from_value(text_response_json()).expect("should deserialise");
        let turn = extract_provider_turn(raw);

        assert_eq!(turn.text, "Hello, adventurer!");
        assert!(turn.tool_calls.is_empty());
        assert!(turn.is_final());
    }

    #[test]
    fn parses_tool_use_response() {
        let raw: AnthropicResponse =
            serde_json::from_value(tool_use_response_json()).expect("should deserialise");
        let turn = extract_provider_turn(raw);

        assert_eq!(turn.text, "Let me roll that for you.");
        assert_eq!(turn.tool_calls.len(), 1);
        assert_eq!(turn.tool_calls[0].id, "toolu_01");
        assert_eq!(turn.tool_calls[0].tool_name, "roll_dice");
        assert!(!turn.is_final());
    }

    #[test]
    fn system_message_extracted_correctly() {
        let messages = vec![
            ChatMessage::system("You are a dungeon master."),
            ChatMessage::user("Begin."),
        ];
        let system = extract_system_prompt(&messages);
        assert_eq!(system.as_deref(), Some("You are a dungeon master."));
    }

    #[test]
    fn system_message_excluded_from_simple_messages() {
        let messages = vec![
            ChatMessage::system("You are a GM."),
            ChatMessage::user("Start."),
        ];
        let simple = to_simple_messages(messages);
        assert_eq!(simple.len(), 1);
        assert_eq!(simple[0].role, "user");
    }

    #[test]
    fn assistant_message_with_tool_calls_serialises_as_blocks() {
        let msg = ChatMessage::assistant_with_tool_calls(
            "Rolling now.".to_string(),
            vec![ToolCall {
                id: "toolu_01".to_string(),
                tool_name: "roll_dice".to_string(),
                arguments: serde_json::json!({"notation": "1d20"}),
            }],
        );

        let anthropic_msg = to_anthropic_message(msg).expect("should produce a message");
        let json = serde_json::to_value(&anthropic_msg).unwrap();

        assert_eq!(json["role"], "assistant");
        let blocks = json["content"].as_array().unwrap();
        assert_eq!(blocks[0]["type"], "text");
        assert_eq!(blocks[1]["type"], "tool_use");
        assert_eq!(blocks[1]["id"], "toolu_01");
    }

    #[test]
    fn tool_results_message_serialises_as_user_role_with_tool_result_blocks() {
        let msg = ChatMessage::tool_results_message(vec![ToolResult {
            call_id: "toolu_01".to_string(),
            tool_name: "roll_dice".to_string(),
            content: serde_json::json!({"total": 14}),
            is_error: false,
        }]);

        let anthropic_msg = to_anthropic_message(msg).expect("should produce a message");
        let json = serde_json::to_value(&anthropic_msg).unwrap();

        assert_eq!(json["role"], "user");
        let blocks = json["content"].as_array().unwrap();
        assert_eq!(blocks[0]["type"], "tool_result");
        assert_eq!(blocks[0]["tool_use_id"], "toolu_01");
    }

    #[test]
    fn tool_definition_converts_to_anthropic_shape() {
        let tool = ToolDefinition {
            name: "roll_dice".to_string(),
            description: "Roll dice.".to_string(),
            parameters: serde_json::json!({"type": "object", "properties": {}}),
        };
        let at = tool_definition_to_anthropic(&tool);
        assert_eq!(at.name, "roll_dice");
        assert_eq!(at.input_schema["type"], "object");
    }

    #[test]
    fn available_models_returns_three_claude_models() {
        let provider = AnthropicProvider::new();
        let models = provider.available_models();
        assert_eq!(models.len(), 3);
        assert!(models.iter().any(|m| m.id == "claude-opus-4-6"));
        assert!(models.iter().any(|m| m.id == "claude-sonnet-4-6"));
        assert!(models.iter().any(|m| m.id == "claude-haiku-4-5-20251001"));
    }
}
