use super::*;

#[test]
fn control_reactions_update_support_status_metadata() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    );
    let record = thread_record("users", json!({}));

    let resolved = handler
        .on_control_reaction(
            &record,
            &ReactionChange {
                post_id: "post-1".to_string(),
                user_id: "user-2".to_string(),
                emoji_name: "white_check_mark".to_string(),
                action: ReactionAction::Added,
                created_at: Utc::now(),
            },
        )
        .expect("white_check_mark should update support status");
    let ThreadEffect::SetThreadMetadata { metadata, .. } = resolved else {
        panic!("expected SetThreadMetadata");
    };
    assert_eq!(metadata[STATE_KEY]["status"], "finished");
    assert_eq!(
        metadata[STATE_KEY]["finished_summary"],
        "resolved by control reaction"
    );

    let stopped = handler
        .on_control_reaction(
            &record,
            &ReactionChange {
                post_id: "post-1".to_string(),
                user_id: "user-2".to_string(),
                emoji_name: "stop_sign".to_string(),
                action: ReactionAction::Added,
                created_at: Utc::now(),
            },
        )
        .expect("stop_sign should update support status");
    let ThreadEffect::SetThreadMetadata { metadata, .. } = stopped else {
        panic!("expected SetThreadMetadata");
    };
    assert_eq!(metadata[STATE_KEY]["status"], "stopped");
}

#[test]
fn control_reactions_route_channel_backed_user_threads_without_thread_kind() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    );
    let mut record = thread_record("users", json!({}));
    record.thread_kind = None;

    let resolved = handler
        .on_control_reaction(
            &record,
            &ReactionChange {
                post_id: "post-1".to_string(),
                user_id: "user-2".to_string(),
                emoji_name: "white_check_mark".to_string(),
                action: ReactionAction::Added,
                created_at: Utc::now(),
            },
        )
        .expect("channel-routed user thread should accept control reaction");
    let ThreadEffect::SetThreadMetadata { metadata, .. } = resolved else {
        panic!("expected SetThreadMetadata");
    };

    assert_eq!(metadata[STATE_KEY]["status"], "finished");
}
