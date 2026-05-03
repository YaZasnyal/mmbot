use crate::config::LlmConfig;
use crate::error::{Result, SupportBotError};
use crate::tools::{ToolCall, ToolSpec};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse>;
}

#[derive(Clone)]
pub struct OpenAiChatCompletionsClient {
    http: reqwest::Client,
    config: LlmConfig,
}

impl OpenAiChatCompletionsClient {
    pub fn new(config: LlmConfig) -> Result<Self> {
        let http = reqwest::Client::builder().timeout(config.timeout).build()?;
        Ok(Self { http, config })
    }

    fn completions_url(&self) -> String {
        format!(
            "{}/v1/chat/completions",
            self.config.base_url.trim_end_matches('/')
        )
    }
}

#[async_trait]
impl LlmClient for OpenAiChatCompletionsClient {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let wire_request = OpenAiChatRequest::from(request);
        let mut http_request = self.http.post(self.completions_url()).json(&wire_request);

        if let Some(api_key) = &self.config.api_key {
            http_request = http_request.bearer_auth(api_key);
        }

        let response = http_request.send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(SupportBotError::Llm(format!(
                "chat completions request failed with {status}: {body}"
            )));
        }

        let wire_response = response.json::<OpenAiChatResponse>().await?;
        wire_response.try_into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ToolSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmResponse {
    pub message: ChatMessage,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: Option<String>,
    pub name: Option<String>,
    pub tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(ChatRole::System, content)
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(ChatRole::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(ChatRole::Assistant, content)
    }

    pub fn tool(call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Tool,
            content: Some(content.into()),
            name: None,
            tool_call_id: Some(call_id.into()),
            tool_calls: Vec::new(),
        }
    }

    fn new(role: ChatRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: Some(content.into()),
            name: None,
            tool_call_id: None,
            tool_calls: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<OpenAiTool>,
}

impl From<LlmRequest> for OpenAiChatRequest {
    fn from(request: LlmRequest) -> Self {
        Self {
            model: request.model,
            messages: request.messages.into_iter().map(Into::into).collect(),
            tools: request.tools.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: ChatRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tool_calls: Vec<OpenAiToolCall>,
}

impl From<ChatMessage> for OpenAiMessage {
    fn from(message: ChatMessage) -> Self {
        Self {
            role: message.role,
            content: message.content,
            name: message.name,
            tool_call_id: message.tool_call_id,
            tool_calls: message.tool_calls.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<OpenAiMessage> for ChatMessage {
    fn from(message: OpenAiMessage) -> Self {
        Self {
            role: message.role,
            content: message.content,
            name: message.name,
            tool_call_id: message.tool_call_id,
            tool_calls: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct OpenAiTool {
    #[serde(rename = "type")]
    kind: &'static str,
    function: OpenAiFunctionSpec,
}

impl From<ToolSpec> for OpenAiTool {
    fn from(tool: ToolSpec) -> Self {
        Self {
            kind: "function",
            function: OpenAiFunctionSpec {
                name: tool.name,
                description: tool.description,
                parameters: tool.input_schema,
            },
        }
    }
}

#[derive(Debug, Serialize)]
struct OpenAiFunctionSpec {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}

impl TryFrom<OpenAiChatResponse> for LlmResponse {
    type Error = SupportBotError;

    fn try_from(response: OpenAiChatResponse) -> Result<Self> {
        let choice = response.choices.into_iter().next().ok_or_else(|| {
            SupportBotError::Llm("chat completions response had no choices".into())
        })?;

        let tool_calls = choice
            .message
            .tool_calls
            .iter()
            .map(ToolCall::try_from)
            .collect::<Result<Vec<_>>>()?;
        let mut message = ChatMessage::from(choice.message);
        message.tool_calls = tool_calls.clone();

        Ok(Self {
            message,
            tool_calls,
        })
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: OpenAiToolCallType,
    function: OpenAiToolCallFunction,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OpenAiToolCallType {
    Function,
}

impl From<ToolCall> for OpenAiToolCall {
    fn from(call: ToolCall) -> Self {
        Self {
            id: call.id,
            kind: OpenAiToolCallType::Function,
            function: OpenAiToolCallFunction {
                name: call.name,
                arguments: call.arguments.to_string(),
            },
        }
    }
}

impl TryFrom<&OpenAiToolCall> for ToolCall {
    type Error = SupportBotError;

    fn try_from(call: &OpenAiToolCall) -> Result<Self> {
        let arguments = if call.function.arguments.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&call.function.arguments)?
        };

        Ok(Self {
            id: call.id.clone(),
            name: call.function.name.clone(),
            arguments,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiToolCallFunction {
    name: String,
    arguments: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolKind;

    #[test]
    fn converts_tool_spec_to_openai_function_tool() {
        let request = LlmRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage::user("hello")],
            tools: vec![ToolSpec {
                name: "search_logs".to_string(),
                description: "Search logs".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }),
                kind: ToolKind::ReadOnly,
            }],
        };

        let wire = OpenAiChatRequest::from(request);
        let value = serde_json::to_value(wire).unwrap();

        assert_eq!(value["tools"][0]["type"], "function");
        assert_eq!(value["tools"][0]["function"]["name"], "search_logs");
        assert_eq!(
            value["tools"][0]["function"]["parameters"]["properties"]["query"]["type"],
            "string"
        );
    }

    #[test]
    fn parses_openai_tool_calls() {
        let response: OpenAiChatResponse = serde_json::from_value(serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "search_logs",
                            "arguments": "{\"query\":\"timeout\"}"
                        }
                    }]
                }
            }]
        }))
        .unwrap();

        let response = LlmResponse::try_from(response).unwrap();

        assert_eq!(response.tool_calls[0].id, "call_1");
        assert_eq!(response.tool_calls[0].name, "search_logs");
        assert_eq!(response.tool_calls[0].arguments["query"], "timeout");
        assert_eq!(response.message.tool_calls[0].id, "call_1");
    }

    #[test]
    fn serializes_assistant_tool_calls() {
        let message = ChatMessage {
            role: ChatRole::Assistant,
            content: None,
            name: None,
            tool_call_id: None,
            tool_calls: vec![ToolCall {
                id: "call_1".to_string(),
                name: "instructions".to_string(),
                arguments: serde_json::json!({ "action": "list" }),
            }],
        };

        let wire = OpenAiMessage::from(message);
        let value = serde_json::to_value(wire).unwrap();

        assert_eq!(value["tool_calls"][0]["type"], "function");
        assert_eq!(value["tool_calls"][0]["function"]["name"], "instructions");
        assert_eq!(
            value["tool_calls"][0]["function"]["arguments"],
            "{\"action\":\"list\"}"
        );
    }
}
