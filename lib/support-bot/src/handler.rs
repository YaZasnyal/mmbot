use crate::config::SupportBotConfig;
use crate::debug::{DebugCommand, DebugCommandHandler};
use crate::llm::{ChatMessage, ChatRole, LlmClient, LlmRequest};
use crate::notifier::SupportNotifier;
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
    notifier: Option<Arc<dyn SupportNotifier>>,
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
            notifier: None,
            system_prompt: system_prompt.into(),
        }
    }

    pub fn with_debug_handler(mut self, handler: Arc<dyn DebugCommandHandler>) -> Self {
        self.debug_handler = Some(handler);
        self
    }

    pub fn with_notifier(mut self, notifier: Arc<dyn SupportNotifier>) -> Self {
        self.notifier = Some(notifier);
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
        state.waiting_for_user = false;
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
                    state.last_action = Some("assistant_reply".to_string());
                    effects.push(ThreadEffect::Reply {
                        message: content,
                        metadata: json!({
                            STATE_KEY: {
                                "kind": "assistant_response"
                            }
                        }),
                    });
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

            let tool_context = ToolContext::new(Arc::new(thread.clone()));
            for call in tool_calls {
                let tool_message = match self.tools.call(tool_context.clone(), call.clone()).await {
                    Ok(ToolExecutionOutcome::ToolResult(result)) => {
                        result_to_message(result, self.config.limits.max_tool_result_bytes)
                    }
                    Ok(ToolExecutionOutcome::Action(action)) => {
                        let result = self
                            .apply_action(thread, &mut state, &mut effects, &call.id, action)
                            .await;
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
            }
        }

        if !trace.is_empty() {
            effects.push(ThreadEffect::UpdateMessage {
                post_id: trigger_message.post_id.clone(),
                message: None,
                metadata: Some(store_trace(&trigger_message.metadata, &trace)?),
            });
        }

        state.last_action = Some("tool_loop_limit_reached".to_string());
        effects.push(ThreadEffect::SetThreadMetadata {
            metadata: store_state(&thread.info.metadata, &state)?,
        });
        effects.push(ThreadEffect::Reply {
            message:
                "I could not complete the support workflow because the tool loop limit was reached."
                    .to_string(),
            metadata: json!({
                STATE_KEY: {
                    "kind": "tool_loop_limit"
                }
            }),
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
        thread: &Thread,
        state: &mut SupportThreadState,
        effects: &mut Vec<ThreadEffect>,
        call_id: &str,
        action: SupportAction,
    ) -> ToolResult {
        match action {
            SupportAction::SendUserMessage { message } => {
                state.last_action = Some("send_user_message".to_string());
                effects.push(ThreadEffect::Reply {
                    message,
                    metadata: json!({
                        STATE_KEY: {
                            "kind": "tool_action",
                            "tool_call_id": call_id,
                            "action": "send_user_message"
                        }
                    }),
                });
                ToolResult {
                    call_id: call_id.to_string(),
                    content: json!({ "status": "sent" }),
                    is_error: false,
                }
            }
            SupportAction::NotifyEngineer { message } => {
                state.last_action = Some("notify_engineer".to_string());
                state.waiting_for_engineer = true;
                let notify_result = match &self.notifier {
                    Some(notifier) => notifier.notify_engineer(thread, message.clone()).await,
                    None => {
                        return ToolResult {
                            call_id: call_id.to_string(),
                            content: json!({
                                "status": "failed",
                                "error": "engineer notifier is not configured"
                            }),
                            is_error: true,
                        };
                    }
                };

                if let Err(error) = notify_result {
                    return ToolResult {
                        call_id: call_id.to_string(),
                        content: json!({
                            "status": "failed",
                            "error": error.to_string()
                        }),
                        is_error: true,
                    };
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
            SupportAction::WaitForUser { reason } => {
                state.last_action = Some("wait_for_user".to_string());
                state.waiting_for_user = true;
                ToolResult {
                    call_id: call_id.to_string(),
                    content: json!({
                        "status": "waiting",
                        "reason": reason
                    }),
                    is_error: false,
                }
            }
            SupportAction::FinishRequest { summary } => {
                state.last_action = Some("finish_request".to_string());
                state.waiting_for_user = false;
                state.waiting_for_engineer = false;
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
        }
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
    use chrono::Utc;
    use std::collections::VecDeque;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::Duration;
    use thread_bot::{ThreadInfo, ThreadMessage, ThreadStatus};

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

    fn context() -> ThreadContext {
        ThreadContext {
            config: Arc::new(Default::default()),
            store: Arc::new(PanicStore),
            plugin_id: "test",
            bot_user_id: Some("bot".to_string()),
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
}
