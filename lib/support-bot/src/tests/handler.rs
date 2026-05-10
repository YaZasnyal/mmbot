use super::*;
use crate::config::{
    DebugCommandConfig, EngineerNotificationConfig, InstructionConfig, LlmConfig, SupportBotLimits,
    SupportRouteConfig, ToolConfig,
};
use crate::conversation::{store_state, STATE_KEY, TRACE_KEY};
use crate::debug::DebugResponse;
use crate::llm::{ChatMessage, ChatRole, LlmResponse};
use crate::state::{SupportThreadState, SupportThreadStatus};
use crate::testutil::PanicStore;
use crate::tools::{
    register_default_workflow_tools, SupportTool, ToolCall, ToolContext, ToolExecutionOutcome,
    ToolKind, ToolResult, ToolSpec,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use thread_bot::{
    AppendReaction, ChannelCheckpoint, ThreadInfo, ThreadInvocation, ThreadLink, ThreadMessage,
    ThreadMessageRecord, ThreadReaction, ThreadRecord, ThreadStore, ThreadTrigger, UpsertThread,
    UpsertThreadLink, UpsertThreadMessage,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct StaticLlm;

#[async_trait]
impl LlmClient for StaticLlm {
    async fn complete(&self, _request: LlmRequest) -> crate::Result<LlmResponse> {
        Ok(LlmResponse {
            message: ChatMessage::assistant("hello from support"),
            tool_calls: Vec::new(),
        })
    }
}

struct StaticDebug;

#[async_trait]
impl DebugCommandHandler for StaticDebug {
    async fn handle_debug_command(&self, command: DebugCommand) -> crate::Result<DebugResponse> {
        Ok(DebugResponse {
            message: format!("debug: {}", command.name),
        })
    }
}

struct EchoReadOnlyTool;

#[async_trait]
impl SupportTool for EchoReadOnlyTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "echo_read".to_string(),
            description: "Echo test tool".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": true
            }),
            kind: ToolKind::ReadOnly,
        }
    }

    async fn call(&self, _ctx: ToolContext, call: ToolCall) -> crate::Result<ToolExecutionOutcome> {
        Ok(ToolExecutionOutcome::ToolResult(ToolResult {
            call_id: call.id,
            content: json!({ "ok": true }),
            is_error: false,
        }))
    }
}

struct SequenceLlm {
    responses: Mutex<VecDeque<LlmResponse>>,
    requests: Mutex<Vec<LlmRequest>>,
}

impl SequenceLlm {
    fn new(responses: Vec<LlmResponse>) -> Self {
        Self {
            responses: Mutex::new(VecDeque::from(responses)),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<LlmRequest> {
        self.requests.lock().unwrap().clone()
    }
}

#[async_trait]
impl LlmClient for SequenceLlm {
    async fn complete(&self, request: LlmRequest) -> crate::Result<LlmResponse> {
        self.requests.lock().unwrap().push(request);
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| crate::SupportBotError::Llm("missing test response".to_string()))
    }
}

fn test_config() -> SupportBotConfig {
    SupportBotConfig {
        system_prompt: "system".to_string(),
        llm: LlmConfig {
            base_url: "http://localhost".to_string(),
            api_key: None,
            model: "test-model".to_string(),
            timeout: Duration::from_secs(1),
        },
        instructions: InstructionConfig {
            root_path: PathBuf::from("."),
            max_context_instructions: 5,
            max_instruction_bytes: 1024,
        },
        tools: ToolConfig::default(),
        limits: SupportBotLimits::default(),
        routes: SupportRouteConfig {
            user_channel_ids: vec!["users".to_string()],
            engineer_channel_id: Some("engineers".to_string()),
            debug_commands: DebugCommandConfig::default(),
        },
        engineer_notifications: EngineerNotificationConfig {
            channel_id: "engineers".to_string(),
        },
    }
}

fn thread(channel_id: &str, message: &str) -> Thread {
    let thread_kind = match channel_id {
        "users" => Some(USER_THREAD_KIND.to_string()),
        "engineers" => Some(ENGINEER_LINK_KIND.to_string()),
        _ => None,
    };
    Thread {
        info: ThreadInfo {
            thread_id: "thread-1".to_string(),
            root_post_id: "post-1".to_string(),
            channel_id: channel_id.to_string(),
            creator_user_id: "user-1".to_string(),
            thread_kind,
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
        reactions: Vec::new(),
    }
}

fn thread_record(channel_id: &str, metadata: serde_json::Value) -> ThreadRecord {
    let now: DateTime<Utc> = Utc::now();
    ThreadRecord {
        thread_id: "thread-1".to_string(),
        root_post_id: "post-1".to_string(),
        channel_id: channel_id.to_string(),
        creator_user_id: "user-1".to_string(),
        thread_kind: Some(USER_THREAD_KIND.to_string()),
        metadata,
        last_seen_post_id: None,
        last_seen_post_at: None,
        last_processed_post_id: None,
        last_processed_post_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn context() -> ThreadContext {
    ThreadContext {
        config: Arc::new(Default::default()),
        store: Arc::new(PanicStore),
        plugin_id: "test",
        bot_user_id: Some("bot".to_string()),
    }
}

struct SnapshotStore {
    record: ThreadRecord,
    messages: Vec<ThreadMessageRecord>,
    links: Vec<ThreadLink>,
}

#[async_trait]
impl ThreadStore for SnapshotStore {
    async fn upsert_thread(&self, _input: UpsertThread) -> Result<ThreadRecord, ThreadBotError> {
        panic!("store should not upsert threads")
    }

    async fn get_thread(&self, thread_id: &str) -> Result<Option<ThreadRecord>, ThreadBotError> {
        Ok((thread_id == self.record.thread_id).then(|| self.record.clone()))
    }

    async fn get_thread_by_post(
        &self,
        _post_id: &str,
    ) -> Result<Option<ThreadRecord>, ThreadBotError> {
        panic!("store should not look up by post")
    }

    async fn list_threads(
        &self,
        _updated_after: Option<DateTime<Utc>>,
        _updated_before: Option<DateTime<Utc>>,
    ) -> Result<Vec<ThreadRecord>, ThreadBotError> {
        panic!("store should not list threads")
    }

    async fn set_thread_metadata(
        &self,
        _thread_id: &str,
        _metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        panic!("store should not set thread metadata")
    }

    async fn upsert_thread_link(
        &self,
        _input: UpsertThreadLink,
    ) -> Result<ThreadLink, ThreadBotError> {
        panic!("store should not upsert thread links")
    }

    async fn get_thread_link(
        &self,
        _source_thread_id: &str,
        _target_thread_id: &str,
    ) -> Result<Option<ThreadLink>, ThreadBotError> {
        panic!("store should not get thread links")
    }

    async fn list_thread_links(
        &self,
        source_thread_id: &str,
    ) -> Result<Vec<ThreadLink>, ThreadBotError> {
        Ok(self
            .links
            .iter()
            .filter(|link| link.source_thread_id == source_thread_id)
            .cloned()
            .collect())
    }

    async fn list_reverse_thread_links(
        &self,
        target_thread_id: &str,
    ) -> Result<Vec<ThreadLink>, ThreadBotError> {
        Ok(self
            .links
            .iter()
            .filter(|link| link.target_thread_id == target_thread_id)
            .cloned()
            .collect())
    }

    async fn update_thread_seen(
        &self,
        _thread_id: &str,
        _post_id: &str,
        _seen_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        panic!("store should not update seen")
    }

    async fn update_thread_processed(
        &self,
        _thread_id: &str,
        _post_id: &str,
        _processed_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        panic!("store should not update processed")
    }

    async fn upsert_message(
        &self,
        _input: UpsertThreadMessage,
    ) -> Result<ThreadMessageRecord, ThreadBotError> {
        panic!("store should not upsert messages")
    }

    async fn get_message(
        &self,
        _post_id: &str,
    ) -> Result<Option<ThreadMessageRecord>, ThreadBotError> {
        panic!("store should not get messages")
    }

    async fn list_thread_messages(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadMessageRecord>, ThreadBotError> {
        Ok(if thread_id == self.record.thread_id {
            self.messages.clone()
        } else {
            Vec::new()
        })
    }

    async fn set_message_metadata(
        &self,
        _post_id: &str,
        _metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        panic!("store should not set message metadata")
    }

    async fn append_reaction(&self, _input: AppendReaction) -> Result<(), ThreadBotError> {
        panic!("store should not append reactions")
    }

    async fn list_thread_reactions(
        &self,
        _thread_id: &str,
    ) -> Result<Vec<ThreadReaction>, ThreadBotError> {
        Ok(Vec::new())
    }

    async fn list_channel_checkpoints(&self) -> Result<Vec<ChannelCheckpoint>, ThreadBotError> {
        panic!("store should not list checkpoints")
    }

    async fn upsert_channel_checkpoint(
        &self,
        _channel_id: &str,
        _last_seen_post_id: &str,
        _last_seen_post_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        panic!("store should not upsert checkpoints")
    }

    async fn advance_channel_checkpoint(
        &self,
        _channel_id: &str,
        _last_seen_post_id: &str,
        _last_seen_post_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        panic!("store should not advance checkpoints")
    }

    async fn set_all_channels_not_reconciled(&self) -> Result<(), ThreadBotError> {
        panic!("store should not mark checkpoints")
    }

    async fn set_channel_reconciled(&self, _channel_id: &str) -> Result<(), ThreadBotError> {
        panic!("store should not reconcile checkpoints")
    }
}

async fn handle_thread(
    handler: &SupportBotHandler,
    thread: Thread,
) -> Result<Vec<ThreadEffect>, ThreadBotError> {
    let (ctx, _server) = context_with_snapshot(&thread).await;
    handler.handle(&invocation_for(&thread), &ctx).await
}

async fn context_with_snapshot(thread: &Thread) -> (ThreadContext, MockServer) {
    let links = if thread.info.thread_kind.as_deref() == Some(USER_THREAD_KIND) {
        vec![ThreadLink {
            source_thread_id: thread.info.thread_id.clone(),
            link_kind: ENGINEER_LINK_KIND.to_string(),
            target_thread_id: "engineer-thread".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }]
    } else {
        Vec::new()
    };
    context_with_snapshot_and_links(thread, links).await
}

async fn context_with_snapshot_and_links(
    thread: &Thread,
    links: Vec<ThreadLink>,
) -> (ThreadContext, MockServer) {
    let server = MockServer::start().await;
    let posts = thread
        .messages
        .iter()
        .map(|message| {
            let mut post = mattermost_api::models::Post::new();
            post.id = message.post_id.clone();
            post.channel_id = Some(thread.info.channel_id.clone());
            post.user_id = Some(message.user_id.clone());
            post.message = Some(message.message.clone());
            post.root_id = message.root_id.clone();
            post.props = Some(message.props.clone());
            post.create_at = Some(message.created_at.timestamp_millis());
            post.update_at = Some(message.updated_at.timestamp_millis());
            (message.post_id.clone(), serde_json::to_value(post).unwrap())
        })
        .collect::<serde_json::Map<_, _>>();
    let order = thread
        .messages
        .iter()
        .map(|message| serde_json::Value::String(message.post_id.clone()))
        .collect::<Vec<_>>();
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v4/posts/{}/thread",
            thread.info.thread_id
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "order": order,
            "posts": posts
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v4/posts/{}/reactions",
            thread.info.root_post_id
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let ctx = ThreadContext {
        config: Arc::new(mattermost_api::apis::configuration::Configuration {
            base_path: server.uri(),
            ..Default::default()
        }),
        store: Arc::new(SnapshotStore {
            record: record_from_thread(thread),
            messages: thread.messages.iter().map(message_record).collect(),
            links,
        }),
        plugin_id: "test",
        bot_user_id: Some("bot".to_string()),
    };
    (ctx, server)
}

fn invocation_for(thread: &Thread) -> ThreadInvocation {
    ThreadInvocation {
        thread: record_from_thread(thread),
        trigger: ThreadTrigger::NewMessage {
            post_id: thread
                .messages
                .iter()
                .rev()
                .find(|message| message.user_id != "bot")
                .map(|message| message.post_id.clone())
                .unwrap_or_else(|| thread.info.root_post_id.clone()),
        },
    }
}

fn record_from_thread(thread: &Thread) -> ThreadRecord {
    ThreadRecord {
        thread_id: thread.info.thread_id.clone(),
        root_post_id: thread.info.root_post_id.clone(),
        channel_id: thread.info.channel_id.clone(),
        creator_user_id: thread.info.creator_user_id.clone(),
        thread_kind: thread.info.thread_kind.clone(),
        metadata: thread.info.metadata.clone(),
        last_seen_post_id: thread.info.last_seen_post_id.clone(),
        last_seen_post_at: thread.info.last_seen_post_at,
        last_processed_post_id: thread.info.last_processed_post_id.clone(),
        last_processed_post_at: thread.info.last_processed_post_at,
        created_at: thread.info.created_at,
        updated_at: thread.info.updated_at,
    }
}

fn message_record(message: &ThreadMessage) -> ThreadMessageRecord {
    ThreadMessageRecord {
        post_id: message.post_id.clone(),
        thread_id: message.thread_id.clone(),
        user_id: message.user_id.clone(),
        is_bot_message: message.user_id == "bot",
        root_id: message.root_id.clone(),
        parent_post_id: message.parent_post_id.clone(),
        metadata: message.metadata.clone(),
        post_created_at: message.created_at,
        post_updated_at: Some(message.updated_at),
        post_deleted_at: None,
        created_at: message.created_at,
        updated_at: message.updated_at,
    }
}

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

#[tokio::test]
async fn engineer_thread_only_handles_last_external_message() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    )
    .with_debug_handler(Arc::new(StaticDebug));
    let mut thread = thread("engineers", "!support state");
    thread.messages.push(ThreadMessage {
        post_id: "post-2".to_string(),
        thread_id: "thread-1".to_string(),
        user_id: "user-1".to_string(),
        message: "ordinary engineer discussion".to_string(),
        root_id: Some("post-1".to_string()),
        parent_post_id: Some("post-1".to_string()),
        props: json!({}),
        metadata: json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_new: true,
    });

    let effects = handle_thread(&handler, thread).await.unwrap();

    assert!(matches!(effects.as_slice(), [ThreadEffect::Noop]));
}

#[tokio::test]
async fn engineer_thread_ignores_new_bot_messages_after_command() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    )
    .with_debug_handler(Arc::new(StaticDebug));
    let mut thread = thread("engineers", "!support state");
    thread.messages.push(ThreadMessage {
        post_id: "post-2".to_string(),
        thread_id: "thread-1".to_string(),
        user_id: "bot".to_string(),
        message: "bot echo".to_string(),
        root_id: Some("post-1".to_string()),
        parent_post_id: Some("post-1".to_string()),
        props: json!({}),
        metadata: json!({}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_new: true,
    });

    let effects = handle_thread(&handler, thread).await.unwrap();

    assert!(matches!(
        &effects[0],
        ThreadEffect::Reply { message, .. } if message == "debug: state"
    ));
}

#[tokio::test]
async fn user_message_gets_llm_reply_and_state_update() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    );

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert!(effects.iter().any(|effect| {
        matches!(effect, ThreadEffect::Reply { message, .. } if message == "hello from support")
    }));
    assert!(effects
        .iter()
        .any(|effect| matches!(effect, ThreadEffect::SetThreadMetadata { .. })));
}

#[tokio::test]
async fn empty_user_message_does_not_call_llm() {
    let llm = Arc::new(SequenceLlm::new(vec![LlmResponse {
        message: ChatMessage::assistant("should not be used"),
        tool_calls: Vec::new(),
    }]));
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        llm.clone(),
        Arc::new(ToolRegistry::new()),
        "system",
    );

    let effects = handle_thread(&handler, thread("users", "")).await.unwrap();

    assert!(matches!(effects.as_slice(), [ThreadEffect::Noop]));
    assert!(llm.requests().is_empty());
}

#[tokio::test]
async fn assistant_thinking_is_not_sent_to_user() {
    let llm = Arc::new(SequenceLlm::new(vec![LlmResponse {
        message: ChatMessage::assistant(
            "<think>private reasoning</think>\nPlease share the request id.",
        ),
        tool_calls: Vec::new(),
    }]));
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        llm,
        Arc::new(ToolRegistry::new()),
        "system",
    );

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert!(effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply { message, .. }
                if message == "Please share the request id."
        )
    }));
    assert!(!effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply { message, .. }
                if message.contains("private reasoning") || message.contains("<think>")
        )
    }));
}

#[tokio::test]
async fn user_tool_trace_is_not_written_via_update_message() {
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "send_user_message".to_string(),
        arguments: json!({ "message": "please send request id" }),
    };
    let llm = Arc::new(SequenceLlm::new(vec![
        LlmResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: None,
                name: None,
                tool_call_id: None,
                tool_calls: vec![call.clone()],
            },
            tool_calls: vec![call],
        },
        LlmResponse {
            message: ChatMessage::assistant("done"),
            tool_calls: Vec::new(),
        },
    ]));
    let mut registry = ToolRegistry::new();
    register_default_workflow_tools(&mut registry).unwrap();
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        llm.clone(),
        Arc::new(registry),
        "system",
    );

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();
    assert!(!effects
        .iter()
        .any(|effect| matches!(effect, ThreadEffect::UpdateMessage { .. })));
    assert_eq!(
            effects
                .iter()
                .filter(|effect| {
                    matches!(effect, ThreadEffect::Reply { message, .. } if message == "please send request id")
                })
                .count(),
            1
        );
    assert_eq!(llm.requests().len(), 1);
}

#[tokio::test]
async fn tool_call_overflow_is_visible_to_next_llm_round() {
    let calls = vec![
        ToolCall {
            id: "call-1".to_string(),
            name: "echo_read".to_string(),
            arguments: json!({ "message": "first" }),
        },
        ToolCall {
            id: "call-2".to_string(),
            name: "echo_read".to_string(),
            arguments: json!({ "message": "second" }),
        },
    ];
    let llm = Arc::new(SequenceLlm::new(vec![
        LlmResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: None,
                name: None,
                tool_call_id: None,
                tool_calls: calls.clone(),
            },
            tool_calls: calls,
        },
        LlmResponse {
            message: ChatMessage::assistant("done"),
            tool_calls: Vec::new(),
        },
    ]));
    let mut registry = ToolRegistry::new();
    registry.register(EchoReadOnlyTool).unwrap();
    let mut config = test_config();
    config.limits.max_tool_calls_per_round = 1;
    let handler =
        SupportBotHandler::new("support", config, llm.clone(), Arc::new(registry), "system");

    handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    let requests = llm.requests();
    assert_eq!(requests.len(), 2);
    let second_round_messages = &requests[1].messages;
    let assistant = second_round_messages
        .iter()
        .find(|message| message.role == ChatRole::Assistant && !message.tool_calls.is_empty())
        .expect("assistant tool-call message should be preserved");
    assert_eq!(assistant.tool_calls.len(), 2);

    let overflow_result = second_round_messages
        .iter()
        .find(|message| message.tool_call_id.as_deref() == Some("call-2"))
        .and_then(|message| message.content.as_deref())
        .expect("overflow tool result should be sent back to the model");
    assert!(overflow_result.contains("max_tool_calls_per_round"));
    assert!(overflow_result.contains("\"is_error\":true"));
}

#[tokio::test]
async fn send_user_message_tool_strips_hidden_reasoning() {
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "send_user_message".to_string(),
        arguments: json!({
            "message": "<thinking>private reasoning</thinking>\nPlease send request id"
        }),
    };
    let llm = Arc::new(SequenceLlm::new(vec![
        LlmResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: None,
                name: None,
                tool_call_id: None,
                tool_calls: vec![call.clone()],
            },
            tool_calls: vec![call],
        },
        LlmResponse {
            message: ChatMessage::assistant("done"),
            tool_calls: Vec::new(),
        },
    ]));
    let mut registry = ToolRegistry::new();
    register_default_workflow_tools(&mut registry).unwrap();
    let handler =
        SupportBotHandler::new("support", test_config(), llm, Arc::new(registry), "system");

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert!(effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply { message, .. } if message == "Please send request id"
        )
    }));
    assert!(!effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply { message, .. }
                if message.contains("private reasoning") || message.contains("<thinking>")
        )
    }));
}

#[tokio::test]
async fn notify_engineer_uses_linked_engineer_thread_reply_effect() {
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "notify_engineer".to_string(),
        arguments: json!({ "message": "need help" }),
    };
    let llm = Arc::new(SequenceLlm::new(vec![
        LlmResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: None,
                name: None,
                tool_call_id: None,
                tool_calls: vec![call.clone()],
            },
            tool_calls: vec![call],
        },
        LlmResponse {
            message: ChatMessage::assistant("sent"),
            tool_calls: Vec::new(),
        },
    ]));
    let mut registry = ToolRegistry::new();
    register_default_workflow_tools(&mut registry).unwrap();
    let handler =
        SupportBotHandler::new("support", test_config(), llm, Arc::new(registry), "system");

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert!(effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply {
                message, metadata, ..
            }
                if message == "need help"
                    && metadata["support_bot"]["kind"] == "engineer_notification"
        )
    }));
}

#[tokio::test]
async fn tool_loop_limit_replies_with_generic_error_and_stops_thread() {
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "missing_tool".to_string(),
        arguments: json!({}),
    };
    let llm = Arc::new(SequenceLlm::new(vec![LlmResponse {
        message: ChatMessage {
            role: ChatRole::Assistant,
            content: None,
            name: None,
            tool_call_id: None,
            tool_calls: vec![call.clone()],
        },
        tool_calls: vec![call],
    }]));
    let mut config = test_config();
    config.limits.max_tool_rounds = 1;

    let handler = SupportBotHandler::new(
        "support",
        config,
        llm,
        Arc::new(ToolRegistry::new()),
        "system",
    );

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert!(effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply {
                message, metadata, ..
            }
                if message.contains("internal issue")
                    && metadata["support_bot"]["kind"] == "tool_loop_limit"
        )
    }));
    let state_meta = effects
        .iter()
        .find_map(|effect| match effect {
            ThreadEffect::SetThreadMetadata { metadata, .. } => Some(metadata),
            _ => None,
        })
        .expect("state metadata must exist");
    assert_eq!(state_meta[STATE_KEY]["status"], "stopped");
}

#[tokio::test]
async fn finished_user_thread_metadata_skips_workflow() {
    let llm = Arc::new(SequenceLlm::new(vec![LlmResponse {
        message: ChatMessage::assistant("should not run"),
        tool_calls: Vec::new(),
    }]));
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        llm.clone(),
        Arc::new(ToolRegistry::new()),
        "system",
    );
    let mut thread = thread("users", "new reply after finish");
    thread.info.metadata = store_state(
        &json!({}),
        &SupportThreadState {
            status: SupportThreadStatus::Finished,
            finished_summary: Some("done".to_string()),
            ..SupportThreadState::default()
        },
    )
    .unwrap();

    let effects = handle_thread(&handler, thread).await.unwrap();

    assert_eq!(llm.requests().len(), 0);
    assert!(matches!(effects.as_slice(), [ThreadEffect::Noop]));
}

#[tokio::test]
async fn stopped_user_thread_metadata_skips_workflow() {
    let llm = Arc::new(SequenceLlm::new(vec![LlmResponse {
        message: ChatMessage::assistant("should not run"),
        tool_calls: Vec::new(),
    }]));
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        llm.clone(),
        Arc::new(ToolRegistry::new()),
        "system",
    );
    let mut thread = thread("users", "new reply after stop");
    thread.info.metadata = store_state(
        &json!({}),
        &SupportThreadState {
            status: SupportThreadStatus::Stopped,
            ..SupportThreadState::default()
        },
    )
    .unwrap();

    let effects = handle_thread(&handler, thread).await.unwrap();

    assert_eq!(llm.requests().len(), 0);
    assert!(matches!(effects.as_slice(), [ThreadEffect::Noop]));
}

#[test]
fn control_reactions_update_support_status_metadata() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    );
    let record = thread_record("users", json!({}));

    let resolved = handler
        .on_control_reaction(
            &record,
            &ReactionChange {
                post_id: "post-1".to_string(),
                user_id: "user-2".to_string(),
                emoji_name: "white_check_mark".to_string(),
                action: ReactionAction::Added,
                created_at: Utc::now(),
            },
        )
        .expect("white_check_mark should update support status");
    let ThreadEffect::SetThreadMetadata { metadata, .. } = resolved else {
        panic!("expected SetThreadMetadata");
    };
    assert_eq!(metadata[STATE_KEY]["status"], "finished");
    assert_eq!(
        metadata[STATE_KEY]["finished_summary"],
        "resolved by control reaction"
    );

    let stopped = handler
        .on_control_reaction(
            &record,
            &ReactionChange {
                post_id: "post-1".to_string(),
                user_id: "user-2".to_string(),
                emoji_name: "stop_sign".to_string(),
                action: ReactionAction::Added,
                created_at: Utc::now(),
            },
        )
        .expect("stop_sign should update support status");
    let ThreadEffect::SetThreadMetadata { metadata, .. } = stopped else {
        panic!("expected SetThreadMetadata");
    };
    assert_eq!(metadata[STATE_KEY]["status"], "stopped");
}

#[test]
fn control_reactions_route_channel_backed_user_threads_without_thread_kind() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    );
    let mut record = thread_record("users", json!({}));
    record.thread_kind = None;

    let resolved = handler
        .on_control_reaction(
            &record,
            &ReactionChange {
                post_id: "post-1".to_string(),
                user_id: "user-2".to_string(),
                emoji_name: "white_check_mark".to_string(),
                action: ReactionAction::Added,
                created_at: Utc::now(),
            },
        )
        .expect("channel-routed user thread should accept control reaction");
    let ThreadEffect::SetThreadMetadata { metadata, .. } = resolved else {
        panic!("expected SetThreadMetadata");
    };

    assert_eq!(metadata[STATE_KEY]["status"], "finished");
}

#[tokio::test]
async fn finish_request_persists_finished_state_without_mark_resolved() {
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "finish_request".to_string(),
        arguments: json!({ "summary": "resolved by cache flush" }),
    };
    let llm = Arc::new(SequenceLlm::new(vec![
        LlmResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: None,
                name: None,
                tool_call_id: None,
                tool_calls: vec![call.clone()],
            },
            tool_calls: vec![call],
        },
        LlmResponse {
            message: ChatMessage::assistant("should not run"),
            tool_calls: Vec::new(),
        },
    ]));
    let mut registry = ToolRegistry::new();
    register_default_workflow_tools(&mut registry).unwrap();
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        llm.clone(),
        Arc::new(registry),
        "system",
    );

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert_eq!(llm.requests().len(), 1);

    effects
        .iter()
        .position(|effect| matches!(effect, ThreadEffect::SetThreadMetadata { .. }))
        .expect("SetThreadMetadata must be emitted");
    let state_meta = effects
        .iter()
        .find_map(|effect| match effect {
            ThreadEffect::SetThreadMetadata { metadata, .. } => Some(metadata),
            _ => None,
        })
        .expect("state metadata must exist");
    assert_eq!(state_meta[STATE_KEY]["status"], "finished");
    assert_eq!(
        state_meta[STATE_KEY]["finished_summary"],
        "resolved by cache flush"
    );
}

#[tokio::test]
async fn finish_request_persists_finished_state_and_queues_status_notification() {
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "finish_request".to_string(),
        arguments: json!({ "summary": "resolved by cache flush" }),
    };
    let llm = Arc::new(SequenceLlm::new(vec![LlmResponse {
        message: ChatMessage {
            role: ChatRole::Assistant,
            content: None,
            name: None,
            tool_call_id: None,
            tool_calls: vec![call.clone()],
        },
        tool_calls: vec![call],
    }]));
    let mut registry = ToolRegistry::new();
    register_default_workflow_tools(&mut registry).unwrap();
    let mut config = test_config();
    config.engineer_notifications.channel_id = "engineers".to_string();
    let handler = SupportBotHandler::new("support", config, llm, Arc::new(registry), "system");

    let mut thread = thread("users", "help");
    thread.info.metadata = store_state(&json!({}), &SupportThreadState::default()).unwrap();
    let (ctx, _server) = context_with_snapshot_and_links(
        &thread,
        vec![ThreadLink {
            source_thread_id: thread.info.thread_id.clone(),
            link_kind: ENGINEER_LINK_KIND.to_string(),
            target_thread_id: "engineer-thread".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }],
    )
    .await;

    let effects = handler
        .handle(&invocation_for(&thread), &ctx)
        .await
        .unwrap();

    let state_meta = effects
        .iter()
        .find_map(|effect| match effect {
            ThreadEffect::SetThreadMetadata { metadata, .. } => Some(metadata),
            _ => None,
        })
        .expect("state metadata must exist");
    assert_eq!(state_meta[STATE_KEY]["status"], "finished");

    let trace_meta = effects
        .iter()
        .find_map(|effect| match effect {
            ThreadEffect::SetMessageMetadata { metadata, .. } => Some(metadata),
            _ => None,
        })
        .expect("trace metadata must exist");
    assert!(trace_meta[STATE_KEY][TRACE_KEY]
        .to_string()
        .contains("notification_status"));
    assert!(trace_meta[STATE_KEY][TRACE_KEY]
        .to_string()
        .contains("queued"));
    assert!(effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply {
            target: ThreadTarget::LinkedThreads { link_kind },
                metadata,
                ..
            } if link_kind == ENGINEER_LINK_KIND
                && metadata[STATE_KEY]["kind"] == "status_update"
        )
    }));
}

#[test]
fn state_is_stored_under_support_bot_key() {
    let metadata = json!({ "other": true });
    let stored = store_state(&metadata, &SupportThreadState::default()).unwrap();

    assert_eq!(stored["other"], true);
    assert_eq!(stored[STATE_KEY]["version"], 1);
}

#[test]
fn tool_registry_specs_are_still_empty_by_default() {
    let registry = ToolRegistry::new();
    let specs: Vec<ToolSpec> = registry.specs();

    assert!(specs.is_empty());
}

#[test]
fn build_llm_messages_keeps_tool_calls_only_in_trace_messages() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    );
    let mut thread = thread("users", "help");
    thread.messages[0].metadata = json!({
        STATE_KEY: {
            TRACE_KEY: [{
                "role": "assistant",
                "content": null,
                "name": null,
                "tool_call_id": null,
                "tool_calls": [{
                    "id": "call-1",
                    "name": "instructions",
                    "arguments": { "id": null }
                }]
            }, {
                "role": "tool",
                "content": "{\"items\":[]}",
                "name": null,
                "tool_call_id": "call-1",
                "tool_calls": []
            }]
        }
    });

    let messages = build_llm_messages(
        &handler.system_prompt,
        &thread,
        &SupportThreadState::default(),
        context().bot_user_id.as_deref(),
    )
    .unwrap();

    let user_msg = messages
        .iter()
        .find(|m| m.role == ChatRole::User)
        .expect("user message should exist");
    assert!(user_msg.tool_calls.is_empty());

    let trace_assistant = messages
        .iter()
        .find(|m| m.role == ChatRole::Assistant && !m.tool_calls.is_empty())
        .expect("assistant tool call message should exist");
    assert_eq!(trace_assistant.tool_calls[0].id, "call-1");
}

#[test]
fn build_llm_messages_orders_thread_messages_by_created_at() {
    let mut thread = thread("users", "second");
    let first_created_at = Utc::now();
    thread.messages[0].post_id = "post-2".to_string();
    thread.messages[0].created_at = first_created_at + chrono::Duration::seconds(1);
    thread.messages.push(ThreadMessage {
        post_id: "post-1".to_string(),
        thread_id: "thread-1".to_string(),
        user_id: "user-1".to_string(),
        message: "first".to_string(),
        root_id: Some("post-1".to_string()),
        parent_post_id: Some("post-1".to_string()),
        props: json!({}),
        metadata: json!({}),
        created_at: first_created_at,
        updated_at: first_created_at,
        is_new: true,
    });

    let messages = build_llm_messages(
        "system",
        &thread,
        &SupportThreadState::default(),
        context().bot_user_id.as_deref(),
    )
    .unwrap();
    let user_contents = messages
        .iter()
        .filter(|message| message.role == ChatRole::User)
        .filter_map(|message| message.content.as_deref())
        .collect::<Vec<_>>();

    assert!(user_contents[0].contains("first"));
    assert!(user_contents[1].contains("second"));
}
