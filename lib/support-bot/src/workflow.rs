use crate::config::EngineerNotificationTarget;
use crate::conversation::STATE_KEY;
use crate::notifier::MattermostSupportNotifier;
use crate::output::sanitize_user_visible_message;
use crate::tools::{SupportAction, ToolResult};
use crate::user_thread_run::UserThreadRun;
use serde_json::json;
use thread_bot::{Thread, ThreadBotError, ThreadContext, ThreadEffect};

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
            let Some(message) = sanitize_user_visible_message(message) else {
                return Ok(ToolResult {
                    call_id: call_id.to_string(),
                    content: json!({
                        "status": "failed",
                        "error": "message contained only hidden reasoning"
                    }),
                    is_error: true,
                });
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
            ToolResult {
                call_id: call_id.to_string(),
                content: json!({ "status": "sent" }),
                is_error: false,
            }
        }
        SupportAction::NotifyEngineer { message } => {
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
            run.finish_request(summary.clone());
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

pub(crate) fn engineer_notifier(
    ctx: &ThreadContext,
    target: &EngineerNotificationTarget,
) -> MattermostSupportNotifier {
    MattermostSupportNotifier::new(ctx.config.clone(), target.clone())
}
