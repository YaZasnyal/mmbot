use super::*;

#[test]
fn strips_think_blocks_from_user_visible_message() {
    assert_eq!(
        sanitize_user_visible_message(
            "<think>private chain</think>\nPlease restart the service.".to_string()
        )
        .as_deref(),
        Some("Please restart the service.")
    );
}

#[test]
fn strips_unclosed_think_block() {
    assert_eq!(
        sanitize_user_visible_message("Useful answer\n<think>private".to_string()).as_deref(),
        Some("Useful answer")
    );
}

#[test]
fn returns_none_when_message_is_only_hidden_reasoning() {
    assert_eq!(
        sanitize_user_visible_message("<thinking>private</thinking>".to_string()),
        None
    );
}
