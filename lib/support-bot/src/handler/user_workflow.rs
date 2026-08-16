use crate::acp::{AcpPrompt, AcpTurn};
use crate::admission::SupportThreadAdmissionDecision;
use crate::metadata::{
    has_thread_state, load_thread_state, metadata_value, store_thread_state, SupportMetadata,
    SupportMetadataKind,
};
use crate::notifier::{quote_for_mattermost, source_post_link, support_post_props};
use crate::state::{SupportRuntimeState, SupportRuntimeStatus, SupportThreadStatus};
use std::time::{Duration, Instant};
use thread_bot::{
    Thread, ThreadBotError, ThreadContext, ThreadEffect, ThreadMetadataTarget, ThreadRecord,
    ThreadTarget, ThreadTrigger,
};
use tracing::{info, warn, Span};

use super::{external_message_by_id, trigger_post_id, SupportBotHandler, ENGINEER_LINK_KIND};

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
            return Ok(vec![ThreadEffect::Noop]);
        };
        Span::current().record("post_id", tracing::field::display(trigger_post_id));

        let mut state = load_thread_state(&record.metadata)?;
        if state.status != SupportThreadStatus::Active {
            info!(status = ?state.status, "support-bot: inactive support thread skipped");
            return Ok(vec![ThreadEffect::Noop]);
        }
        if let Some(effect) = self.live_control_reaction_effect(record, ctx).await {
            return Ok(vec![effect]);
        }

        let thread = ctx.build_thread_snapshot(&record.thread_id).await?;
        let Some(message) = external_message_by_id(&thread, ctx, trigger_post_id) else {
            return Ok(vec![ThreadEffect::Noop]);
        };

        let first_turn = !has_thread_state(&record.metadata);
        if first_turn {
            if let Some(hook) = &self.admission_hook {
                if let SupportThreadAdmissionDecision::Ignore { reason } =
                    hook.evaluate(&thread).await?
                {
                    state.status = SupportThreadStatus::Ignored;
                    state.ignored_reason = reason;
                    return Ok(vec![persist_state(&thread, &state)?]);
                }
            }
        }

        if let Some(mut effects) = self
            .ensure_engineer_thread_effects(ctx, &thread, &self.config.routes.engineer_channel_id)
            .await?
        {
            if first_turn && self.admission_hook.is_some() {
                effects.insert(0, persist_state(&thread, &state)?);
            }
            return Ok(effects);
        }

        if state
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.last_inbound_post_id == message.post_id)
        {
            info!("support-bot: duplicate ACP inbound post skipped");
            return Ok(vec![ThreadEffect::Noop]);
        }

        let mut effects = mirror_user_message(&thread, message, ctx);
        let started = Instant::now();
        let result = self
            .acp_runtime
            .prompt(AcpPrompt {
                thread_id: thread.info.thread_id.clone(),
                post_id: message.post_id.clone(),
                message: message.message.clone(),
                session_id: state
                    .runtime
                    .as_ref()
                    .map(|runtime| runtime.acp_session_id.clone()),
            })
            .await;
        let elapsed = started.elapsed();
        self.metrics
            .record_acp_request(if result.is_ok() { "success" } else { "error" }, elapsed);

        match result {
            Ok(turn) => {
                state.runtime = Some(SupportRuntimeState {
                    version: 1,
                    acp_session_id: turn.session_id.clone(),
                    last_inbound_post_id: message.post_id.clone(),
                    last_outbound_post_id: None,
                    status: SupportRuntimeStatus::Active,
                });
                effects.push(acp_engineer_report(&thread, &turn, elapsed));
                effects.push(ThreadEffect::Reply {
                    target: ThreadTarget::CurrentThread,
                    message: turn.response.clone(),
                    metadata: metadata_value(&SupportMetadata::new(
                        SupportMetadataKind::AssistantResponse,
                    ))?,
                });
                effects.push(ThreadEffect::Reply {
                    target: engineer_threads(),
                    message: format!(
                        "**Bot message**\n\n{}",
                        quote_for_mattermost(&turn.response)
                    ),
                    metadata: support_post_props(SupportMetadataKind::BotMessage, &thread),
                });
                self.metrics.record_reply("user", "success");
            }
            Err(error) => {
                warn!(error = %error, "support-bot: ACP prompt failed");
                if let Some(runtime) = &mut state.runtime {
                    runtime.status = SupportRuntimeStatus::Failed;
                    runtime.last_inbound_post_id = message.post_id.clone();
                }
                effects.push(acp_failure_report(&thread, &error.to_string(), elapsed));
                effects.push(ThreadEffect::Reply {
                    target: ThreadTarget::CurrentThread,
                    message: "The support runtime is temporarily unavailable. The engineering team has been notified."
                        .to_string(),
                    metadata: metadata_value(&SupportMetadata::new(
                        SupportMetadataKind::AssistantResponse,
                    ))?,
                });
                self.metrics.record_reply("user", "error");
            }
        }
        effects.push(persist_state(&thread, &state)?);
        Ok(effects)
    }
}

fn mirror_user_message(
    thread: &Thread,
    message: &thread_bot::ThreadMessage,
    ctx: &ThreadContext,
) -> Vec<ThreadEffect> {
    if message.post_id == thread.info.root_post_id {
        return Vec::new();
    }
    vec![ThreadEffect::Reply {
        target: engineer_threads(),
        message: format!(
            "**User message** ([source]({}))\n\n{}",
            source_post_link(&ctx.config, &message.post_id),
            quote_for_mattermost(&message.message)
        ),
        metadata: support_post_props(SupportMetadataKind::UserMessage, thread),
    }]
}

fn persist_state(
    thread: &Thread,
    state: &crate::state::SupportThreadState,
) -> Result<ThreadEffect, ThreadBotError> {
    Ok(ThreadEffect::SetThreadMetadata {
        target: ThreadMetadataTarget::CurrentThread,
        metadata: store_thread_state(&thread.info.metadata, state)?,
    })
}

fn engineer_threads() -> ThreadTarget {
    ThreadTarget::LinkedThreads {
        link_kind: ENGINEER_LINK_KIND.to_string(),
    }
}

fn acp_engineer_report(thread: &Thread, turn: &AcpTurn, elapsed: Duration) -> ThreadEffect {
    ThreadEffect::Reply {
        target: engineer_threads(),
        message: format!(
            "**Qwen ACP report**\n\n- source_thread_id: `{}`\n- acp_session_id: `{}`\n- duration_ms: `{}`\n- stop_reason: `{}`\n- session_recovered: `{}`\n\n**Final response**\n\n{}",
            thread.info.thread_id,
            turn.session_id,
            elapsed.as_millis(),
            turn.stop_reason,
            turn.session_recovered,
            quote_for_mattermost(&turn.response)
        ),
        metadata: support_post_props(SupportMetadataKind::AcpReport, thread),
    }
}

fn acp_failure_report(thread: &Thread, error: &str, elapsed: Duration) -> ThreadEffect {
    ThreadEffect::Reply {
        target: engineer_threads(),
        message: format!(
            "**Qwen ACP failure**\n\n- source_thread_id: `{}`\n- duration_ms: `{}`\n- error: `{}`",
            thread.info.thread_id,
            elapsed.as_millis(),
            error.chars().take(2000).collect::<String>()
        ),
        metadata: support_post_props(SupportMetadataKind::AcpReport, thread),
    }
}
