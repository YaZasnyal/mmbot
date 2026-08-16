use super::{
    find_session_file, format_prompt, parse_session_trace, AcpPrompt, AcpRuntime, QwenAcpConfig,
    QwenAcpRuntime,
};
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn support_prompt_contains_thread_post_and_untrusted_message() {
    let prompt = format_prompt(&AcpPrompt {
        thread_id: "thread-1".to_string(),
        post_id: "post-2".to_string(),
        message: "service is slow".to_string(),
        session_id: None,
    });

    assert_eq!(
        prompt,
        "/support\n\nMattermost thread: thread-1\nMattermost post: post-2\nUser message:\nservice is slow"
    );
}

#[test]
fn session_trace_parses_tools_and_redacts_secrets() {
    let trace = parse_session_trace(
        r#"{"timestamp":"2026-08-16T10:00:00Z","type":"assistant","message":{"parts":[{"functionCall":{"name":"curl","args":{"url":"https://example.com","authorization":"Bearer secret"}}}]}}
{"timestamp":"2026-08-16T10:00:01Z","type":"user","message":{"parts":[{"functionResponse":{"name":"curl","response":{"status":200}}}]}}
not-json"#,
    );

    assert_eq!(trace.events.len(), 2);
    assert_eq!(trace.events[0].kind, "tool_call");
    assert!(trace.events[0].body.contains("[REDACTED]"));
    assert!(!trace.events[0].body.contains("Bearer secret"));
    assert_eq!(trace.warnings.len(), 1);
}

#[test]
fn session_file_is_found_across_qwen_projects() {
    let root = std::env::temp_dir().join(format!("mmbot-qwen-session-{}", std::process::id()));
    let chats = root.join("different-project").join("chats");
    std::fs::create_dir_all(&chats).unwrap();
    let expected = chats.join("session-1.jsonl");
    std::fs::write(&expected, "{}").unwrap();

    assert_eq!(
        find_session_file(&root, "session-1").unwrap(),
        Some(expected)
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
#[ignore = "requires the configured local Qwen Code runtime"]
async fn local_qwen_continues_the_same_acp_session() {
    let config = QwenAcpConfig {
        executable: PathBuf::from("qwen"),
        cwd: std::env::current_dir().unwrap(),
        model: Some("qwen/qwen3.6-35b-a3b".to_string()),
        timeout: Duration::from_secs(120),
    };
    let runtime = QwenAcpRuntime::start(config.clone()).await.unwrap();
    let first = runtime
        .prompt(AcpPrompt {
            thread_id: "smoke-thread".to_string(),
            post_id: "smoke-1".to_string(),
            message: "Reply with exactly: ACP_FIRST_OK".to_string(),
            session_id: None,
        })
        .await
        .unwrap();
    assert!(first.response.contains("ACP_FIRST_OK"));

    let second = runtime
        .prompt(AcpPrompt {
            thread_id: "smoke-thread".to_string(),
            post_id: "smoke-2".to_string(),
            message: "Reply with exactly: ACP_SECOND_OK".to_string(),
            session_id: Some(first.session_id.clone()),
        })
        .await
        .unwrap();
    assert_eq!(second.session_id, first.session_id);
    assert!(second.response.contains("ACP_SECOND_OK"));

    drop(runtime);
    let restarted = QwenAcpRuntime::start(config).await.unwrap();
    let third = restarted
        .prompt(AcpPrompt {
            thread_id: "smoke-thread".to_string(),
            post_id: "smoke-3".to_string(),
            message: "Reply with exactly: ACP_RESTART_OK".to_string(),
            session_id: Some(first.session_id.clone()),
        })
        .await
        .unwrap();
    assert_eq!(third.session_id, first.session_id);
    assert!(!third.session_recovered);
    assert!(third.response.contains("ACP_RESTART_OK"));
}
