use crate::config::{EngineerNotificationTarget, SupportBotConfig};
use crate::debug::{DebugCommand, DebugCommandHandler};
use crate::error::SupportBotError;
use crate::llm::{ChatMessage, ChatRole, LlmClient, LlmRequest};
use crate::notifier::{
    render_thread_html_report, MattermostSupportNotifier, SupportReportPost, SupportReportTrace,
};
use crate::state::SupportThreadState;
use crate::tools::{SupportAction, ToolContext, ToolExecutionOutcome, ToolRegistry, ToolResult};
use async_trait::async_trait;
use mattermost_api::apis::posts_api;
use serde_json::json;
use std::sync::Arc;
use thread_bot::{
    Thread, ThreadBotError, ThreadContext, ThreadEffect, ThreadHandler, ThreadMessage,
};
use tracing::{info, warn};

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
            return self.handle_debug_export_html(thread, ctx).await;
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
                info!(
                    thread_id = %thread.info.thread_id,
                    post_id = %trigger_message.post_id,
                    trace_len = trace.len(),
                    "support-bot: llm completed without further tools"
                );
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
                let trace_metadata = with_trace_metadata(
                    &trigger_message.metadata,
                    &trace,
                )?;
                effects.push(ThreadEffect::SetMessageMetadata {
                    post_id: trigger_message.post_id.clone(),
                    metadata: trace_metadata,
                });
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
            info!(
                thread_id = %thread.info.thread_id,
                post_id = %trigger_message.post_id,
                tool_calls = tool_calls.len(),
                "support-bot: executing tool calls from llm response"
            );

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
            }
        }

        let failure_message = "I ran into an internal issue while processing your request. "
            .to_string()
            + "I've notified the engineering team and stopped this thread for now.";

        let trace_summary = if trace.is_empty() {
            "(no trace captured)".to_string()
        } else {
            format!(
                "trace_messages={}, last_trace={}",
                trace.len(),
                serde_json::to_string(trace.last().unwrap_or(&ChatMessage::assistant("none")))
                    .unwrap_or_else(|_| "{}".to_string())
            )
        };

        let engineer_error_message = format!(
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
            self.config.limits.max_tool_rounds,
            self.config.limits.max_tool_calls_per_round,
            truncate_utf8(trace_summary, 2000)
        );

        if let Ok(Some(engineer_thread)) = self
            .engineer_notifier(ctx)
            .notify_engineer(
                thread,
                state.engineer_thread.as_ref(),
                engineer_error_message,
            )
            .await
        {
            state.engineer_thread = Some(engineer_thread);
        }

        let trace_metadata = with_trace_metadata(
            &trigger_message.metadata,
            &trace,
        )?;
        effects.push(ThreadEffect::SetMessageMetadata {
            post_id: trigger_message.post_id.clone(),
            metadata: trace_metadata,
        });
        self.push_reply_effect(
            ctx,
            thread,
            &mut state,
            &mut effects,
            failure_message,
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
        effects.push(ThreadEffect::MarkStopped);
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
                tool_calls: load_tool_calls(&message.metadata),
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

    async fn handle_debug_export_html(
        &self,
        engineer_thread: &Thread,
        ctx: &ThreadContext,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        let Some(source_thread_id) = source_user_thread_id(engineer_thread) else {
            warn!(
                engineer_thread_id = %engineer_thread.info.thread_id,
                "support-bot: debug-report source thread id missing in engineer thread props"
            );
            return Ok(vec![ThreadEffect::Reply {
                message: "Cannot find source support thread for this engineer thread.".to_string(),
                metadata: debug_response_metadata(),
            }]);
        };
        info!(
            engineer_thread_id = %engineer_thread.info.thread_id,
            source_thread_id = %source_thread_id,
            "support-bot: exporting debug-report"
        );
        let Some(record) = ctx.store.get_thread(&source_thread_id).await? else {
            warn!(
                source_thread_id = %source_thread_id,
                "support-bot: debug-report source thread not found in store"
            );
            return Ok(vec![ThreadEffect::Reply {
                message: format!("Source support thread not found: {source_thread_id}"),
                metadata: debug_response_metadata(),
            }]);
        };

        let post_list = posts_api::get_post_thread(
            &ctx.config,
            &record.root_post_id,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .map_err(|error| {
            ThreadBotError::internal(SupportBotError::Mattermost(error.to_string()))
        })?;

        let mut posts = Vec::new();
        if let (Some(order), Some(map)) = (post_list.order, post_list.posts) {
            for post_id in order {
                if let Some(post) = map.get(&post_id) {
                    posts.push(report_post_from_mm_post(post));
                }
            }
        }
        info!(
            source_thread_id = %record.thread_id,
            messages = posts.len(),
            "support-bot: fetched source thread posts for debug-report"
        );

        let traces_by_post = ctx
            .store
            .list_thread_messages(&record.thread_id)
            .await?
            .into_iter()
            .filter_map(|message| {
                let trace = load_trace(&message.metadata).ok()?;
                if trace.is_empty() {
                    return None;
                }
                Some(SupportReportTrace {
                    post_id: message.post_id,
                    entries: trace
                        .iter()
                        .map(|entry| {
                            serde_json::to_string_pretty(entry)
                                .unwrap_or_else(|_| "{}".to_string())
                        })
                        .collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>();

        let html = render_thread_html_report(
            &record.thread_id,
            &record.channel_id,
            &record.root_post_id,
            &posts,
            &traces_by_post,
        );
        let html_size = html.len();
        self.engineer_notifier(ctx)
            .post_html_attachment_to_thread(
                &engineer_thread.info.channel_id,
                &engineer_thread.info.root_post_id,
                &format!("support-thread-{}.html", record.thread_id),
                html,
                format!(
                    "Attached: support thread HTML export for `{}`.",
                    record.thread_id
                ),
                json!({
                    STATE_KEY: {
                        "kind": "thread_html_report",
                        "source_thread_id": record.thread_id,
                        "reason": "engineer_request"
                    }
                }),
            )
            .await?;
        info!(
            source_thread_id = %record.thread_id,
            engineer_thread_id = %engineer_thread.info.thread_id,
            html_bytes = html_size,
            "support-bot: debug-report uploaded to engineer thread"
        );

        Ok(vec![ThreadEffect::Reply {
            message: format!(
                "Debug report exported for support thread `{}`.",
                record.thread_id
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

fn timestamp_ms_to_rfc3339(timestamp_ms: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(timestamp_ms)
        .map(|ts| ts.to_rfc3339())
        .unwrap_or_else(|| timestamp_ms.to_string())
}

fn report_post_from_mm_post(post: &mattermost_api::models::Post) -> SupportReportPost {
    SupportReportPost {
        post_id: post.id.clone(),
        user_id: post
            .user_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        message: post.message.clone().unwrap_or_default(),
        created_at: post
            .create_at
            .map(timestamp_ms_to_rfc3339)
            .unwrap_or_else(|| "unknown".to_string()),
    }
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

fn load_tool_calls(metadata: &serde_json::Value) -> Vec<crate::tools::ToolCall> {
    load_trace(metadata)
        .unwrap_or_default()
        .into_iter()
        .filter(|message| message.role == ChatRole::Assistant)
        .flat_map(|message| message.tool_calls)
        .collect()
}

fn with_trace_metadata(
    metadata: &serde_json::Value,
    trace: &[ChatMessage],
) -> Result<serde_json::Value, ThreadBotError> {
    if trace.is_empty() {
        return Ok(metadata.clone());
    }

    let mut metadata = metadata.clone();
    if !metadata.is_object() {
        metadata = json!({});
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
}
