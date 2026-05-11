use super::*;

#[tokio::test]
async fn engineer_thread_only_handles_last_external_message() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    )
    .with_debug_handler(Arc::new(StaticDebug));
    let mut thread = thread("engineers", "!support state");
    thread.messages.push(ThreadMessage {
        post_id: "post-2".to_string(),
        thread_id: "thread-1".to_string(),
        user_id: "user-1".to_string(),
        message: "ordinary engineer discussion".to_string(),
        root_id: Some("post-1".to_string()),
        parent_post_id: Some("post-1".to_string()),
        props: json!({}),
        metadata: json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_new: true,
    });

    let effects = handle_thread(&handler, thread).await.unwrap();

    assert!(matches!(effects.as_slice(), [ThreadEffect::Noop]));
}

#[tokio::test]
async fn engineer_thread_ignores_new_bot_messages_after_command() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    )
    .with_debug_handler(Arc::new(StaticDebug));
    let mut thread = thread("engineers", "!support state");
    thread.messages.push(ThreadMessage {
        post_id: "post-2".to_string(),
        thread_id: "thread-1".to_string(),
        user_id: "bot".to_string(),
        message: "bot echo".to_string(),
        root_id: Some("post-1".to_string()),
        parent_post_id: Some("post-1".to_string()),
        props: json!({}),
        metadata: json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_new: true,
    });

    let effects = handle_thread(&handler, thread).await.unwrap();

    assert!(matches!(
        &effects[0],
        ThreadEffect::Reply { message, .. } if message == "debug: state"
    ));
}
