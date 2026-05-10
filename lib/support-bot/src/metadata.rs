use crate::llm::ChatMessage;
use crate::state::SupportThreadState;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thread_bot::{Thread, ThreadBotError};

pub(crate) const SUPPORT_METADATA_KEY: &str = "support_bot";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SupportPostKind {
    AssistantResponse,
    BotMessage,
    DebugResponse,
    EngineerNotification,
    EngineerThread,
    EngineerThreadRoot,
    StatusUpdate,
    ThreadHtmlReport,
    ToolAction,
    ToolLoopLimit,
    UserMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SupportPostMetadata {
    pub kind: SupportPostKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_root_post_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_channel_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl SupportPostMetadata {
    pub(crate) fn new(kind: SupportPostKind) -> Self {
        Self {
            kind,
            source_thread_id: None,
            source_root_post_id: None,
            source_channel_id: None,
            tool_call_id: None,
            action: None,
            reason: None,
        }
    }

    pub(crate) fn for_source_thread(kind: SupportPostKind, thread: &Thread) -> Self {
        Self {
            source_thread_id: Some(thread.info.thread_id.clone()),
            source_root_post_id: Some(thread.info.root_post_id.clone()),
            source_channel_id: Some(thread.info.channel_id.clone()),
            ..Self::new(kind)
        }
    }

    pub(crate) fn tool_action(call_id: &str, action: impl Into<String>) -> Self {
        Self {
            tool_call_id: Some(call_id.to_string()),
            action: Some(action.into()),
            ..Self::new(SupportPostKind::ToolAction)
        }
    }

    pub(crate) fn thread_html_report(source_thread_id: impl Into<String>) -> Self {
        Self {
            source_thread_id: Some(source_thread_id.into()),
            reason: Some("engineer_request".to_string()),
            ..Self::new(SupportPostKind::ThreadHtmlReport)
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct SupportMessageMetadata {
    #[serde(default, rename = "llm_trace", skip_serializing_if = "Vec::is_empty")]
    pub llm_trace: Vec<ChatMessage>,
}

pub(crate) fn metadata_value<T>(payload: &T) -> Result<Value, ThreadBotError>
where
    T: Serialize,
{
    let mut value = Map::new();
    value.insert(
        SUPPORT_METADATA_KEY.to_string(),
        serde_json::to_value(payload).map_err(ThreadBotError::from)?,
    );
    Ok(Value::Object(value))
}

pub(crate) fn load_thread_state(metadata: &Value) -> Result<SupportThreadState, ThreadBotError> {
    load_payload(metadata).map(|payload| payload.unwrap_or_default())
}

pub(crate) fn store_thread_state(
    metadata: &Value,
    state: &SupportThreadState,
) -> Result<Value, ThreadBotError> {
    store_payload(metadata, state)
}

pub(crate) fn load_message_trace(metadata: &Value) -> Result<Vec<ChatMessage>, ThreadBotError> {
    load_payload::<SupportMessageMetadata>(metadata)
        .map(|payload| payload.unwrap_or_default().llm_trace)
}

pub(crate) fn store_message_trace(
    metadata: &Value,
    trace: &[ChatMessage],
) -> Result<Value, ThreadBotError> {
    if trace.is_empty() {
        return Ok(metadata.clone());
    }

    let mut payload = load_payload::<SupportMessageMetadata>(metadata)?.unwrap_or_default();
    payload.llm_trace = trace.to_vec();
    store_payload(metadata, &payload)
}

fn load_payload<T>(metadata: &Value) -> Result<Option<T>, ThreadBotError>
where
    T: for<'de> Deserialize<'de>,
{
    let Some(payload) = metadata.get(SUPPORT_METADATA_KEY) else {
        return Ok(None);
    };
    serde_json::from_value(payload.clone())
        .map(Some)
        .map_err(ThreadBotError::from)
}

fn store_payload<T>(metadata: &Value, payload: &T) -> Result<Value, ThreadBotError>
where
    T: Serialize,
{
    let mut envelope = metadata.as_object().cloned().unwrap_or_default();
    envelope.insert(
        SUPPORT_METADATA_KEY.to_string(),
        serde_json::to_value(payload).map_err(ThreadBotError::from)?,
    );
    Ok(Value::Object(envelope))
}

#[cfg(test)]
#[path = "tests/metadata.rs"]
mod tests;
