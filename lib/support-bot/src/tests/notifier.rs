use super::*;
use chrono::Utc;
use thread_bot::{ThreadInfo, ThreadMessage};

fn thread() -> Thread {
    Thread {
        info: ThreadInfo {
            thread_id: "thread-1".to_string(),
            root_post_id: "root-1".to_string(),
            channel_id: "users".to_string(),
            creator_user_id: "user-1".to_string(),
            thread_kind: Some("support_user".to_string()),
            metadata: serde_json::json!({}),
            last_seen_post_id: None,
            last_seen_post_at: None,
            last_processed_post_id: None,
            last_processed_post_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        messages: vec![ThreadMessage {
            post_id: "root-1".to_string(),
            thread_id: "thread-1".to_string(),
            user_id: "user-1".to_string(),
            message: "service is down".to_string(),
            root_id: None,
            parent_post_id: None,
            props: serde_json::json!({}),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_new: true,
        }],
    }
}

#[test]
fn engineer_root_message_links_and_quotes_source_post() {
    let message = engineer_thread_root_message(
        &thread(),
        "http://localhost:8065/_redirect/pl/root-1".to_string(),
    );

    assert!(message.contains("Source: http://localhost:8065/_redirect/pl/root-1"));
    assert!(message.contains("```\nservice is down\n```"));
}

#[test]
fn source_link_uses_mattermost_redirect_url() {
    let config = Configuration {
        base_path: "http://localhost:8065/".to_string(),
        ..Configuration::default()
    };

    assert_eq!(
        source_post_link(&config, "post-1"),
        "http://localhost:8065/_redirect/pl/post-1"
    );
}

#[test]
fn mattermost_quote_uses_fence_that_survives_backticks() {
    let quoted = quote_for_mattermost("before\n```json\n{}\n```\nafter");

    assert!(quoted.starts_with("````\n"));
    assert!(quoted.ends_with("\n````"));
    assert!(quoted.contains("```json"));
}

#[test]
fn mattermost_quote_truncates_large_messages() {
    let quoted = quote_for_mattermost(&"x".repeat(MIRRORED_MESSAGE_MAX_CHARS + 1));

    assert!(quoted.contains("[message truncated]"));
    assert!(quoted.len() < MIRRORED_MESSAGE_MAX_CHARS + 100);
}

#[test]
fn html_report_includes_runtime_summary() {
    let summary = SupportReportSummary {
        support_status: "finished".to_string(),
        state_json: "{\"status\":\"finished\"}".to_string(),
    };
    let html = render_thread_html_report(
        "thread-1",
        "users",
        "root-1",
        &summary,
        &[SupportReportPost {
            post_id: "post-1".to_string(),
            user_id: "user-1".to_string(),
            message: "help".to_string(),
            created_at: "now".to_string(),
        }],
    );

    assert!(html.contains("support_status"));
    assert!(html.contains("<code>finished</code>"));
    assert!(html.contains("Runtime state"));
}

#[test]
fn html_report_orders_messages_by_created_at() {
    let summary = SupportReportSummary {
        support_status: "active".to_string(),
        state_json: "{}".to_string(),
    };
    let html = render_thread_html_report(
        "thread-1",
        "users",
        "root-1",
        &summary,
        &[
            SupportReportPost {
                post_id: "post-2".to_string(),
                user_id: "user-1".to_string(),
                message: "second".to_string(),
                created_at: "2026-05-04T20:00:02+00:00".to_string(),
            },
            SupportReportPost {
                post_id: "post-1".to_string(),
                user_id: "user-1".to_string(),
                message: "first".to_string(),
                created_at: "2026-05-04T20:00:01+00:00".to_string(),
            },
        ],
    );

    assert!(html.find("first").unwrap() < html.find("second").unwrap());
}
