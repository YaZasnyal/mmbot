use crate::config::EngineerNotificationTarget;
use crate::conversation::STATE_KEY;
use crate::notifier::MattermostSupportNotifier;
use crate::output::sanitize_user_visible_message;
use crate::tools::{SupportAction, ToolResult};
use crate::user_thread_run::UserThreadRun;
use serde_json::json;
use thread_bot::{Thread, ThreadBotError, ThreadContext, ThreadEffect};
use tracing::{info, warn};

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
                    let notify_result = engineer_notifier(ctx, target)
                        .notify_engineer(thread, run.engineer_thread(), message.clone())
                        .await;

                    let thread_ref = match notify_result {
                        Ok(Some(thread_ref)) => thread_ref,
                        Ok(None) => {
                            return Ok(failed_result(call_id, "engineer thread was not created"));
                        }
                        Err(error) => {
                            return Ok(failed_result(call_id, error.to_string()));
                        }
                    };
                    run.set_engineer_thread(thread_ref);
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
            let notification_result = engineer_notifier(ctx, target)
                .notify_status_update(
                    thread,
                    run.engineer_thread(),
                    run.status(),
                    run.finished_summary(),
                )
                .await;

            match notification_result {
                Ok(Some(engineer_thread)) => {
                    run.set_engineer_thread(engineer_thread);
                    finished_result(call_id, summary, "sent", None)
                }
                Ok(None) => finished_result(call_id, summary, "skipped", None),
                Err(error) => {
                    warn!(
                        thread_id = %thread.info.thread_id,
                        error = %error,
                        "support-bot: finish status notification failed after state transition"
                    );
                    finished_result(call_id, summary, "failed", Some(error.to_string()))
                }
            }
        }
    };
    info!("support-bot: workflow action applied");
    Ok(result)
}

pub(crate) fn engineer_notifier(
    ctx: &ThreadContext,
    target: &EngineerNotificationTarget,
) -> MattermostSupportNotifier {
    MattermostSupportNotifier::new(ctx.config.clone(), target.clone())
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
