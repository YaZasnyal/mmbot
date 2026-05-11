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

#[test]
fn strips_leading_thread_context_marker() {
    assert_eq!(
        sanitize_user_visible_message(
            "[post_id=post-1, user_id=user-1]\nPlease share the request id.".to_string()
        )
        .as_deref(),
        Some("Please share the request id.")
    );
}

#[test]
fn strips_multiple_leading_thread_context_markers() {
    assert_eq!(
        sanitize_user_visible_message(
            "[post_id=post-1, user_id=bot] [post_id=post-2, user_id=user-1]\nDone."
                .to_string()
        )
        .as_deref(),
        Some("Done.")
    );
}

#[test]
fn returns_none_when_message_is_only_thread_context_marker() {
    assert_eq!(
        sanitize_user_visible_message("[post_id=post-1, user_id=user-1]".to_string()),
        None
    );
}
