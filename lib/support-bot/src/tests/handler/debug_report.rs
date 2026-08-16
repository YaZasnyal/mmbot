use super::*;

#[tokio::test]
async fn engineer_debug_command_is_handled_without_invoking_acp() {
    let handler = SupportBotHandler::new("support", test_config(), Arc::new(StaticAcp))
        .with_debug_handler(Arc::new(StaticDebug));

    let effects = handle_thread(&handler, thread("engineers", "!support state"))
        .await
        .unwrap();

    assert!(matches!(
        &effects[0],
        ThreadEffect::Reply { message, .. } if message == "debug: state"
    ));
}
