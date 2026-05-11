use super::*;
use chrono::Utc;
use serde_json::json;
use thread_bot::{Thread, ThreadInfo, ThreadMessage};

fn thread(message: &str) -> Thread {
    Thread {
        info: ThreadInfo {
            thread_id: "thread-1".to_string(),
            root_post_id: "post-1".to_string(),
            channel_id: "users".to_string(),
            creator_user_id: "user-1".to_string(),
            thread_kind: None,
            metadata: json!({}),
            last_seen_post_id: None,
            last_seen_post_at: None,
            last_processed_post_id: None,
            last_processed_post_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        messages: vec![ThreadMessage {
            post_id: "post-1".to_string(),
            thread_id: "thread-1".to_string(),
            user_id: "user-1".to_string(),
            message: message.to_string(),
            root_id: None,
            parent_post_id: None,
            props: json!({}),
            metadata: json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_new: true,
        }],
    }
}

#[tokio::test]
async fn first_message_text_hook_accepts_empty_required_list() {
    let hook = FirstMessageTextAdmissionHook::new(Vec::<String>::new());

    let decision = hook.evaluate(&thread("anything")).await.unwrap();

    assert_eq!(decision, SupportThreadAdmissionDecision::Accept);
}

#[tokio::test]
async fn first_message_text_hook_accepts_required_text() {
    let hook = FirstMessageTextAdmissionHook::new(["@xxxduty"]);

    let decision = hook
        .evaluate(&thread("please help @xxxduty"))
        .await
        .unwrap();

    assert_eq!(decision, SupportThreadAdmissionDecision::Accept);
}

#[tokio::test]
async fn first_message_text_hook_ignores_missing_required_text() {
    let hook = FirstMessageTextAdmissionHook::new(["@xxxduty"]);

    let decision = hook.evaluate(&thread("please help")).await.unwrap();

    assert_eq!(
        decision,
        SupportThreadAdmissionDecision::Ignore {
            reason: Some("missing required text: @xxxduty".to_string())
        }
    );
}
