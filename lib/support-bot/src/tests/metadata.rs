use super::*;
use crate::llm::ChatMessage;
use serde_json::json;

#[test]
fn store_thread_state_preserves_unrelated_metadata() {
    let state = SupportThreadState::default();
    let stored = store_thread_state(&json!({ "other": true }), &state).unwrap();

    assert_eq!(stored["other"], true);
    assert_eq!(stored[SUPPORT_METADATA_KEY]["version"], 1);
}

#[test]
fn message_trace_round_trips_under_support_bot_key() {
    let trace = vec![ChatMessage::assistant("hello")];
    let stored = store_message_trace(&json!({}), &trace).unwrap();
    let loaded = load_message_trace(&stored).unwrap();

    assert_eq!(loaded, trace);
    assert!(stored[SUPPORT_METADATA_KEY]["llm_trace"].is_array());
}

#[test]
fn post_metadata_serializes_kind_as_snake_case() {
    let value = metadata_value(&SupportMetadata::tool_action(
        "call-1",
        "send_user_message",
    ))
    .unwrap();

    assert_eq!(value[SUPPORT_METADATA_KEY]["kind"], "tool_action");
    assert_eq!(value[SUPPORT_METADATA_KEY]["tool_call_id"], "call-1");
    assert_eq!(value[SUPPORT_METADATA_KEY]["action"], "send_user_message");
}
