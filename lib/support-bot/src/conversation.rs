use crate::llm::{ChatMessage, ChatRole};
use crate::state::SupportThreadState;
use crate::tools::ToolResult;
use serde_json::json;
use thread_bot::{Thread, ThreadBotError, ThreadMessage};

pub(crate) const STATE_KEY: &str = "support_bot";
pub(crate) const TRACE_KEY: &str = "llm_trace";

pub(crate) fn build_llm_messages(
    system_prompt: &str,
    thread: &Thread,
    state: &SupportThreadState,
    bot_user_id: Option<&str>,
) -> Result<Vec<ChatMessage>, ThreadBotError> {
    let mut messages = vec![
        ChatMessage::system(system_prompt.to_string()),
        ChatMessage::system(format!(
            "Current support state JSON: {}",
            serde_json::to_string(state).unwrap_or_else(|_| "{}".to_string())
        )),
    ];

    let mut thread_messages = thread
        .messages
        .iter()
        .filter(|message| !message.message.trim().is_empty())
        .collect::<Vec<_>>();
    thread_messages.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.post_id.cmp(&right.post_id))
    });

    for message in thread_messages {
        messages.push(thread_message_to_chat_message(message, bot_user_id));
        messages.extend(load_trace(&message.metadata)?);
    }

    Ok(messages)
}

fn thread_message_to_chat_message(
    message: &ThreadMessage,
    bot_user_id: Option<&str>,
) -> ChatMessage {
    let role = if bot_user_id == Some(message.user_id.as_str()) {
        ChatRole::Assistant
    } else {
        ChatRole::User
    };

    ChatMessage {
        role,
        content: Some(format!(
            "[post_id={}, user_id={}] {}",
            message.post_id, message.user_id, message.message
        )),
        name: None,
        tool_call_id: None,
        tool_calls: Vec::new(),
    }
}

pub(crate) fn result_to_message(result: ToolResult, max_bytes: usize) -> ChatMessage {
    let content = if result.is_error {
        json!({
            "is_error": true,
            "content": result.content
        })
    } else {
        result.content
    };

    ChatMessage::tool(
        result.call_id,
        truncate_utf8(content.to_string(), max_bytes),
    )
}

pub(crate) fn load_trace(metadata: &serde_json::Value) -> Result<Vec<ChatMessage>, ThreadBotError> {
    match metadata
        .get(STATE_KEY)
        .and_then(|value| value.get(TRACE_KEY))
    {
        Some(value) => serde_json::from_value(value.clone()).map_err(ThreadBotError::from),
        None => Ok(Vec::new()),
    }
}

pub(crate) fn with_trace_metadata(
    metadata: &serde_json::Value,
    trace: &[ChatMessage],
) -> Result<serde_json::Value, ThreadBotError> {
    if trace.is_empty() {
        return Ok(metadata.clone());
    }

    let mut metadata = metadata.clone();
    if !metadata.is_object() {
        metadata = json!({});
    }
    metadata[STATE_KEY][TRACE_KEY] = serde_json::to_value(trace).map_err(ThreadBotError::from)?;
    Ok(metadata)
}

pub(crate) fn load_state(
    metadata: &serde_json::Value,
) -> Result<SupportThreadState, ThreadBotError> {
    match metadata.get(STATE_KEY) {
        Some(value) => serde_json::from_value(value.clone()).map_err(ThreadBotError::from),
        None => Ok(SupportThreadState::default()),
    }
}

pub(crate) fn store_state(
    metadata: &serde_json::Value,
    state: &SupportThreadState,
) -> Result<serde_json::Value, ThreadBotError> {
    let mut metadata = metadata.clone();
    if !metadata.is_object() {
        metadata = json!({});
    }

    metadata[STATE_KEY] = serde_json::to_value(state).map_err(ThreadBotError::from)?;
    Ok(metadata)
}

pub(crate) fn truncate_utf8(value: String, max_bytes: usize) -> String {
    if max_bytes == 0 || value.len() <= max_bytes {
        return value;
    }

    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...[truncated]", &value[..end])
}
