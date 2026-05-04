use crate::config::SupportBotConfig;
use crate::conversation::{
    build_llm_messages, load_state, result_to_message, truncate_utf8, STATE_KEY,
};
use crate::debug::{DebugCommand, DebugCommandHandler};
use crate::debug_export::handle_debug_export_html;
use crate::llm::{LlmClient, LlmRequest};
use crate::metrics::SupportBotMetricsHandle;
use crate::notifier::MattermostSupportNotifier;
use crate::output::sanitize_user_visible_message;
use crate::tools::ToolCall;
use crate::tools::{ToolContext, ToolExecutionOutcome, ToolRegistry, ToolResult};
use crate::user_thread_run::UserThreadRun;
use crate::workflow::apply_action;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use thread_bot::{
    Thread, ThreadBotError, ThreadCloseReason, ThreadContext, ThreadEffect, ThreadHandler,
    ThreadMessage,
};
use tracing::{info, warn};

pub struct SupportBotHandler {
    id: &'static str,
    config: SupportBotConfig,
    llm: Arc<dyn LlmClient>,
    tools: Arc<ToolRegistry>,
    debug_handler: Option<Arc<dyn DebugCommandHandler>>,
    system_prompt: String,
    metrics: SupportBotMetricsHandle,
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
            metrics: SupportBotMetricsHandle::noop(),
        }
    }

    pub fn with_debug_handler(mut self, handler: Arc<dyn DebugCommandHandler>) -> Self {
        self.debug_handler = Some(handler);
        self
    }

    pub fn with_metrics(mut self, metrics: SupportBotMetricsHandle) -> Self {
        self.metrics = metrics;
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
            info!(
                thread_id = %thread.info.thread_id,
                channel_id = %thread.info.channel_id,
                "support-bot: engineer thread has no new external message"
            );
            return Ok(vec![ThreadEffect::Noop]);
        };
        info!(
            thread_id = %thread.info.thread_id,
            channel_id = %thread.info.channel_id,
            post_id = %message.post_id,
            "support-bot: received engineer thread message"
        );

        let Some(command) =
            DebugCommand::parse(&message.message, &self.config.routes.debug_commands)
                .map(|matched| matched.command)
        else {
            info!(
                thread_id = %thread.info.thread_id,
                post_id = %message.post_id,
                message = %truncate_utf8(message.message.clone(), 200),
                "support-bot: engineer message is not a debug command"
            );
            return Ok(vec![ThreadEffect::Noop]);
        };

        if command.name == "debug-report" {
            info!(
                thread_id = %thread.info.thread_id,
                channel_id = %thread.info.channel_id,
                "support-bot: received debug-report command"
            );
            let effects =
                handle_debug_export_html(&self.config.engineer_notifications.target, thread, ctx)
                    .await;
            if effects.is_ok() {
                self.metrics.record_reply("engineer", "success");
            } else {
                self.metrics.record_reply("engineer", "error");
            }
            return effects;
        }

        let response = match &self.debug_handler {
            Some(handler) => handler.handle_debug_command(command).await?,
            None => crate::debug::DebugResponse {
                message: "Debug command handler is not configured.".to_string(),
            },
        };

        self.metrics.record_reply("engineer", "success");
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
        info!(
            thread_id = %thread.info.thread_id,
            channel_id = %thread.info.channel_id,
            post_id = %trigger_message.post_id,
            "support-bot: handling user thread message"
        );

        let mut state = load_state(&thread.info.metadata)?;
        if let Some(engineer_thread) = self
            .engineer_notifier(ctx)
            .ensure_engineer_thread(thread, state.engineer_thread.as_ref())
            .await?
        {
            state.engineer_thread = Some(engineer_thread);
        }
        let messages = build_llm_messages(
            &self.system_prompt,
            thread,
            &state,
            ctx.bot_user_id.as_deref(),
        )?;
        let mut run = UserThreadRun::new(state, messages);

        run.mirror_user_message(
            &self.config.engineer_notifications.target,
            ctx,
            thread,
            trigger_message,
        )
        .await?;

        for _round in 0..self.config.limits.max_tool_rounds {
            let llm_started = Instant::now();
            let response_result = self
                .llm
                .complete(LlmRequest {
                    model: self.config.llm.model.clone(),
                    messages: run.messages().to_vec(),
                    tools: self.tools.specs(),
                })
                .await;
            let response = match response_result {
                Ok(response) => {
                    self.metrics
                        .record_llm_request("success", llm_started.elapsed());
                    response
                }
                Err(error) => {
                    self.metrics
                        .record_llm_request("error", llm_started.elapsed());
                    return Err(error.into());
                }
            };

            if response.tool_calls.is_empty() {
                info!(
                    thread_id = %thread.info.thread_id,
                    post_id = %trigger_message.post_id,
                    trace_len = run.trace_len(),
                    "support-bot: llm completed without further tools"
                );
                if let Some(content) = response
                    .message
                    .content
                    .and_then(sanitize_user_visible_message)
                {
                    run.reply(
                        &self.config.engineer_notifications.target,
                        ctx,
                        thread,
                        content,
                        json!({
                            STATE_KEY: {
                                "kind": "assistant_response"
                            }
                        }),
                    )
                    .await?;
                    self.metrics.record_reply("user", "success");
                }
                return run.into_effects(thread, trigger_message);
            }

            let tool_calls = response
                .tool_calls
                .into_iter()
                .take(self.config.limits.max_tool_calls_per_round)
                .collect::<Vec<_>>();
            info!(
                thread_id = %thread.info.thread_id,
                post_id = %trigger_message.post_id,
                tool_calls = tool_calls.len(),
                "support-bot: executing tool calls from llm response"
            );

            self.process_tool_calls(ctx, thread, &mut run, response.message.content, tool_calls)
                .await?;

            if run.should_stop_after_tools() {
                info!(
                    thread_id = %thread.info.thread_id,
                    post_id = %trigger_message.post_id,
                    trace_len = run.trace_len(),
                    "support-bot: finishing request after finish_request action"
                );
                return run.into_resolved_effects(thread, trigger_message);
            }
        }

        let failure_message = "I ran into an internal issue while processing your request. "
            .to_string()
            + "I've notified the engineering team and stopped this thread for now.";

        if let Ok(Some(engineer_thread)) = self
            .engineer_notifier(ctx)
            .notify_engineer(
                thread,
                run.engineer_thread(),
                tool_loop_limit_engineer_message(
                    thread,
                    trigger_message,
                    self.config.limits.max_tool_rounds,
                    self.config.limits.max_tool_calls_per_round,
                    run.trace_summary(),
                ),
            )
            .await
        {
            run.set_engineer_thread(engineer_thread);
        }

        run.reply(
            &self.config.engineer_notifications.target,
            ctx,
            thread,
            failure_message,
            json!({
                STATE_KEY: {
                    "kind": "tool_loop_limit"
                }
            }),
        )
        .await?;
        self.metrics.record_reply("user", "success");
        run.into_stopped_effects(thread, trigger_message)
    }

    async fn process_tool_calls(
        &self,
        ctx: &ThreadContext,
        thread: &Thread,
        run: &mut UserThreadRun,
        assistant_content: Option<String>,
        tool_calls: Vec<ToolCall>,
    ) -> Result<(), ThreadBotError> {
        run.push_tool_round(assistant_content, tool_calls.clone());
        let tool_context = ToolContext::new(Arc::new(thread.clone()));

        for call in tool_calls {
            let tool_started = Instant::now();
            let tool_outcome = self.tools.call(tool_context.clone(), call.clone()).await;
            let tool_message = match tool_outcome {
                Ok(ToolExecutionOutcome::ToolResult(result)) => {
                    self.metrics.record_tool_call(
                        &call.name,
                        if result.is_error {
                            "tool_error"
                        } else {
                            "success"
                        },
                        tool_started.elapsed(),
                    );
                    result_to_message(result, self.config.limits.max_tool_result_bytes)
                }
                Ok(ToolExecutionOutcome::Action(action)) => {
                    let action_result = apply_action(
                        &self.config.engineer_notifications.target,
                        ctx,
                        thread,
                        run,
                        &call.id,
                        action,
                    )
                    .await;
                    let result = match action_result {
                        Ok(result) => {
                            self.metrics.record_tool_call(
                                &call.name,
                                if result.is_error {
                                    "action_error"
                                } else {
                                    "action"
                                },
                                tool_started.elapsed(),
                            );
                            result
                        }
                        Err(error) => {
                            self.metrics.record_tool_call(
                                &call.name,
                                "error",
                                tool_started.elapsed(),
                            );
                            return Err(error);
                        }
                    };
                    match &result.content {
                        serde_json::Value::Object(map)
                            if map.get("status").and_then(|value| value.as_str())
                                == Some("sent") =>
                        {
                            match call.name.as_str() {
                                "send_user_message" => self.metrics.record_reply("user", "success"),
                                "notify_engineer" => {
                                    self.metrics.record_reply("engineer", "success")
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    result_to_message(result, self.config.limits.max_tool_result_bytes)
                }
                Err(error) => {
                    self.metrics
                        .record_tool_call(&call.name, "error", tool_started.elapsed());
                    result_to_message(
                        ToolResult {
                            call_id: call.id,
                            content: json!({
                                "error": error.to_string()
                            }),
                            is_error: true,
                        },
                        self.config.limits.max_tool_result_bytes,
                    )
                }
            };
            run.push_tool_message(tool_message);
        }

        Ok(())
    }

    fn engineer_notifier(&self, ctx: &ThreadContext) -> MattermostSupportNotifier {
        MattermostSupportNotifier::new(
            ctx.config.clone(),
            self.config.engineer_notifications.target.clone(),
        )
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
        let route = self.route(&thread.info.channel_id);
        let route_label = route.label();
        let started = Instant::now();
        let result = match route {
            SupportRoute::User => {
                info!(
                    thread_id = %thread.info.thread_id,
                    channel_id = %thread.info.channel_id,
                    "support-bot: route=user"
                );
                self.handle_user_thread(thread, ctx).await
            }
            SupportRoute::Engineer => {
                info!(
                    thread_id = %thread.info.thread_id,
                    channel_id = %thread.info.channel_id,
                    "support-bot: route=engineer"
                );
                self.handle_engineer_thread(thread, ctx).await
            }
            SupportRoute::Ignored => {
                info!(
                    thread_id = %thread.info.thread_id,
                    channel_id = %thread.info.channel_id,
                    "support-bot: route=ignored"
                );
                Ok(vec![ThreadEffect::Noop])
            }
        };
        let outcome = if result.is_ok() { "success" } else { "error" };
        self.metrics
            .record_thread_event(route_label, "handle", outcome);
        self.metrics
            .observe_handle_duration(route_label, outcome, started.elapsed());
        result
    }

    async fn on_thread_closed(
        &self,
        thread: &Thread,
        reason: ThreadCloseReason,
        ctx: &ThreadContext,
    ) -> Result<(), ThreadBotError> {
        if !matches!(self.route(&thread.info.channel_id), SupportRoute::User) {
            return Ok(());
        }

        let metadata = match ctx.store.get_thread(&thread.info.thread_id).await? {
            Some(record) => record.metadata,
            None => thread.info.metadata.clone(),
        };

        let current_thread = load_state(&metadata)
            .map(|state| state.engineer_thread)
            .unwrap_or_else(|error| {
                warn!(
                    thread_id = %thread.info.thread_id,
                    error = %error,
                    "support-bot: failed to load state while closing thread"
                );
                None
            });

        let result = self
            .engineer_notifier(ctx)
            .notify_engineer(
                thread,
                current_thread.as_ref(),
                format!(
                    "**Support thread closed**\n\nreason: `{reason:?}`\nsource: {}",
                    thread.info.thread_id
                ),
            )
            .await
            .map(|_| ())
            .map_err(ThreadBotError::from);
        self.metrics.record_thread_close(
            thread_close_reason_label(reason),
            if result.is_ok() { "success" } else { "error" },
        );
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SupportRoute {
    User,
    Engineer,
    Ignored,
}

impl SupportRoute {
    fn label(self) -> &'static str {
        match self {
            SupportRoute::User => "user",
            SupportRoute::Engineer => "engineer",
            SupportRoute::Ignored => "ignored",
        }
    }
}

fn thread_close_reason_label(reason: ThreadCloseReason) -> &'static str {
    match reason {
        ThreadCloseReason::ResolvedByReaction => "resolved_by_reaction",
        ThreadCloseReason::StoppedByReaction => "stopped_by_reaction",
        ThreadCloseReason::ResolvedByHandler => "resolved_by_handler",
        ThreadCloseReason::StoppedByHandler => "stopped_by_handler",
        ThreadCloseReason::ResolvedExternally => "resolved_externally",
        ThreadCloseReason::StoppedExternally => "stopped_externally",
    }
}

fn last_new_external_message<'a>(
    thread: &'a Thread,
    ctx: &ThreadContext,
) -> Option<&'a ThreadMessage> {
    thread.messages.iter().rev().find(|message| {
        message.is_new
            && ctx.bot_user_id.as_deref() != Some(message.user_id.as_str())
            && !message.message.trim().is_empty()
    })
}

fn tool_loop_limit_engineer_message(
    thread: &Thread,
    trigger_message: &ThreadMessage,
    max_tool_rounds: usize,
    max_tool_calls_per_round: usize,
    trace_summary: String,
) -> String {
    format!(
        "**Support bot stopped due to tool loop limit**\n\n\
        - source_thread_id: `{}`\n\
        - source_channel_id: `{}`\n\
        - source_post_id: `{}`\n\
        - max_tool_rounds: `{}`\n\
        - max_tool_calls_per_round: `{}`\n\
        - detail: `{}`\n\n\
        No further bot messages will be posted in this support thread until an engineer restarts handling.\n\
        To export diagnostics HTML, run `!support debug-report` in this engineer thread.",
        thread.info.thread_id,
        thread.info.channel_id,
        trigger_message.post_id,
        max_tool_rounds,
        max_tool_calls_per_round,
        truncate_utf8(trace_summary, 2000)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        DebugCommandConfig, EngineerNotificationConfig, EngineerNotificationTarget,
        InstructionConfig, LlmConfig, SupportBotLimits, SupportRouteConfig, ToolConfig,
    };
    use crate::conversation::{store_state, TRACE_KEY};
    use crate::debug::DebugResponse;
    use crate::debug_export::source_user_thread_id;
    use crate::llm::{ChatMessage, ChatRole, LlmResponse};
    use crate::state::SupportThreadState;
    use crate::testutil::PanicStore;
    use crate::tools::{register_default_workflow_tools, ToolCall, ToolSpec};
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::VecDeque;
    use std::path::PathBuf;
    use std::sync::Arc;
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

        let effects = handler.handle(&thread, &context()).await.unwrap();

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

        let effects = handler
            .handle(&thread("users", ""), &context())
            .await
            .unwrap();

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

        let effects = handler
            .handle(&thread("users", "help"), &context())
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

        let effects = handler
            .handle(&thread("users", "help"), &context())
            .await
            .unwrap();
        assert!(!effects
            .iter()
            .any(|effect| matches!(effect, ThreadEffect::UpdateMessage { .. })));
        assert_eq!(llm.requests().len(), 2);
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

        let effects = handler
            .handle(&thread("users", "help"), &context())
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

        let effects = handler
            .handle(&thread("users", "help"), &context())
            .await
            .unwrap();

        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                ThreadEffect::Reply { message, metadata }
                    if message.contains("internal issue")
                        && metadata["support_bot"]["kind"] == "tool_loop_limit"
            )
        }));
        assert!(effects
            .iter()
            .any(|effect| matches!(effect, ThreadEffect::MarkStopped)));
    }

    #[tokio::test]
    async fn finish_request_persists_finished_state_before_mark_resolved() {
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

        let effects = handler
            .handle(&thread("users", "help"), &context())
            .await
            .unwrap();

        assert_eq!(llm.requests().len(), 1);

        let set_idx = effects
            .iter()
            .position(|effect| matches!(effect, ThreadEffect::SetThreadMetadata { .. }))
            .expect("SetThreadMetadata must be emitted");
        let resolved_idx = effects
            .iter()
            .position(|effect| matches!(effect, ThreadEffect::MarkResolved))
            .expect("MarkResolved must be emitted");
        assert!(set_idx < resolved_idx);

        let state_meta = effects
            .iter()
            .find_map(|effect| match effect {
                ThreadEffect::SetThreadMetadata { metadata } => Some(metadata),
                _ => None,
            })
            .expect("state metadata must exist");
        assert_eq!(state_meta[STATE_KEY]["status"], "finished");
        assert_eq!(
            state_meta[STATE_KEY]["finished_summary"],
            "resolved by cache flush"
        );
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
                        "arguments": { "action": "list" }
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
}
