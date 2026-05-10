use super::*;

#[test]
fn support_thread_state_round_trips_as_json() {
    let state = SupportThreadState::default();

    let value = serde_json::to_value(&state).unwrap();
    let decoded: SupportThreadState = serde_json::from_value(value).unwrap();

    assert_eq!(decoded, state);
}

#[test]
fn support_thread_state_ignores_legacy_engineer_thread_field() {
    let decoded: SupportThreadState = serde_json::from_value(serde_json::json!({
        "version": 1,
        "engineer_thread": {
            "channel_id": "engineers",
            "root_post_id": "post-1"
        },
        "status": "active"
    }))
    .unwrap();

    assert_eq!(decoded, SupportThreadState::default());
}
