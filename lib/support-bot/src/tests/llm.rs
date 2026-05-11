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
            arguments: serde_json::json!({ "id": null }),
        }],
    };

    let wire = OpenAiMessage::from(message);
    let value = serde_json::to_value(wire).unwrap();

    assert_eq!(value["tool_calls"][0]["type"], "function");
    assert_eq!(value["tool_calls"][0]["function"]["name"], "instructions");
    assert_eq!(
        value["tool_calls"][0]["function"]["arguments"],
        "{\"id\":null}"
    );
}
