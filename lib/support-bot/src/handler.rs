use crate::config::SupportBotConfig;
use crate::conversation::{
    build_llm_messages, load_state, result_to_message, store_state, truncate_utf8,
};
use crate::debug::{DebugCommand, DebugCommandHandler};
use crate::debug_export::handle_debug_export_html;
use crate::llm::{LlmClient, LlmRequest};
use crate::metadata::{metadata_value, SupportPostKind, SupportPostMetadata};
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
    ThreadHandler, ThreadInvocation, ThreadMessage, ThreadMetadataTarget, ThreadRecord,
    ThreadTarget, ThreadTrigger,
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
pub(crate) const USER_THREAD_KIND: &str = "support_user";

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

    fn route(&self, thread_kind: Option<&str>, channel_id: &str) -> SupportRoute {
        match thread_kind {
            Some(ENGINEER_LINK_KIND) => SupportRoute::Engineer,
            Some(USER_THREAD_KIND) => SupportRoute::User,
            _ => {
                if self
                    .config
                    .routes
                    .user_channel_ids
                    .iter()
                    .any(|user_channel_id| user_channel_id == channel_id)
                {
                    SupportRoute::User
                } else if channel_id == self.config.engineer_notifications.channel_id {
                    SupportRoute::Engineer
                } else {
                    SupportRoute::Ignored
                }
            }
        }
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
            let effects = handle_debug_export_html(&thread, ctx).await;
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
            metadata: metadata_value(&SupportPostMetadata::new(SupportPostKind::DebugResponse))?,
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

        if let Some(effects) = self
            .ensure_engineer_thread_effects(
                ctx,
                &thread,
                &self.config.engineer_notifications.channel_id,
            )
            .await?
        {
            info!("support-bot: queuing linked engineer thread creation");
            return Ok(effects);
        }

        let messages = build_llm_messages(
            &self.system_prompt,
            &thread,
            &state,
            ctx.bot_user_id.as_deref(),
        )?;
        let mut run = UserThreadRun::new(state, messages);

        run.mirror_user_message(
            &thread,
            trigger_message,
            source_post_link(&ctx.config, &trigger_message.post_id),
        );

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
                        &thread,
                        content,
                        metadata_value(&SupportPostMetadata::new(
                            SupportPostKind::AssistantResponse,
                        ))?,
                    );
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
            metadata: support_post_props(SupportPostKind::EngineerNotification, &thread),
        });

        run.reply(
            &thread,
            failure_message,
            metadata_value(&SupportPostMetadata::new(SupportPostKind::ToolLoopLimit))?,
        );
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
                    let action_result = apply_action(thread, run, &call.id, action);
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
                thread_kind: Some(ENGINEER_LINK_KIND.to_string()),
                message: engineer_thread_root_message(
                    thread,
                    source_post_link(&ctx.config, &thread.info.root_post_id),
                ),
                metadata: support_post_props(SupportPostKind::EngineerThreadRoot, thread),
                thread_metadata: metadata_value(&SupportPostMetadata::for_source_thread(
                    SupportPostKind::EngineerThread,
                    thread,
                ))?,
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
                record.thread_kind.as_deref(),
                &record.thread_id,
                &record.channel_id,
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
        thread_kind: Option<&str>,
        thread_id: &str,
        channel_id: &str,
        metadata: &serde_json::Value,
        change: &ReactionChange,
    ) -> Option<ThreadEffect> {
        if !matches!(self.route(thread_kind, channel_id), SupportRoute::User) {
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
            Ok(metadata) => Some(ThreadEffect::SetThreadMetadata {
                target: ThreadMetadataTarget::CurrentThread,
                metadata,
            }),
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

    async fn handle(
        &self,
        invocation: &ThreadInvocation,
        ctx: &ThreadContext,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        let route = self.route(
            invocation.thread.thread_kind.as_deref(),
            invocation.thread.channel_id.as_str(),
        );
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
            SupportRoute::Ignored => {
                tracing::warn!("Unknown thread type. Ignoring it");
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

    fn on_control_reaction(
        &self,
        thread_record: &ThreadRecord,
        change: &ReactionChange,
    ) -> Option<ThreadEffect> {
        self.support_control_reaction_effect(
            thread_record.thread_kind.as_deref(),
            &thread_record.thread_id,
            &thread_record.channel_id,
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
#[path = "tests/handler.rs"]
mod tests;
