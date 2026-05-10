use crate::config::{EngineerNotificationTarget, SupportBotConfig};
use crate::conversation::{
    build_llm_messages, load_state, result_to_message, store_state, truncate_utf8, STATE_KEY,
};
use crate::debug::{DebugCommand, DebugCommandHandler};
use crate::debug_export::handle_debug_export_html;
use crate::llm::{LlmClient, LlmRequest};
use crate::metrics::SupportBotMetricsHandle;
use crate::notifier::{engineer_thread_root_message, source_post_link, support_post_props};
use crate::output::sanitize_user_visible_message;
use crate::state::SupportThreadStatus;
use crate::tools::ToolCall;
use crate::tools::{ToolContext, ToolExecutionOutcome, ToolRegistry, ToolResult};
use crate::user_thread_run::StopAfterTools;
use crate::user_thread_run::UserThreadRun;
use crate::workflow::apply_action;
use async_trait::async_trait;
use chrono::Utc;
use mattermost_api::apis::reactions_api;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use thread_bot::{
    ReactionAction, ReactionChange, Thread, ThreadBotError, ThreadContext, ThreadEffect,
    ThreadHandler, ThreadInvocation, ThreadMessage, ThreadRecord, ThreadTarget, ThreadTrigger,
};
use tracing::{debug, info, warn, Instrument, Span};

pub struct SupportBotHandler {
    id: &'static str,
    config: SupportBotConfig,
    llm: Arc<dyn LlmClient>,
    tools: Arc<ToolRegistry>,
    debug_handler: Option<Arc<dyn DebugCommandHandler>>,
    system_prompt: String,
    metrics: SupportBotMetricsHandle,
}

pub(crate) const ENGINEER_LINK_KIND: &str = "support_engineer";

struct ToolCallBatch {
    assistant_content: Option<String>,
    tool_calls: Vec<ToolCall>,
    overflow_tool_calls: Vec<ToolCall>,
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

    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(thread_id = %record.thread_id, post_id = tracing::field::Empty)
    )]
    async fn handle_engineer_thread(
        &self,
        record: &ThreadRecord,
        trigger: &ThreadTrigger,
        ctx: &ThreadContext,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        let Some(trigger_post_id) = trigger_post_id(trigger) else {
            info!("support-bot: engineer thread was invoked without a new message");
            return Ok(vec![ThreadEffect::Noop]);
        };
        let thread = ctx.build_thread_snapshot(&record.thread_id).await?;
        let Some(message) = external_message_by_id(&thread, ctx, trigger_post_id) else {
            info!("support-bot: engineer thread has no new external message");
            return Ok(vec![ThreadEffect::Noop]);
        };
        Span::current().record("post_id", tracing::field::display(&message.post_id));
        info!("support-bot: received engineer thread message");

        let Some(command) =
            DebugCommand::parse(&message.message, &self.config.routes.debug_commands)
                .map(|matched| matched.command)
        else {
            info!("support-bot: engineer message is not a debug command");
            return Ok(vec![ThreadEffect::Noop]);
        };

        if command.name == "debug-report" {
            info!("support-bot: received debug-report command");
            let effects =
                handle_debug_export_html(&self.config.engineer_notifications.target, &thread, ctx)
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
            target: ThreadTarget::CurrentThread,
            message: response.message,
            metadata: json!({
                STATE_KEY: {
                    "kind": "debug_response"
                }
            }),
        }])
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(thread_id = %record.thread_id, post_id = tracing::field::Empty)
    )]
    async fn handle_user_thread(
        &self,
        record: &ThreadRecord,
        trigger: &ThreadTrigger,
        ctx: &ThreadContext,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        let Some(trigger_post_id) = trigger_post_id(trigger) else {
            info!("support-bot: user thread was invoked without a new message");
            return Ok(vec![ThreadEffect::Noop]);
        };
        Span::current().record("post_id", tracing::field::display(trigger_post_id));

        let state = load_state(&record.metadata)?;
        if !matches!(state.status, SupportThreadStatus::Active) {
            info!(
                status = ?state.status,
                finished_summary = state.finished_summary.as_deref(),
                "support-bot: user thread is not active in metadata; skipping workflow"
            );
            return Ok(vec![ThreadEffect::Noop]);
        }

        if let Some(effect) = self.live_control_reaction_effect(record, ctx).await {
            info!("support-bot: live control reaction found; persisting metadata status");
            return Ok(vec![effect]);
        }

        let thread = ctx.build_thread_snapshot(&record.thread_id).await?;
        let Some(trigger_message) = external_message_by_id(&thread, ctx, trigger_post_id) else {
            info!("support-bot: user thread trigger message is not processable");
            return Ok(vec![ThreadEffect::Noop]);
        };
        info!("support-bot: handling user thread message");

        match &self.config.engineer_notifications.target {
            EngineerNotificationTarget::SameThread => {}
            EngineerNotificationTarget::MattermostChannel { channel_id } => {
                if let Some(effects) = self
                    .ensure_engineer_thread_effects(ctx, &thread, channel_id)
                    .await?
                {
                    info!("support-bot: queuing linked engineer thread creation");
                    return Ok(effects);
                }
            }
        }

        let messages = build_llm_messages(
            &self.system_prompt,
            &thread,
            &state,
            ctx.bot_user_id.as_deref(),
        )?;
        let mut run = UserThreadRun::new(state, messages);

        run.mirror_user_message(
            &self.config.engineer_notifications.target,
            ctx,
            &thread,
            trigger_message,
        )
        .await?;

        for round in 0..self.config.limits.max_tool_rounds {
            let round_started = Instant::now();
            info!(round, "support-bot: llm round started");
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
            let requested_tool_calls = response.tool_calls.len();
            info!(
                round,
                requested_tool_calls, "support-bot: llm response received"
            );

            if response.tool_calls.is_empty() {
                if response
                    .message
                    .content
                    .as_ref()
                    .map(|content| content.trim().is_empty())
                    .unwrap_or(true)
                {
                    warn!(
                        round,
                        "support-bot: empty assistant response with no tool calls"
                    );
                }
                info!(
                    round,
                    elapsed_ms = round_started.elapsed().as_millis() as u64,
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
                        &thread,
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
                return run.into_effects(&thread, trigger_message);
            }

            let mut tool_calls = response.tool_calls;
            let overflow_tool_calls = tool_calls.split_off(
                self.config
                    .limits
                    .max_tool_calls_per_round
                    .min(tool_calls.len()),
            );
            let truncated_tool_calls = overflow_tool_calls.len();
            if truncated_tool_calls > 0 {
                warn!(
                    round,
                    requested_tool_calls,
                    truncated_tool_calls,
                    "support-bot: tool calls truncated by per-round limit"
                );
            }
            info!(
                round,
                requested_tool_calls,
                truncated_tool_calls,
                "support-bot: executing tool calls from llm response"
            );

            self.process_tool_calls(
                ctx,
                &thread,
                &mut run,
                ToolCallBatch {
                    assistant_content: response.message.content,
                    tool_calls,
                    overflow_tool_calls,
                },
                &trigger_message.post_id,
            )
            .await?;
            info!(
                round,
                elapsed_ms = round_started.elapsed().as_millis() as u64,
                "support-bot: llm round finished"
            );

            match run.stop_after_tools() {
                StopAfterTools::Continue => {}
                StopAfterTools::AwaitNextUser => {
                    info!("support-bot: stopping tool loop after user-facing workflow action");
                    return run.into_effects(&thread, trigger_message);
                }
                StopAfterTools::FinishRequest => {
                    info!("support-bot: finishing request after finish_request action");
                    return run.into_effects(&thread, trigger_message);
                }
            }
        }

        let failure_message = "I ran into an internal issue while processing your request. "
            .to_string()
            + "I've notified the engineering team and stopped this thread for now.";

        if matches!(
            self.config.engineer_notifications.target,
            EngineerNotificationTarget::MattermostChannel { .. }
        ) {
            run.push_effect(ThreadEffect::Reply {
                target: ThreadTarget::LinkedThreads {
                    link_kind: ENGINEER_LINK_KIND.to_string(),
                },
                message: tool_loop_limit_engineer_message(
                    &thread,
                    trigger_message,
                    self.config.limits.max_tool_rounds,
                    self.config.limits.max_tool_calls_per_round,
                    run.trace_summary(),
                ),
                metadata: support_post_props("engineer_notification", &thread),
            });
        }

        run.reply(
            &self.config.engineer_notifications.target,
            ctx,
            &thread,
            failure_message,
            json!({
                STATE_KEY: {
                    "kind": "tool_loop_limit"
                }
            }),
        )
        .await?;
        self.metrics.record_reply("user", "success");
        run.stop_request(Some("tool loop limit reached".to_string()));
        run.into_effects(&thread, trigger_message)
    }

    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(thread_id = %thread.info.thread_id, post_id = tracing::field::Empty)
    )]
    async fn process_tool_calls(
        &self,
        ctx: &ThreadContext,
        thread: &Thread,
        run: &mut UserThreadRun,
        batch: ToolCallBatch,
        trigger_post_id: &str,
    ) -> Result<(), ThreadBotError> {
        Span::current().record("post_id", tracing::field::display(trigger_post_id));
        debug!("support-bot: processing tool call batch");
        let ToolCallBatch {
            assistant_content,
            tool_calls,
            overflow_tool_calls,
        } = batch;
        let executed_tool_calls = tool_calls.len();
        let truncated_tool_calls = overflow_tool_calls.len();
        let all_tool_calls = tool_calls
            .iter()
            .cloned()
            .chain(overflow_tool_calls.iter().cloned())
            .collect::<Vec<_>>();
        run.push_tool_round(assistant_content, all_tool_calls);
        for call in overflow_tool_calls {
            run.push_tool_message(result_to_message(
                ToolResult {
                    call_id: call.id,
                    content: json!({
                        "error": "tool call skipped because max_tool_calls_per_round was exceeded",
                        "max_tool_calls_per_round": self.config.limits.max_tool_calls_per_round,
                        "executed_tool_calls": executed_tool_calls,
                        "truncated_tool_calls": truncated_tool_calls
                    }),
                    is_error: true,
                },
                self.config.limits.max_tool_result_bytes,
            ));
        }
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
                    info!("support-bot: applying workflow action from tool call");
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
                    info!("support-bot: tool call completed with workflow action");
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

    async fn ensure_engineer_thread_effects(
        &self,
        ctx: &ThreadContext,
        thread: &Thread,
        channel_id: &str,
    ) -> Result<Option<Vec<ThreadEffect>>, ThreadBotError> {
        let has_engineer_link = ctx
            .store
            .list_thread_links(&thread.info.thread_id)
            .await?
            .into_iter()
            .any(|link| link.link_kind == ENGINEER_LINK_KIND);

        if has_engineer_link {
            return Ok(None);
        }

        Ok(Some(vec![
            ThreadEffect::EnsureLinkedThread {
                link_kind: ENGINEER_LINK_KIND.to_string(),
                channel_id: channel_id.to_string(),
                thread_kind: Some("support_engineer".to_string()),
                message: engineer_thread_root_message(
                    thread,
                    source_post_link(&ctx.config, &thread.info.root_post_id),
                ),
                metadata: support_post_props("engineer_thread_root", thread),
                thread_metadata: json!({
                    STATE_KEY: {
                        "kind": "engineer_thread",
                        "source_thread_id": thread.info.thread_id,
                        "source_root_post_id": thread.info.root_post_id,
                        "source_channel_id": thread.info.channel_id
                    }
                }),
                link_metadata: json!({
                    STATE_KEY: {
                        "kind": "engineer_thread_link"
                    }
                }),
            },
            ThreadEffect::Reschedule,
        ]))
    }

    async fn live_control_reaction_effect(
        &self,
        record: &ThreadRecord,
        ctx: &ThreadContext,
    ) -> Option<ThreadEffect> {
        let reactions = match reactions_api::get_reactions(&ctx.config, &record.root_post_id).await
        {
            Ok(reactions) => reactions,
            Err(error) => {
                debug!(
                    thread_id = %record.thread_id,
                    root_post_id = %record.root_post_id,
                    error = %error,
                    "support-bot: failed to fetch live control reactions"
                );
                return None;
            }
        };

        for reaction in reactions {
            let (Some(user_id), Some(emoji_name)) = (reaction.user_id, reaction.emoji_name) else {
                continue;
            };
            let change = ReactionChange {
                post_id: record.root_post_id.clone(),
                user_id,
                emoji_name,
                action: ReactionAction::Added,
                created_at: reaction
                    .create_at
                    .and_then(chrono::DateTime::from_timestamp_millis)
                    .unwrap_or_else(Utc::now),
            };
            if let Some(effect) = self.support_control_reaction_effect(
                &record.channel_id,
                &record.thread_id,
                &record.metadata,
                &change,
            ) {
                return Some(effect);
            }
        }

        None
    }

    fn support_control_reaction_effect(
        &self,
        channel_id: &str,
        thread_id: &str,
        metadata: &serde_json::Value,
        change: &ReactionChange,
    ) -> Option<ThreadEffect> {
        if !matches!(self.route(channel_id), SupportRoute::User) {
            return None;
        }

        let status = match (&change.action, change.emoji_name.as_str()) {
            (ReactionAction::Added, "white_check_mark") => SupportThreadStatus::Finished,
            (ReactionAction::Added, "stop_sign") => SupportThreadStatus::Stopped,
            _ => return None,
        };

        let mut state = match load_state(metadata) {
            Ok(state) => state,
            Err(error) => {
                warn!(
                    thread_id = %thread_id,
                    post_id = %change.post_id,
                    error = %error,
                    "support-bot: failed to load state for control reaction"
                );
                return None;
            }
        };
        if matches!(status, SupportThreadStatus::Finished) && state.finished_summary.is_none() {
            state.finished_summary = Some("resolved by control reaction".to_string());
        }
        state.status = status;

        match store_state(metadata, &state) {
            Ok(metadata) => Some(ThreadEffect::SetThreadMetadata { metadata }),
            Err(error) => {
                warn!(
                    thread_id = %thread_id,
                    post_id = %change.post_id,
                    error = %error,
                    "support-bot: failed to store state for control reaction"
                );
                None
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
        Ok(matches!(
            self.route(&thread.info.channel_id),
            SupportRoute::User
        ))
    }

    fn thread_kind(&self, thread: &Thread) -> Option<String> {
        match self.route(&thread.info.channel_id) {
            SupportRoute::User => Some("support_user".to_string()),
            SupportRoute::Engineer => Some("support_engineer".to_string()),
            SupportRoute::Ignored => None,
        }
    }

    async fn handle(
        &self,
        invocation: &ThreadInvocation,
        ctx: &ThreadContext,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        let route = self.route(&invocation.thread.channel_id);
        let route_label = route.label();
        let started = Instant::now();
        let result = match route {
            SupportRoute::User => {
                self.handle_user_thread(&invocation.thread, &invocation.trigger, ctx)
                    .instrument(tracing::info_span!("handle", route = "user"))
                    .await
            }
            SupportRoute::Engineer => {
                self.handle_engineer_thread(&invocation.thread, &invocation.trigger, ctx)
                    .instrument(tracing::info_span!("handle", route = "engineer"))
                    .await
            }
            SupportRoute::Ignored => Ok(vec![ThreadEffect::Noop]),
        };
        let outcome = if result.is_ok() { "success" } else { "error" };
        self.metrics
            .record_thread_event(route_label, "handle", outcome);
        self.metrics
            .observe_handle_duration(route_label, outcome, started.elapsed());
        result
    }

    fn on_control_reaction(
        &self,
        thread_record: &ThreadRecord,
        change: &ReactionChange,
    ) -> Option<ThreadEffect> {
        self.support_control_reaction_effect(
            &thread_record.channel_id,
            &thread_record.thread_id,
            &thread_record.metadata,
            change,
        )
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

fn trigger_post_id(trigger: &ThreadTrigger) -> Option<&str> {
    match trigger {
        ThreadTrigger::NewMessage { post_id } => Some(post_id),
        ThreadTrigger::Reschedule | ThreadTrigger::Wake => None,
    }
}

fn external_message_by_id<'a>(
    thread: &'a Thread,
    ctx: &ThreadContext,
    post_id: &str,
) -> Option<&'a ThreadMessage> {
    thread.messages.iter().find(|message| {
        message.post_id == post_id
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
        ThreadMessageRecord, ThreadReaction, ThreadRecord, ThreadStore, ThreadTrigger,
        UpsertThread, UpsertThreadLink, UpsertThreadMessage,
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
        async fn handle_debug_command(
            &self,
            command: DebugCommand,
        ) -> crate::Result<DebugResponse> {
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

        async fn call(
            &self,
            _ctx: ToolContext,
            call: ToolCall,
        ) -> crate::Result<ToolExecutionOutcome> {
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
                thread_kind: Some("support_user".to_string()),
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
            thread_kind: Some("support_user".to_string()),
            metadata,
            last_seen_post_id: None,
            last_seen_post_at: None,
            last_processed_post_id: None,
            last_processed_post_at: None,
            created_at: now,
            updated_at: now,
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

    struct SnapshotStore {
        record: ThreadRecord,
        messages: Vec<ThreadMessageRecord>,
        links: Vec<ThreadLink>,
    }

    #[async_trait]
    impl ThreadStore for SnapshotStore {
        async fn upsert_thread(
            &self,
            _input: UpsertThread,
        ) -> Result<ThreadRecord, ThreadBotError> {
            panic!("store should not upsert threads")
        }

        async fn get_thread(
            &self,
            thread_id: &str,
        ) -> Result<Option<ThreadRecord>, ThreadBotError> {
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
            _target_thread_id: &str,
        ) -> Result<Vec<ThreadLink>, ThreadBotError> {
            panic!("store should not list reverse thread links")
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
        context_with_snapshot_and_links(thread, Vec::new()).await
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
    async fn should_track_only_user_threads() {
        let handler = SupportBotHandler::new(
            "support",
            test_config(),
            Arc::new(StaticLlm),
            Arc::new(ToolRegistry::new()),
            "system",
        );

        assert!(handler
            .should_track(&thread("users", "help"), &context())
            .await
            .unwrap());
        assert!(!handler
            .should_track(&thread("engineers", "!support state"), &context())
            .await
            .unwrap());
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
                ThreadEffect::SetThreadMetadata { metadata } => Some(metadata),
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
        let ThreadEffect::SetThreadMetadata { metadata } = resolved else {
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
        let ThreadEffect::SetThreadMetadata { metadata } = stopped else {
            panic!("expected SetThreadMetadata");
        };
        assert_eq!(metadata[STATE_KEY]["status"], "stopped");
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
        config.engineer_notifications.target = EngineerNotificationTarget::MattermostChannel {
            channel_id: "engineers".to_string(),
        };
        let handler = SupportBotHandler::new("support", config, llm, Arc::new(registry), "system");

        let mut thread = thread("users", "help");
        thread.info.metadata = store_state(&json!({}), &SupportThreadState::default()).unwrap();
        let (ctx, _server) = context_with_snapshot_and_links(
            &thread,
            vec![ThreadLink {
                source_thread_id: thread.info.thread_id.clone(),
                link_kind: ENGINEER_LINK_KIND.to_string(),
                target_thread_id: "engineer-thread".to_string(),
                metadata: json!({}),
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
                ThreadEffect::SetThreadMetadata { metadata } => Some(metadata),
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
}
