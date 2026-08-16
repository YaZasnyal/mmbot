use super::*;
use crate::state::{SupportRuntimeState, SupportRuntimeStatus};
use std::sync::Mutex;

struct RecordingAcp {
    result: Result<AcpTurn, String>,
    prompts: Mutex<Vec<AcpPrompt>>,
}

#[async_trait]
impl AcpRuntime for RecordingAcp {
    async fn prompt(&self, prompt: AcpPrompt) -> crate::Result<AcpTurn> {
        self.prompts.lock().unwrap().push(prompt);
        self.result.clone().map_err(crate::SupportBotError::Acp)
    }
}

fn runtime(result: Result<AcpTurn, String>) -> Arc<RecordingAcp> {
    Arc::new(RecordingAcp {
        result,
        prompts: Mutex::new(Vec::new()),
    })
}

fn success() -> AcpTurn {
    AcpTurn {
        session_id: "session-1".to_string(),
        response: "final answer".to_string(),
        stop_reason: "end_turn".to_string(),
        session_recovered: false,
    }
}

#[tokio::test]
async fn acp_response_is_published_and_session_is_persisted() {
    let runtime = runtime(Ok(success()));
    let handler = SupportBotHandler::new("support", test_config(), runtime.clone());
    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert_eq!(runtime.prompts.lock().unwrap().len(), 1);
    assert!(effects.iter().any(|effect| matches!(
        effect,
        ThreadEffect::Reply { target: ThreadTarget::CurrentThread, message, .. }
            if message == "final answer"
    )));
    assert!(effects.iter().any(|effect| matches!(
        effect,
        ThreadEffect::Reply { message, .. } if message.contains("Qwen ACP report")
    )));
    let metadata = effects.iter().find_map(|effect| match effect {
        ThreadEffect::SetThreadMetadata { metadata, .. } => Some(metadata),
        _ => None,
    });
    assert_eq!(
        metadata.unwrap()[STATE_KEY]["support_runtime"]["acp_session_id"],
        "session-1"
    );
}

#[tokio::test]
async fn duplicate_post_does_not_prompt_acp_again() {
    let runtime = runtime(Ok(success()));
    let handler = SupportBotHandler::new("support", test_config(), runtime.clone());
    let mut thread = thread("users", "help");
    thread.info.metadata = store_state(
        &json!({}),
        &SupportThreadState {
            runtime: Some(SupportRuntimeState {
                version: 1,
                acp_session_id: "session-1".to_string(),
                last_inbound_post_id: "post-1".to_string(),
                last_outbound_post_id: None,
                status: SupportRuntimeStatus::Active,
            }),
            ..SupportThreadState::default()
        },
    )
    .unwrap();

    let effects = handle_thread(&handler, thread).await.unwrap();
    assert!(matches!(effects.as_slice(), [ThreadEffect::Noop]));
    assert!(runtime.prompts.lock().unwrap().is_empty());
}

#[tokio::test]
async fn acp_failure_is_detailed_for_engineers_and_safe_for_users() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        runtime(Err("private diagnostic".to_string())),
    );
    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert!(effects.iter().any(|effect| matches!(
        effect,
        ThreadEffect::Reply { target: ThreadTarget::LinkedThreads { .. }, message, .. }
            if message.contains("private diagnostic")
    )));
    assert!(effects.iter().any(|effect| matches!(
        effect,
        ThreadEffect::Reply { target: ThreadTarget::CurrentThread, message, .. }
            if !message.contains("private diagnostic") && message.contains("temporarily unavailable")
    )));
}

#[tokio::test]
async fn admission_ignore_stops_before_acp() {
    let runtime = runtime(Ok(success()));
    let hook = Arc::new(StaticAdmissionHook::new(
        SupportThreadAdmissionDecision::Ignore {
            reason: Some("not support".to_string()),
        },
    ));
    let handler = SupportBotHandler::new("support", test_config(), runtime.clone())
        .with_admission_hook(hook.clone());
    let effects = handle_thread(&handler, thread("users", "hello"))
        .await
        .unwrap();

    assert_eq!(hook.calls(), 1);
    assert!(runtime.prompts.lock().unwrap().is_empty());
    assert!(matches!(
        effects.as_slice(),
        [ThreadEffect::SetThreadMetadata { .. }]
    ));
}
