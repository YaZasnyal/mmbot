use super::*;
use serde_json::json;

#[test]
fn thread_state_round_trips_without_overwriting_unrelated_metadata() {
    let state = SupportThreadState::default();
    let stored = store_thread_state(&json!({ "other": true }), &state).unwrap();

    assert_eq!(stored["other"], true);
    assert_eq!(load_thread_state(&stored).unwrap(), state);
}

#[test]
fn post_metadata_serializes_kind_as_snake_case() {
    let value = metadata_value(&SupportMetadata::new(SupportMetadataKind::AcpReport)).unwrap();
    assert_eq!(value[SUPPORT_METADATA_KEY]["kind"], "acp_report");
}
