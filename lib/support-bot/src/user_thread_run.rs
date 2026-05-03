use crate::config::EngineerNotificationTarget;
use crate::conversation::{store_state, with_trace_metadata};
use crate::llm::{ChatMessage, ChatRole};
use crate::state::{EngineerThreadRef, SupportThreadState, SupportThreadStatus};
use crate::tools::ToolCall;
use crate::workflow::engineer_notifier;
use thread_bot::{Thread, ThreadBotError, ThreadContext, ThreadEffect, ThreadMessage};

pub(crate) struct UserThreadRun {
    state: SupportThreadState,
    messages: Vec<ChatMessage>,
    trace: Vec<ChatMessage>,
    effects: Vec<ThreadEffect>,
    stop_after_tools: bool,
}

impl UserThreadRun {
    pub(crate) fn new(state: SupportThreadState, messages: Vec<ChatMessage>) -> Self {
        Self {
            state,
            messages,
            trace: Vec::new(),
            effects: Vec::new(),
            stop_after_tools: false,
        }
    }

    pub(crate) fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub(crate) fn trace_len(&self) -> usize {
        self.trace.len()
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
        ctx: &ThreadContext,
        thread: &Thread,
        message: String,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        self.effects.push(ThreadEffect::Reply {
            message: message.clone(),
            metadata,
        });
        if let Some(engineer_thread) = engineer_notifier(ctx, target)
            .mirror_bot_message(thread, self.state.engineer_thread.as_ref(), message)
            .await?
        {
            self.state.engineer_thread = Some(engineer_thread);
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

        if let Some(engineer_thread) = engineer_notifier(ctx, target)
            .mirror_user_message(thread, self.state.engineer_thread.as_ref(), message)
            .await?
        {
            self.state.engineer_thread = Some(engineer_thread);
        }
        Ok(())
    }

    pub(crate) fn push_effect(&mut self, effect: ThreadEffect) {
        self.effects.push(effect);
    }

    pub(crate) fn engineer_thread(&self) -> Option<&EngineerThreadRef> {
        self.state.engineer_thread.as_ref()
    }

    pub(crate) fn set_engineer_thread(&mut self, engineer_thread: EngineerThreadRef) {
        self.state.engineer_thread = Some(engineer_thread);
    }

    pub(crate) fn finish_request(&mut self, summary: Option<String>) {
        self.state.status = SupportThreadStatus::Finished;
        self.state.finished_summary = summary;
        self.stop_after_tools = true;
    }

    pub(crate) fn should_stop_after_tools(&self) -> bool {
        self.stop_after_tools
    }

    pub(crate) fn into_effects(
        mut self,
        thread: &Thread,
        trigger_message: &ThreadMessage,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        self.persist_trace_and_state(thread, trigger_message)?;
        Ok(self.effects)
    }

    pub(crate) fn into_resolved_effects(
        mut self,
        thread: &Thread,
        trigger_message: &ThreadMessage,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        self.persist_trace_and_state(thread, trigger_message)?;
        self.effects.push(ThreadEffect::MarkResolved);
        Ok(self.effects)
    }

    pub(crate) fn into_stopped_effects(
        mut self,
        thread: &Thread,
        trigger_message: &ThreadMessage,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        self.persist_trace_and_state(thread, trigger_message)?;
        self.effects.push(ThreadEffect::MarkStopped);
        Ok(self.effects)
    }

    fn persist_trace_and_state(
        &mut self,
        thread: &Thread,
        trigger_message: &ThreadMessage,
    ) -> Result<(), ThreadBotError> {
        let trace_metadata = with_trace_metadata(&trigger_message.metadata, &self.trace)?;
        self.effects.push(ThreadEffect::SetMessageMetadata {
            post_id: trigger_message.post_id.clone(),
            metadata: trace_metadata,
        });
        self.effects.push(ThreadEffect::SetThreadMetadata {
            metadata: store_state(&thread.info.metadata, &self.state)?,
        });
        Ok(())
    }
}
