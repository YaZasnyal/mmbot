use crate::config::EngineerNotificationTarget;
use crate::conversation::STATE_KEY;
use crate::handler::ENGINEER_LINK_KIND;
use crate::notifier::{status_update_message, support_post_props};
use crate::output::sanitize_user_visible_message;
use crate::tools::{SupportAction, ToolResult};
use crate::user_thread_run::UserThreadRun;
use serde_json::json;
use thread_bot::{Thread, ThreadBotError, ThreadContext, ThreadEffect, ThreadTarget};
use tracing::info;

#[tracing::instrument(
    level = "info",
    skip_all,
    fields(thread_id = %thread.info.thread_id, tool_call_id = %call_id, action = tracing::field::Empty)
)]
pub(crate) async fn apply_action(
    target: &EngineerNotificationTarget,
    ctx: &ThreadContext,
    thread: &Thread,
    run: &mut UserThreadRun,
    call_id: &str,
    action: SupportAction,
) -> Result<ToolResult, ThreadBotError> {
    let result = match action {
        SupportAction::SendUserMessage { message } => {
            tracing::Span::current().record("action", tracing::field::display("send_user_message"));
            let Some(message) = sanitize_user_visible_message(message) else {
                return Ok(failed_result(
                    call_id,
                    "message contained only hidden reasoning",
                ));
            };
            run.reply(
                target,
                ctx,
                thread,
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
            run.await_next_user_message();
            sent_result(call_id)
        }
        SupportAction::NotifyEngineer { message } => {
            tracing::Span::current().record("action", tracing::field::display("notify_engineer"));
            match target {
                EngineerNotificationTarget::SameThread => {
                    run.push_effect(ThreadEffect::Reply {
                        target: ThreadTarget::CurrentThread,
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
                    run.push_effect(ThreadEffect::Reply {
                        target: ThreadTarget::LinkedThreads {
                            link_kind: ENGINEER_LINK_KIND.to_string(),
                        },
                        message: message.clone(),
                        metadata: support_post_props("engineer_notification", thread),
                    });
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
        SupportAction::FinishRequest { summary } => {
            tracing::Span::current().record("action", tracing::field::display("finish_request"));
            run.finish_request(summary.clone());
            match target {
                EngineerNotificationTarget::SameThread => {
                    finished_result(call_id, summary, "skipped", None)
                }
                EngineerNotificationTarget::MattermostChannel { .. } => {
                    run.push_effect(ThreadEffect::Reply {
                        target: ThreadTarget::LinkedThreads {
                            link_kind: ENGINEER_LINK_KIND.to_string(),
                        },
                        message: status_update_message(
                            thread,
                            run.status(),
                            run.finished_summary(),
                        ),
                        metadata: support_post_props("status_update", thread),
                    });
                    finished_result(call_id, summary, "queued", None)
                }
            }
        }
    };
    info!("support-bot: workflow action applied");
    Ok(result)
}

fn sent_result(call_id: &str) -> ToolResult {
    ToolResult {
        call_id: call_id.to_string(),
        content: json!({ "status": "sent" }),
        is_error: false,
    }
}

fn failed_result(call_id: &str, error: impl Into<String>) -> ToolResult {
    ToolResult {
        call_id: call_id.to_string(),
        content: json!({
            "status": "failed",
            "error": error.into()
        }),
        is_error: true,
    }
}

fn finished_result(
    call_id: &str,
    summary: Option<String>,
    notification_status: &'static str,
    notification_error: Option<String>,
) -> ToolResult {
    let is_error = notification_error.is_some();
    ToolResult {
        call_id: call_id.to_string(),
        content: json!({
            "status": "finished",
            "summary": summary,
            "notification_status": notification_status,
            "notification_error": notification_error
        }),
        is_error,
    }
}
