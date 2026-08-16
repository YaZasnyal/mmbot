use super::{format_prompt, AcpPrompt, AcpRuntime, QwenAcpConfig, QwenAcpRuntime};
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
