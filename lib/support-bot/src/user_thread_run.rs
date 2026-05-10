use crate::config::EngineerNotificationTarget;
use crate::conversation::{store_state, with_trace_metadata};
use crate::llm::{ChatMessage, ChatRole};
use crate::notifier::{quote_for_mattermost, support_post_props};
use crate::state::{SupportThreadState, SupportThreadStatus};
use crate::tools::ToolCall;
use thread_bot::{
    Thread, ThreadBotError, ThreadContext, ThreadEffect, ThreadMessage, ThreadTarget,
};
use tracing::{info, warn};

pub(crate) struct UserThreadRun {
    state: SupportThreadState,
    messages: Vec<ChatMessage>,
    trace: Vec<ChatMessage>,
    effects: Vec<ThreadEffect>,
    stop_after_tools: StopAfterTools,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StopAfterTools {
    Continue,
    AwaitNextUser,
    FinishRequest,
}

impl UserThreadRun {
    pub(crate) fn new(state: SupportThreadState, messages: Vec<ChatMessage>) -> Self {
        Self {
            state,
            messages,
            trace: Vec::new(),
            effects: Vec::new(),
            stop_after_tools: StopAfterTools::Continue,
        }
    }

    pub(crate) fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub(crate) fn trace_summary(&self) -> String {
        if self.trace.is_empty() {
            return "(no trace captured)".to_string();
        }

        format!(
            "trace_messages={}, last_trace={}",
            self.trace.len(),
            serde_json::to_string(self.trace.last().unwrap_or(&ChatMessage::assistant("none")))
                .unwrap_or_else(|_| "{}".to_string())
        )
    }

    pub(crate) fn push_tool_round(&mut self, content: Option<String>, tool_calls: Vec<ToolCall>) {
        let assistant_tool_call_message = ChatMessage {
            role: ChatRole::Assistant,
            content,
            name: None,
            tool_call_id: None,
            tool_calls,
        };
        self.messages.push(assistant_tool_call_message.clone());
        self.trace.push(assistant_tool_call_message);
    }

    pub(crate) fn push_tool_message(&mut self, message: ChatMessage) {
        self.messages.push(message.clone());
        self.trace.push(message);
    }

    pub(crate) async fn reply(
        &mut self,
        target: &EngineerNotificationTarget,
        _ctx: &ThreadContext,
        thread: &Thread,
        message: String,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        self.effects.push(ThreadEffect::Reply {
            target: ThreadTarget::CurrentThread,
            message: message.clone(),
            metadata,
        });
        if matches!(target, EngineerNotificationTarget::MattermostChannel { .. }) {
            self.effects.push(ThreadEffect::Reply {
                target: ThreadTarget::LinkedThreads {
                    link_kind: crate::handler::ENGINEER_LINK_KIND.to_string(),
                },
                message: format!("**Bot message**\n\n{}", quote_for_mattermost(&message)),
                metadata: support_post_props("bot_message", thread),
            });
        }
        Ok(())
    }

    pub(crate) async fn mirror_user_message(
        &mut self,
        target: &EngineerNotificationTarget,
        ctx: &ThreadContext,
        thread: &Thread,
        message: &ThreadMessage,
    ) -> Result<(), ThreadBotError> {
        if message.post_id == thread.info.root_post_id {
            return Ok(());
        }

        if matches!(target, EngineerNotificationTarget::MattermostChannel { .. }) {
            self.effects.push(ThreadEffect::Reply {
                target: ThreadTarget::LinkedThreads {
                    link_kind: crate::handler::ENGINEER_LINK_KIND.to_string(),
                },
                message: format!(
                    "**User message** ([source]({}))\n\n{}",
                    crate::notifier::source_post_link(&ctx.config, &message.post_id),
                    quote_for_mattermost(&message.message)
                ),
                metadata: support_post_props("user_message", thread),
            });
        }
        Ok(())
    }

    pub(crate) fn push_effect(&mut self, effect: ThreadEffect) {
        self.effects.push(effect);
    }

    pub(crate) fn finish_request(&mut self, summary: Option<String>) {
        self.state.status = SupportThreadStatus::Finished;
        self.state.finished_summary = summary;
        self.stop_after_tools = StopAfterTools::FinishRequest;
    }

    pub(crate) fn stop_request(&mut self, summary: Option<String>) {
        self.state.status = SupportThreadStatus::Stopped;
        self.state.finished_summary = summary;
    }

    pub(crate) fn status(&self) -> &SupportThreadStatus {
        &self.state.status
    }

    pub(crate) fn finished_summary(&self) -> Option<&str> {
        self.state.finished_summary.as_deref()
    }

    pub(crate) fn stop_after_tools(&self) -> StopAfterTools {
        self.stop_after_tools
    }

    pub(crate) fn await_next_user_message(&mut self) {
        if self.stop_after_tools == StopAfterTools::Continue {
            self.stop_after_tools = StopAfterTools::AwaitNextUser;
        }
    }

    pub(crate) fn into_effects(
        mut self,
        thread: &Thread,
        trigger_message: &ThreadMessage,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        self.persist_trace_and_state(thread, trigger_message)?;
        Ok(self.effects)
    }

    fn persist_trace_and_state(
        &mut self,
        thread: &Thread,
        trigger_message: &ThreadMessage,
    ) -> Result<(), ThreadBotError> {
        let trace_metadata = with_trace_metadata(&trigger_message.metadata, &self.trace)?;
        info!(
            thread_id = %thread.info.thread_id,
            post_id = %trigger_message.post_id,
            metadata_key = "support_bot.llm_trace",
            trace_entries = self.trace.len(),
            metadata_bytes = trace_metadata.to_string().len(),
            "support-bot: queuing message metadata persistence"
        );
        self.effects.push(ThreadEffect::SetMessageMetadata {
            post_id: trigger_message.post_id.clone(),
            metadata: trace_metadata,
        });
        let thread_metadata = store_state(&thread.info.metadata, &self.state)?;
        info!(
            thread_id = %thread.info.thread_id,
            metadata_key = "support_bot",
            metadata_bytes = thread_metadata.to_string().len(),
            "support-bot: queuing thread metadata persistence"
        );
        self.effects.push(ThreadEffect::SetThreadMetadata {
            metadata: thread_metadata,
        });
        if self.stop_after_tools == StopAfterTools::FinishRequest && self.trace.is_empty() {
            warn!(
                thread_id = %thread.info.thread_id,
                post_id = %trigger_message.post_id,
                "support-bot: finishing thread with empty trace"
            );
        }
        Ok(())
    }
}
