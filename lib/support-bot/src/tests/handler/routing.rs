use super::*;

#[tokio::test]
async fn ignored_route_returns_noop() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    );

    let ignored = thread("other", "hello");
    let effects = handler
        .handle(
            &ThreadInvocation {
                thread: record_from_thread(&ignored),
                trigger: ThreadTrigger::NewMessage {
                    post_id: "root".to_string(),
                },
            },
            &context(),
        )
        .await
        .unwrap();

    assert!(matches!(effects.as_slice(), [ThreadEffect::Noop]));
}
