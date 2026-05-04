use crate::config::EngineerNotificationTarget;
use crate::conversation::{load_state, load_trace, STATE_KEY};
use crate::llm::{ChatMessage, ChatRole};
use crate::notifier::{
    render_thread_html_report, MattermostSupportNotifier, SupportReportPost, SupportReportSummary,
    SupportReportToolCall, SupportReportTrace,
};
use serde_json::json;
use thread_bot::{Thread, ThreadBotError, ThreadContext, ThreadEffect, ThreadStatus};
use tracing::{info, warn};

pub(crate) async fn handle_debug_export_html(
    target: &EngineerNotificationTarget,
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

    let source_thread = ctx.build_thread_snapshot(&record.thread_id).await?;
    let posts = source_thread
        .messages
        .iter()
        .map(report_post_from_thread_message)
        .collect::<Vec<_>>();
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
                        serde_json::to_string_pretty(entry).unwrap_or_else(|_| "{}".to_string())
                    })
                    .collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    let summary = build_report_summary(record.status, &record.metadata, &traces_by_post);

    let html = render_thread_html_report(
        &record.thread_id,
        &record.channel_id,
        &record.root_post_id,
        &summary,
        &posts,
        &traces_by_post,
    );
    let html_size = html.len();
    MattermostSupportNotifier::new(ctx.config.clone(), target.clone())
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

pub(crate) fn source_user_thread_id(thread: &Thread) -> Option<String> {
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

pub(crate) fn debug_response_metadata() -> serde_json::Value {
    json!({
        STATE_KEY: {
            "kind": "debug_response"
        }
    })
}

fn report_post_from_thread_message(message: &thread_bot::ThreadMessage) -> SupportReportPost {
    SupportReportPost {
        post_id: message.post_id.clone(),
        user_id: message.user_id.clone(),
        message: message.message.clone(),
        created_at: message.created_at.to_rfc3339(),
    }
}

fn build_report_summary(
    thread_status: ThreadStatus,
    thread_metadata: &serde_json::Value,
    traces: &[SupportReportTrace],
) -> SupportReportSummary {
    let state_json = match load_state(thread_metadata) {
        Ok(state) => serde_json::to_string_pretty(&state).unwrap_or_else(|_| "{}".to_string()),
        Err(error) => format!("failed to decode support state: {error}"),
    };
    let status = serde_json::to_value(thread_status)
        .ok()
        .and_then(|value| value.as_str().map(ToString::to_string))
        .unwrap_or_else(|| format!("{thread_status:?}"));

    let mut tool_calls = Vec::new();
    let mut tool_errors = 0;
    let mut truncated_results = 0;

    for trace in traces {
        let parsed_entries = trace
            .entries
            .iter()
            .filter_map(|entry| serde_json::from_str::<ChatMessage>(entry).ok())
            .collect::<Vec<_>>();
        let mut round = 0;
        for entry in &parsed_entries {
            if entry.role != ChatRole::Assistant || entry.tool_calls.is_empty() {
                continue;
            }

            round += 1;
            for call in &entry.tool_calls {
                let tool_result = parsed_entries.iter().find(|candidate| {
                    candidate.role == ChatRole::Tool
                        && candidate.tool_call_id.as_deref() == Some(call.id.as_str())
                });
                let status = tool_result_status(tool_result);
                let truncated = tool_result
                    .and_then(|message| message.content.as_deref())
                    .is_some_and(|content| content.contains("[truncated]"));
                if status == "error" {
                    tool_errors += 1;
                }
                if truncated {
                    truncated_results += 1;
                }
                tool_calls.push(SupportReportToolCall {
                    post_id: trace.post_id.clone(),
                    round,
                    call_id: call.id.clone(),
                    name: call.name.clone(),
                    status: status.to_string(),
                    truncated,
                });
            }
        }
    }

    SupportReportSummary {
        status,
        state_json,
        tool_errors,
        truncated_results,
        tool_calls,
    }
}

fn tool_result_status(result: Option<&ChatMessage>) -> &'static str {
    let Some(result) = result else {
        return "missing";
    };
    let Some(content) = result.content.as_deref() else {
        return "ok";
    };
    match serde_json::from_str::<serde_json::Value>(content) {
        Ok(value) if value.get("is_error").and_then(serde_json::Value::as_bool) == Some(true) => {
            "error"
        }
        _ => "ok",
    }
}
