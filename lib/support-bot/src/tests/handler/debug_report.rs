use super::*;

#[tokio::test]
async fn engineer_debug_command_is_handled_before_llm() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    )
    .with_debug_handler(Arc::new(StaticDebug));

    let effects = handle_thread(&handler, thread("engineers", "!support state"))
        .await
        .unwrap();

    assert!(matches!(
        &effects[0],
        ThreadEffect::Reply { message, .. } if message == "debug: state"
    ));
}
