use crate::admission::SupportThreadAdmissionHook;
use crate::config::SupportBotConfig;
use crate::debug::{DebugCommand, DebugCommandHandler};
use crate::debug_export::handle_debug_export_html;
use crate::llm::LlmClient;
use crate::metadata::{
    load_thread_state, metadata_value, store_thread_state, SupportMetadata, SupportMetadataKind,
};
use crate::metrics::SupportBotMetricsHandle;
use crate::notifier::{engineer_thread_root_message, source_post_link, support_post_props};
use crate::state::SupportThreadStatus;
use crate::tools::ToolRegistry;
use async_trait::async_trait;
use chrono::Utc;
use mattermost_api::apis::reactions_api;
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
    admission_hook: Option<Arc<dyn SupportThreadAdmissionHook>>,
    debug_handler: Option<Arc<dyn DebugCommandHandler>>,
    system_prompt: String,
    metrics: SupportBotMetricsHandle,
}

pub(crate) const ENGINEER_LINK_KIND: &str = "support_engineer";
pub(crate) const USER_THREAD_KIND: &str = "support_user";

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
            admission_hook: None,
            debug_handler: None,
            system_prompt: system_prompt.into(),
            metrics: SupportBotMetricsHandle::noop(),
        }
    }

    pub fn with_debug_handler(mut self, handler: Arc<dyn DebugCommandHandler>) -> Self {
        self.debug_handler = Some(handler);
        self
    }

    pub fn with_admission_hook(mut self, hook: Arc<dyn SupportThreadAdmissionHook>) -> Self {
        self.admission_hook = Some(hook);
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
                } else if channel_id == self.config.routes.engineer_channel_id {
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
            metadata: metadata_value(&SupportMetadata::new(SupportMetadataKind::DebugResponse))?,
        }])
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
                metadata: support_post_props(SupportMetadataKind::EngineerThreadRoot, thread),
                thread_metadata: metadata_value(&SupportMetadata::for_source_thread(
                    SupportMetadataKind::EngineerThread,
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

        let mut state = match load_thread_state(metadata) {
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

        match store_thread_state(metadata, &state) {
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

mod user_workflow;

#[cfg(test)]
#[path = "tests/handler/mod.rs"]
mod tests;
