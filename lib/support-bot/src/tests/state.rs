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

#[test]
fn ignored_support_thread_state_round_trips_as_json() {
    let state = SupportThreadState {
        status: SupportThreadStatus::Ignored,
        ignored_reason: Some("missing required text: @xxxduty".to_string()),
        ..SupportThreadState::default()
    };

    let value = serde_json::to_value(&state).unwrap();
    let decoded: SupportThreadState = serde_json::from_value(value).unwrap();

    assert_eq!(decoded, state);
}

#[test]
fn acp_session_state_round_trips_without_transcript() {
    let state = SupportThreadState {
        runtime: Some(SupportRuntimeState {
            version: 1,
            acp_session_id: "session-1".to_string(),
            last_inbound_post_id: "post-1".to_string(),
            last_outbound_post_id: None,
            status: SupportRuntimeStatus::Active,
        }),
        ..SupportThreadState::default()
    };

    let value = serde_json::to_value(&state).unwrap();
    assert_eq!(value["support_runtime"]["acp_session_id"], "session-1");
    assert!(value.get("transcript").is_none());
    assert_eq!(
        serde_json::from_value::<SupportThreadState>(value).unwrap(),
        state
    );
}
