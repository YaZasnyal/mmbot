use crate::admission::SupportThreadAdmissionDecision;
use crate::conversation::{build_llm_messages, result_to_message, truncate_utf8};
use crate::metadata::{
    has_thread_state, load_thread_state, metadata_value, store_thread_state, SupportMetadata,
    SupportMetadataKind,
};
use crate::notifier::{source_post_link, support_post_props};
use crate::output::sanitize_user_visible_message;
use crate::state::SupportThreadStatus;
use crate::tools::{ToolCall, ToolContext, ToolExecutionOutcome, ToolResult};
use crate::user_thread_run::{StopAfterTools, UserThreadRun};
use crate::workflow::apply_action;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use thread_bot::{
    Thread, ThreadBotError, ThreadContext, ThreadEffect, ThreadMessage, ThreadRecord, ThreadTarget,
    ThreadTrigger,
};
use tracing::{debug, info, warn, Span};

use super::{external_message_by_id, trigger_post_id, SupportBotHandler, ENGINEER_LINK_KIND};

struct ToolCallBatch {
    assistant_content: Option<String>,
    tool_calls: Vec<ToolCall>,
    overflow_tool_calls: Vec<ToolCall>,
}

impl SupportBotHandler {
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(thread_id = %record.thread_id, post_id = tracing::field::Empty)
    )]
    pub(crate) async fn handle_user_thread(
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

        let mut state = load_thread_state(&record.metadata)?;
        if !matches!(state.status, SupportThreadStatus::Active) {
            info!(
                status = ?state.status,
                finished_summary = state.finished_summary.as_deref(),
                ignored_reason = state.ignored_reason.as_deref(),
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

        let should_persist_admission_accept =
            self.admission_hook.is_some() && !has_thread_state(&record.metadata);
        if let Some(hook) = &self.admission_hook {
            if !has_thread_state(&record.metadata) {
                match hook.evaluate(&thread).await? {
                    SupportThreadAdmissionDecision::Accept => {
                        info!("support-bot: user thread accepted by admission hook");
                    }
                    SupportThreadAdmissionDecision::Ignore { reason } => {
                        info!(
                            ignored_reason = reason.as_deref(),
                            "support-bot: user thread ignored by admission hook"
                        );
                        state.status = SupportThreadStatus::Ignored;
                        state.ignored_reason = reason;
                        let metadata = store_thread_state(&thread.info.metadata, &state)?;
                        return Ok(vec![ThreadEffect::SetThreadMetadata {
                            target: thread_bot::ThreadMetadataTarget::CurrentThread,
                            metadata,
                        }]);
                    }
                }
            }
        }
        info!("support-bot: handling user thread message");

        if let Some(effects) = self
            .ensure_engineer_thread_effects(ctx, &thread, &self.config.routes.engineer_channel_id)
            .await?
        {
            info!("support-bot: queuing linked engineer thread creation");
            if should_persist_admission_accept {
                let metadata = store_thread_state(&thread.info.metadata, &state)?;
                let mut effects = effects;
                effects.insert(
                    0,
                    ThreadEffect::SetThreadMetadata {
                        target: thread_bot::ThreadMetadataTarget::CurrentThread,
                        metadata,
                    },
                );
                return Ok(effects);
            }
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
                .complete(crate::llm::LlmRequest {
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
                        metadata_value(&SupportMetadata::new(
                            SupportMetadataKind::AssistantResponse,
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
            metadata: support_post_props(SupportMetadataKind::EngineerNotification, &thread),
        });

        run.reply(
            &thread,
            failure_message,
            metadata_value(&SupportMetadata::new(SupportMetadataKind::ToolLoopLimit))?,
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
