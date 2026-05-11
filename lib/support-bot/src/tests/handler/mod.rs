use super::*;
use crate::admission::{SupportThreadAdmissionDecision, SupportThreadAdmissionHook};
use crate::config::{
    DebugCommandConfig, InstructionConfig, LlmConfig, SupportBotLimits, SupportRouteConfig,
    ToolConfig,
};
use crate::conversation::{build_llm_messages, STATE_KEY, TRACE_KEY};
use crate::debug::DebugResponse;
use crate::llm::{ChatMessage, ChatRole, LlmRequest, LlmResponse};
use crate::metadata::store_thread_state as store_state;
use crate::state::{SupportThreadState, SupportThreadStatus};
use crate::testutil::PanicStore;
use crate::tools::{
    register_default_workflow_tools, SupportTool, ToolCall, ToolContext, ToolExecutionOutcome,
    ToolKind, ToolRegistry, ToolResult, ToolSpec,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use thread_bot::{
    ChannelCheckpoint, ThreadInfo, ThreadInvocation, ThreadLink, ThreadMessage,
    ThreadMessageRecord, ThreadRecord, ThreadStore, ThreadTrigger, UpsertThread, UpsertThreadLink,
    UpsertThreadMessage,
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

struct StaticAdmissionHook {
    decision: SupportThreadAdmissionDecision,
    calls: AtomicUsize,
}

impl StaticAdmissionHook {
    fn new(decision: SupportThreadAdmissionDecision) -> Self {
        Self {
            decision,
            calls: AtomicUsize::new(0),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl SupportThreadAdmissionHook for StaticAdmissionHook {
    async fn evaluate(&self, _thread: &Thread) -> crate::Result<SupportThreadAdmissionDecision> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.decision.clone())
    }
}

struct ErrorAdmissionHook;

#[async_trait]
impl SupportThreadAdmissionHook for ErrorAdmissionHook {
    async fn evaluate(&self, _thread: &Thread) -> crate::Result<SupportThreadAdmissionDecision> {
        Err(crate::SupportBotError::Config(
            "admission failed".to_string(),
        ))
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
            engineer_channel_id: "engineers".to_string(),
            debug_commands: DebugCommandConfig::default(),
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

mod control_reactions;
mod debug_report;
mod engineer_thread;
mod routing;
mod user_workflow;
