use crate::config::{EngineerNotificationTarget, SupportBotConfig};
use crate::debug::{DebugCommand, DebugCommandHandler};
use crate::llm::{ChatMessage, ChatRole, LlmClient, LlmRequest};
use crate::notifier::MattermostSupportNotifier;
use crate::state::SupportThreadState;
use crate::tools::{SupportAction, ToolContext, ToolExecutionOutcome, ToolRegistry, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use thread_bot::{
    Thread, ThreadBotError, ThreadContext, ThreadEffect, ThreadHandler, ThreadMessage,
};

const STATE_KEY: &str = "support_bot";
const TRACE_KEY: &str = "llm_trace";

pub struct SupportBotHandler {
    id: &'static str,
    config: SupportBotConfig,
    llm: Arc<dyn LlmClient>,
    tools: Arc<ToolRegistry>,
    debug_handler: Option<Arc<dyn DebugCommandHandler>>,
    system_prompt: String,
}

impl SupportBotHandler {
    pub fn new(
        id: &'static str,
        config: SupportBotConfig,
        llm: Arc<dyn LlmClient>,
        tools: Arc<ToolRegistry>,
        system_prompt: impl Into<String>,
    ) -> Self {
        Self {
            id,
            config,
            llm,
            tools,
            debug_handler: None,
            system_prompt: system_prompt.into(),
        }
    }

    pub fn with_debug_handler(mut self, handler: Arc<dyn DebugCommandHandler>) -> Self {
        self.debug_handler = Some(handler);
        self
    }

    fn route(&self, channel_id: &str) -> SupportRoute {
        if self
            .config
            .routes
            .engineer_channel_id
            .as_deref()
            .is_some_and(|engineer_channel_id| engineer_channel_id == channel_id)
        {
            return SupportRoute::Engineer;
        }

        if self.config.routes.user_channel_ids.is_empty()
            || self
                .config
                .routes
                .user_channel_ids
                .iter()
                .any(|user_channel_id| user_channel_id == channel_id)
        {
            return SupportRoute::User;
        }

        SupportRoute::Ignored
    }

    async fn handle_engineer_thread(
        &self,
        thread: &Thread,
        ctx: &ThreadContext,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        let Some(message) = last_new_external_message(thread, ctx) else {
            return Ok(vec![ThreadEffect::Noop]);
        };

        let Some(command) =
            DebugCommand::parse(&message.message, &self.config.routes.debug_commands)
                .map(|matched| matched.command)
        else {
            return Ok(vec![ThreadEffect::Noop]);
        };

        if command.name == "debug" {
            return self.handle_debug_toggle(thread, ctx, command).await;
        }

        let response = match &self.debug_handler {
            Some(handler) => handler.handle_debug_command(command).await?,
            None => crate::debug::DebugResponse {
                message: "Debug command handler is not configured.".to_string(),
            },
        };

        Ok(vec![ThreadEffect::Reply {
            message: response.message,
            metadata: json!({
                STATE_KEY: {
                    "kind": "debug_response"
                }
            }),
        }])
    }

    async fn handle_user_thread(
        &self,
        thread: &Thread,
        ctx: &ThreadContext,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        let Some(trigger_message) = last_new_external_message(thread, ctx) else {
            return Ok(vec![ThreadEffect::Noop]);
        };

        let mut state = load_state(&thread.info.metadata)?;
        if let Some(engineer_thread) = self
            .engineer_notifier(ctx)
            .ensure_engineer_thread(thread, state.engineer_thread.as_ref())
            .await?
        {
            state.engineer_thread = Some(engineer_thread);
        }
        self.mirror_user_message(ctx, thread, &mut state, trigger_message)
            .await?;

        let mut messages = self.build_llm_messages(thread, &state, ctx)?;
        let mut trace = Vec::new();
        let mut effects = Vec::new();

        for _round in 0..self.config.limits.max_tool_rounds {
            let response = self
                .llm
                .complete(LlmRequest {
                    model: self.config.llm.model.clone(),
                    messages: messages.clone(),
                    tools: self.tools.specs(),
                })
                .await?;

            if response.tool_calls.is_empty() {
                if let Some(content) = response
                    .message
                    .content
                    .filter(|content| !content.is_empty())
                {
                    self.push_reply_effect(
                        ctx,
                        thread,
                        &mut state,
                        &mut effects,
                        content,
                        json!({
                            STATE_KEY: {
                                "kind": "assistant_response"
                            }
                        }),
                    )
                    .await?;
                }
                if !trace.is_empty() {
                    effects.push(ThreadEffect::UpdateMessage {
                        post_id: trigger_message.post_id.clone(),
                        message: None,
                        metadata: Some(store_trace(&trigger_message.metadata, &trace)?),
                    });
                }
                effects.push(ThreadEffect::SetThreadMetadata {
                    metadata: store_state(&thread.info.metadata, &state)?,
                });
                return Ok(effects);
            }

            let tool_calls = response
                .tool_calls
                .into_iter()
                .take(self.config.limits.max_tool_calls_per_round)
                .collect::<Vec<_>>();

            let assistant_tool_call_message = ChatMessage {
                role: ChatRole::Assistant,
                content: response.message.content,
                name: None,
                tool_call_id: None,
                tool_calls: tool_calls.clone(),
            };
            messages.push(assistant_tool_call_message.clone());
            trace.push(assistant_tool_call_message);
            self.send_debug_message(
                ctx,
                thread,
                &mut state,
                format_debug_chat_message("assistant tool calls", messages.last().unwrap()),
            )
            .await?;

            let tool_context = ToolContext::new(Arc::new(thread.clone()));
            for call in tool_calls {
                let tool_message = match self.tools.call(tool_context.clone(), call.clone()).await {
                    Ok(ToolExecutionOutcome::ToolResult(result)) => {
                        result_to_message(result, self.config.limits.max_tool_result_bytes)
                    }
                    Ok(ToolExecutionOutcome::Action(action)) => {
                        let result = self
                            .apply_action(ctx, thread, &mut state, &mut effects, &call.id, action)
                            .await?;
                        result_to_message(result, self.config.limits.max_tool_result_bytes)
                    }
                    Err(error) => result_to_message(
                        ToolResult {
                            call_id: call.id,
                            content: json!({
                                "error": error.to_string()
                            }),
                            is_error: true,
                        },
                        self.config.limits.max_tool_result_bytes,
                    ),
                };
                messages.push(tool_message.clone());
                trace.push(tool_message);
                self.send_debug_message(
                    ctx,
                    thread,
                    &mut state,
                    format_debug_chat_message("tool result", messages.last().unwrap()),
                )
                .await?;
            }
        }

        if !trace.is_empty() {
            effects.push(ThreadEffect::UpdateMessage {
                post_id: trigger_message.post_id.clone(),
                message: None,
                metadata: Some(store_trace(&trigger_message.metadata, &trace)?),
            });
        }

        self.push_reply_effect(
            ctx,
            thread,
            &mut state,
            &mut effects,
            "I could not complete the support workflow because the tool loop limit was reached."
                .to_string(),
            json!({
                STATE_KEY: {
                    "kind": "tool_loop_limit"
                }
            }),
        )
        .await?;
        effects.push(ThreadEffect::SetThreadMetadata {
            metadata: store_state(&thread.info.metadata, &state)?,
        });
        Ok(effects)
    }

    fn build_llm_messages(
        &self,
        thread: &Thread,
        state: &SupportThreadState,
        ctx: &ThreadContext,
    ) -> Result<Vec<ChatMessage>, ThreadBotError> {
        let mut messages = vec![
            ChatMessage::system(self.system_prompt.clone()),
            ChatMessage::system(format!(
                "Current support state JSON: {}",
                serde_json::to_string(state).unwrap_or_else(|_| "{}".to_string())
            )),
        ];

        for message in &thread.messages {
            if message.message.trim().is_empty() {
                continue;
            }

            let role = if ctx.bot_user_id.as_deref() == Some(message.user_id.as_str()) {
                ChatRole::Assistant
            } else {
                ChatRole::User
            };
            messages.push(ChatMessage {
                role,
                content: Some(format!(
                    "[post_id={}, user_id={}] {}",
                    message.post_id, message.user_id, message.message
                )),
                name: None,
                tool_call_id: None,
                tool_calls: Vec::new(),
            });

            messages.extend(load_trace(&message.metadata)?);
        }

        Ok(messages)
    }

    async fn apply_action(
        &self,
        ctx: &ThreadContext,
        thread: &Thread,
        state: &mut SupportThreadState,
        effects: &mut Vec<ThreadEffect>,
        call_id: &str,
        action: SupportAction,
    ) -> Result<ToolResult, ThreadBotError> {
        let result = match action {
            SupportAction::SendUserMessage { message } => {
                self.push_reply_effect(
                    ctx,
                    thread,
                    state,
                    effects,
                    message,
                    json!({
                        STATE_KEY: {
                            "kind": "tool_action",
                            "tool_call_id": call_id,
                            "action": "send_user_message"
                        }
                    }),
                )
                .await?;
                ToolResult {
                    call_id: call_id.to_string(),
                    content: json!({ "status": "sent" }),
                    is_error: false,
                }
            }
            SupportAction::NotifyEngineer { message } => {
                match &self.config.engineer_notifications.target {
                    EngineerNotificationTarget::SameThread => {
                        effects.push(ThreadEffect::Reply {
                            message: message.clone(),
                            metadata: json!({
                                STATE_KEY: {
                                    "kind": "engineer_notification",
                                    "tool_call_id": call_id
                                }
                            }),
                        });
                    }
                    EngineerNotificationTarget::MattermostChannel { .. } => {
                        let notify_result = self
                            .engineer_notifier(ctx)
                            .notify_engineer(
                                thread,
                                state.engineer_thread.as_ref(),
                                message.clone(),
                            )
                            .await;

                        let thread_ref = match notify_result {
                            Ok(Some(thread_ref)) => thread_ref,
                            Ok(None) => {
                                return Ok(ToolResult {
                                    call_id: call_id.to_string(),
                                    content: json!({
                                        "status": "failed",
                                        "error": "engineer thread was not created"
                                    }),
                                    is_error: true,
                                });
                            }
                            Err(error) => {
                                return Ok(ToolResult {
                                    call_id: call_id.to_string(),
                                    content: json!({
                                        "status": "failed",
                                        "error": error.to_string()
                                    }),
                                    is_error: true,
                                });
                            }
                        };
                        state.engineer_thread = Some(thread_ref);
                    }
                }

                ToolResult {
                    call_id: call_id.to_string(),
                    content: json!({
                        "status": "sent",
                        "message": message
                    }),
                    is_error: false,
                }
            }
            SupportAction::WaitForUser { reason } => ToolResult {
                call_id: call_id.to_string(),
                content: json!({
                    "status": "waiting",
                    "reason": reason
                }),
                is_error: false,
            },
            SupportAction::FinishRequest { summary } => {
                effects.push(ThreadEffect::MarkResolved);
                ToolResult {
                    call_id: call_id.to_string(),
                    content: json!({
                        "status": "finished",
                        "summary": summary
                    }),
                    is_error: false,
                }
            }
        };
        Ok(result)
    }

    fn engineer_notifier(&self, ctx: &ThreadContext) -> MattermostSupportNotifier {
        MattermostSupportNotifier::new(
            ctx.config.clone(),
            self.config.engineer_notifications.target.clone(),
        )
    }

    async fn push_reply_effect(
        &self,
        ctx: &ThreadContext,
        thread: &Thread,
        state: &mut SupportThreadState,
        effects: &mut Vec<ThreadEffect>,
        message: String,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        effects.push(ThreadEffect::Reply {
            message: message.clone(),
            metadata,
        });
        if let Some(engineer_thread) = self
            .engineer_notifier(ctx)
            .mirror_bot_message(thread, state.engineer_thread.as_ref(), message)
            .await?
        {
            state.engineer_thread = Some(engineer_thread);
        }
        Ok(())
    }

    async fn mirror_user_message(
        &self,
        ctx: &ThreadContext,
        thread: &Thread,
        state: &mut SupportThreadState,
        message: &ThreadMessage,
    ) -> Result<(), ThreadBotError> {
        if message.post_id == thread.info.root_post_id {
            return Ok(());
        }

        if let Some(engineer_thread) = self
            .engineer_notifier(ctx)
            .mirror_user_message(thread, state.engineer_thread.as_ref(), message)
            .await?
        {
            state.engineer_thread = Some(engineer_thread);
        }
        Ok(())
    }

    async fn send_debug_message(
        &self,
        ctx: &ThreadContext,
        thread: &Thread,
        state: &mut SupportThreadState,
        message: String,
    ) -> Result<(), ThreadBotError> {
        if !state.debug {
            return Ok(());
        }

        if let Some(engineer_thread) = self
            .engineer_notifier(ctx)
            .send_debug_message(thread, state.engineer_thread.as_ref(), message)
            .await?
        {
            state.engineer_thread = Some(engineer_thread);
        }
        Ok(())
    }

    async fn handle_debug_toggle(
        &self,
        thread: &Thread,
        ctx: &ThreadContext,
        command: DebugCommand,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        let Some(user_thread_id) = source_user_thread_id(thread) else {
            return Ok(vec![ThreadEffect::Reply {
                message: "Cannot find source support thread for this engineer thread.".to_string(),
                metadata: debug_response_metadata(),
            }]);
        };
        let Some(record) = ctx.store.get_thread(&user_thread_id).await? else {
            return Ok(vec![ThreadEffect::Reply {
                message: format!("Source support thread not found: {user_thread_id}"),
                metadata: debug_response_metadata(),
            }]);
        };

        let mut state = load_state(&record.metadata)?;
        let requested = command.args.first().map(String::as_str);
        state.debug = match requested {
            Some("on") | Some("true") | Some("1") => true,
            Some("off") | Some("false") | Some("0") => false,
            Some("status") => state.debug,
            _ => !state.debug,
        };
        ctx.store
            .set_thread_metadata(&user_thread_id, store_state(&record.metadata, &state)?)
            .await?;

        Ok(vec![ThreadEffect::Reply {
            message: format!(
                "Debug mode is {} for support thread {}.",
                if state.debug { "on" } else { "off" },
                user_thread_id
            ),
            metadata: debug_response_metadata(),
        }])
    }
}

#[async_trait]
impl ThreadHandler for SupportBotHandler {
    fn id(&self) -> &'static str {
        self.id
    }

    async fn should_track(
        &self,
        thread: &Thread,
        _ctx: &ThreadContext,
    ) -> Result<bool, ThreadBotError> {
        Ok(!matches!(
            self.route(&thread.info.channel_id),
            SupportRoute::Ignored
        ))
    }

    async fn handle(
        &self,
        thread: &Thread,
        ctx: &ThreadContext,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        match self.route(&thread.info.channel_id) {
            SupportRoute::User => self.handle_user_thread(thread, ctx).await,
            SupportRoute::Engineer => self.handle_engineer_thread(thread, ctx).await,
            SupportRoute::Ignored => Ok(vec![ThreadEffect::Noop]),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SupportRoute {
    User,
    Engineer,
    Ignored,
}

fn last_new_external_message<'a>(
    thread: &'a Thread,
    ctx: &ThreadContext,
) -> Option<&'a ThreadMessage> {
    let message = thread.messages.last()?;
    if !message.is_new || ctx.bot_user_id.as_deref() == Some(message.user_id.as_str()) {
        return None;
    }

    Some(message)
}

fn source_user_thread_id(thread: &Thread) -> Option<String> {
    thread
        .messages
        .iter()
        .find(|message| message.post_id == thread.info.root_post_id)
        .or_else(|| thread.messages.first())
        .and_then(|message| message.props.get(STATE_KEY))
        .and_then(|props| props.get("source_thread_id"))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
}

fn debug_response_metadata() -> serde_json::Value {
    json!({
        STATE_KEY: {
            "kind": "debug_response"
        }
    })
}

fn format_debug_chat_message(label: &str, message: &ChatMessage) -> String {
    let value = serde_json::to_string_pretty(message).unwrap_or_else(|_| "{}".to_string());
    format!("**Debug: {label}**\n\n```json\n{value}\n```")
}

fn result_to_message(result: ToolResult, max_bytes: usize) -> ChatMessage {
    let content = if result.is_error {
        json!({
            "is_error": true,
            "content": result.content
        })
    } else {
        result.content
    };

    ChatMessage::tool(
        result.call_id,
        truncate_utf8(content.to_string(), max_bytes),
    )
}

fn load_trace(metadata: &serde_json::Value) -> Result<Vec<ChatMessage>, ThreadBotError> {
    match metadata
        .get(STATE_KEY)
        .and_then(|value| value.get(TRACE_KEY))
    {
        Some(value) => serde_json::from_value(value.clone()).map_err(ThreadBotError::from),
        None => Ok(Vec::new()),
    }
}

fn store_trace(
    metadata: &serde_json::Value,
    trace: &[ChatMessage],
) -> Result<serde_json::Value, ThreadBotError> {
    let mut metadata = metadata.clone();
    if !metadata.is_object() {
        metadata = json!({});
    }
    if !metadata
        .get(STATE_KEY)
        .is_some_and(serde_json::Value::is_object)
    {
        metadata[STATE_KEY] = json!({});
    }

    metadata[STATE_KEY][TRACE_KEY] = serde_json::to_value(trace).map_err(ThreadBotError::from)?;
    Ok(metadata)
}

fn truncate_utf8(value: String, max_bytes: usize) -> String {
    if max_bytes == 0 || value.len() <= max_bytes {
        return value;
    }

    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...[truncated]", &value[..end])
}

fn load_state(metadata: &serde_json::Value) -> Result<SupportThreadState, ThreadBotError> {
    match metadata.get(STATE_KEY) {
        Some(value) => serde_json::from_value(value.clone()).map_err(ThreadBotError::from),
        None => Ok(SupportThreadState::default()),
    }
}

fn store_state(
    metadata: &serde_json::Value,
    state: &SupportThreadState,
) -> Result<serde_json::Value, ThreadBotError> {
    let mut metadata = metadata.clone();
    if !metadata.is_object() {
        metadata = json!({});
    }

    metadata[STATE_KEY] = serde_json::to_value(state).map_err(ThreadBotError::from)?;
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        DebugCommandConfig, EngineerNotificationConfig, EngineerNotificationTarget,
        InstructionConfig, LlmConfig, SupportBotLimits, SupportRouteConfig, ToolConfig,
    };
    use crate::debug::DebugResponse;
    use crate::llm::LlmResponse;
    use crate::testutil::PanicStore;
    use crate::tools::{register_default_workflow_tools, ToolCall, ToolSpec};
    use async_trait::async_trait;
    use chrono::DateTime;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::collections::VecDeque;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;
    use thread_bot::{
        AppendReaction, ChannelCheckpoint, ThreadInfo, ThreadMessage, ThreadMessageRecord,
        ThreadReaction, ThreadRecord, ThreadStatus, ThreadStore, UpsertThread, UpsertThreadMessage,
    };

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
        async fn handle_debug_command(
            &self,
            command: DebugCommand,
        ) -> crate::Result<DebugResponse> {
            Ok(DebugResponse {
                message: format!("debug: {}", command.name),
            })
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
                target: EngineerNotificationTarget::SameThread,
            },
        }
    }

    fn thread(channel_id: &str, message: &str) -> Thread {
        Thread {
            info: ThreadInfo {
                thread_id: "thread-1".to_string(),
                root_post_id: "post-1".to_string(),
                channel_id: channel_id.to_string(),
                creator_user_id: "user-1".to_string(),
                status: ThreadStatus::Active,
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

    fn engineer_thread_with_source(source_thread_id: &str) -> Thread {
        let mut thread = thread("engineers", "engineer root");
        thread.messages[0].props = json!({
            STATE_KEY: {
                "source_thread_id": source_thread_id
            }
        });
        thread
    }

    fn context() -> ThreadContext {
        ThreadContext {
            config: Arc::new(Default::default()),
            store: Arc::new(PanicStore),
            plugin_id: "test",
            bot_user_id: Some("bot".to_string()),
        }
    }

    fn context_with_store(store: Arc<dyn ThreadStore>) -> ThreadContext {
        ThreadContext {
            config: Arc::new(Default::default()),
            store,
            plugin_id: "test",
            bot_user_id: Some("bot".to_string()),
        }
    }

    struct TestStore {
        threads: Mutex<HashMap<String, ThreadRecord>>,
    }

    impl TestStore {
        fn new(records: Vec<ThreadRecord>) -> Self {
            Self {
                threads: Mutex::new(
                    records
                        .into_iter()
                        .map(|record| (record.thread_id.clone(), record))
                        .collect(),
                ),
            }
        }

        fn thread(&self, thread_id: &str) -> Option<ThreadRecord> {
            self.threads.lock().unwrap().get(thread_id).cloned()
        }
    }

    #[async_trait]
    impl ThreadStore for TestStore {
        async fn upsert_thread(
            &self,
            _input: UpsertThread,
        ) -> Result<ThreadRecord, ThreadBotError> {
            panic!("not used in this test")
        }

        async fn get_thread(
            &self,
            thread_id: &str,
        ) -> Result<Option<ThreadRecord>, ThreadBotError> {
            Ok(self.threads.lock().unwrap().get(thread_id).cloned())
        }

        async fn get_thread_by_post(
            &self,
            _post_id: &str,
        ) -> Result<Option<ThreadRecord>, ThreadBotError> {
            panic!("not used in this test")
        }

        async fn list_threads_by_status(
            &self,
            _statuses: &[ThreadStatus],
            _updated_after: Option<DateTime<Utc>>,
            _updated_before: Option<DateTime<Utc>>,
        ) -> Result<Vec<ThreadRecord>, ThreadBotError> {
            panic!("not used in this test")
        }

        async fn update_thread_status(
            &self,
            _thread_id: &str,
            _status: ThreadStatus,
        ) -> Result<(), ThreadBotError> {
            panic!("not used in this test")
        }

        async fn set_thread_metadata(
            &self,
            thread_id: &str,
            metadata: serde_json::Value,
        ) -> Result<(), ThreadBotError> {
            let mut threads = self.threads.lock().unwrap();
            let Some(record) = threads.get_mut(thread_id) else {
                return Err(ThreadBotError::ThreadNotFound(thread_id.to_string()));
            };
            record.metadata = metadata;
            Ok(())
        }

        async fn update_thread_seen(
            &self,
            _thread_id: &str,
            _post_id: &str,
            _seen_at: DateTime<Utc>,
        ) -> Result<(), ThreadBotError> {
            panic!("not used in this test")
        }

        async fn update_thread_processed(
            &self,
            _thread_id: &str,
            _post_id: &str,
            _processed_at: DateTime<Utc>,
        ) -> Result<(), ThreadBotError> {
            panic!("not used in this test")
        }

        async fn upsert_message(
            &self,
            _input: UpsertThreadMessage,
        ) -> Result<ThreadMessageRecord, ThreadBotError> {
            panic!("not used in this test")
        }

        async fn get_message(
            &self,
            _post_id: &str,
        ) -> Result<Option<ThreadMessageRecord>, ThreadBotError> {
            panic!("not used in this test")
        }

        async fn list_thread_messages(
            &self,
            _thread_id: &str,
        ) -> Result<Vec<ThreadMessageRecord>, ThreadBotError> {
            panic!("not used in this test")
        }

        async fn set_message_metadata(
            &self,
            _post_id: &str,
            _metadata: serde_json::Value,
        ) -> Result<(), ThreadBotError> {
            panic!("not used in this test")
        }

        async fn append_reaction(&self, _input: AppendReaction) -> Result<(), ThreadBotError> {
            panic!("not used in this test")
        }

        async fn list_thread_reactions(
            &self,
            _thread_id: &str,
        ) -> Result<Vec<ThreadReaction>, ThreadBotError> {
            panic!("not used in this test")
        }

        async fn list_channel_checkpoints(&self) -> Result<Vec<ChannelCheckpoint>, ThreadBotError> {
            panic!("not used in this test")
        }

        async fn upsert_channel_checkpoint(
            &self,
            _channel_id: &str,
            _last_seen_post_at: DateTime<Utc>,
        ) -> Result<(), ThreadBotError> {
            panic!("not used in this test")
        }

        async fn advance_channel_checkpoint(
            &self,
            _channel_id: &str,
            _last_seen_post_at: DateTime<Utc>,
        ) -> Result<(), ThreadBotError> {
            panic!("not used in this test")
        }

        async fn set_all_channels_not_reconciled(&self) -> Result<(), ThreadBotError> {
            panic!("not used in this test")
        }

        async fn set_channel_reconciled(&self, _channel_id: &str) -> Result<(), ThreadBotError> {
            panic!("not used in this test")
        }
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

        let effects = handler
            .handle(&thread("engineers", "!support state"), &context())
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

        let effects = handler.handle(&thread, &context()).await.unwrap();

        assert!(matches!(effects.as_slice(), [ThreadEffect::Noop]));
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

        let effects = handler
            .handle(&thread("users", "help"), &context())
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
    async fn user_tool_trace_is_stored_on_triggering_message_metadata() {
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

        let effects = handler
            .handle(&thread("users", "help"), &context())
            .await
            .unwrap();
        let metadata = effects
            .iter()
            .find_map(|effect| match effect {
                ThreadEffect::UpdateMessage {
                    post_id, metadata, ..
                } if post_id == "post-1" => metadata.clone(),
                _ => None,
            })
            .expect("tool trace update effect");

        let trace = load_trace(&metadata).unwrap();
        assert_eq!(trace.len(), 2);
        assert_eq!(trace[0].tool_calls[0].name, "send_user_message");
        assert_eq!(trace[1].tool_call_id.as_deref(), Some("call-1"));

        let mut next_thread = thread("users", "help");
        next_thread.messages[0].metadata = metadata;
        let rebuilt = handler
            .build_llm_messages(&next_thread, &SupportThreadState::default(), &context())
            .unwrap();
        assert!(rebuilt
            .iter()
            .any(|message| message.tool_call_id.as_deref() == Some("call-1")));
        assert_eq!(llm.requests().len(), 2);
    }

    #[tokio::test]
    async fn notify_engineer_same_thread_uses_reply_effect() {
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

        let effects = handler
            .handle(&thread("users", "help"), &context())
            .await
            .unwrap();

        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                ThreadEffect::Reply { message, metadata }
                    if message == "need help"
                        && metadata["support_bot"]["kind"] == "engineer_notification"
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
    fn source_user_thread_id_is_read_from_engineer_root_props() {
        assert_eq!(
            source_user_thread_id(&engineer_thread_with_source("user-thread-1")).as_deref(),
            Some("user-thread-1")
        );
    }

    #[test]
    fn debug_chat_message_is_json_block() {
        let message = ChatMessage::tool("call-1", "{\"ok\":true}");
        let formatted = format_debug_chat_message("tool result", &message);

        assert!(formatted.contains("**Debug: tool result**"));
        assert!(formatted.contains("```json"));
        assert!(formatted.contains("call-1"));
    }

    #[tokio::test]
    async fn debug_on_command_updates_source_thread_metadata() {
        let now = Utc::now();
        let user_record = ThreadRecord {
            thread_id: "user-thread-1".to_string(),
            root_post_id: "user-root-1".to_string(),
            channel_id: "users".to_string(),
            creator_user_id: "user-1".to_string(),
            status: ThreadStatus::Active,
            metadata: json!({}),
            last_seen_post_id: None,
            last_seen_post_at: None,
            last_processed_post_id: None,
            last_processed_post_at: None,
            created_at: now,
            updated_at: now,
        };
        let store = Arc::new(TestStore::new(vec![user_record]));
        let handler = SupportBotHandler::new(
            "support",
            test_config(),
            Arc::new(StaticLlm),
            Arc::new(ToolRegistry::new()),
            "system",
        );
        let mut engineer_thread = engineer_thread_with_source("user-thread-1");
        engineer_thread.messages[0].message = "!support debug on".to_string();

        let effects = handler
            .handle(&engineer_thread, &context_with_store(store.clone()))
            .await
            .unwrap();

        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                ThreadEffect::Reply { message, .. } if message.contains("Debug mode is on")
            )
        }));
        let updated = store.thread("user-thread-1").expect("source thread");
        let state = load_state(&updated.metadata).unwrap();
        assert!(state.debug);
    }

    #[tokio::test]
    async fn debug_status_command_keeps_state_value() {
        let now = Utc::now();
        let user_record = ThreadRecord {
            thread_id: "user-thread-2".to_string(),
            root_post_id: "user-root-2".to_string(),
            channel_id: "users".to_string(),
            creator_user_id: "user-1".to_string(),
            status: ThreadStatus::Active,
            metadata: store_state(
                &json!({}),
                &SupportThreadState {
                    debug: true,
                    ..Default::default()
                },
            )
            .unwrap(),
            last_seen_post_id: None,
            last_seen_post_at: None,
            last_processed_post_id: None,
            last_processed_post_at: None,
            created_at: now,
            updated_at: now,
        };
        let store = Arc::new(TestStore::new(vec![user_record]));
        let handler = SupportBotHandler::new(
            "support",
            test_config(),
            Arc::new(StaticLlm),
            Arc::new(ToolRegistry::new()),
            "system",
        );
        let mut engineer_thread = engineer_thread_with_source("user-thread-2");
        engineer_thread.messages[0].message = "!support debug status".to_string();

        handler
            .handle(&engineer_thread, &context_with_store(store.clone()))
            .await
            .unwrap();

        let updated = store.thread("user-thread-2").expect("source thread");
        let state = load_state(&updated.metadata).unwrap();
        assert!(state.debug);
    }
}
