use crate::handler::ENGINEER_LINK_KIND;
use crate::llm::{ChatMessage, ChatRole};
use crate::metadata::{
    load_message_trace, load_thread_state, metadata_value, SupportMetadata, SupportMetadataKind,
};
use crate::notifier::{
    render_thread_html_report, DebugReportPoster, SupportReportPost, SupportReportSummary,
    SupportReportToolCall, SupportReportTrace,
};
use crate::state::SupportThreadStatus;
use thread_bot::{Thread, ThreadBotError, ThreadContext, ThreadEffect, ThreadTarget};
use tracing::{info, warn};

// NOTE: This module is the only intentional direct Mattermost side-effect path.
// We upload an HTML attachment and create a post via API here because
// ThreadEffect currently has no file-attachment variant.
pub(crate) async fn handle_debug_export_html(
    engineer_thread: &Thread,
    ctx: &ThreadContext,
) -> Result<Vec<ThreadEffect>, ThreadBotError> {
    let source_links = ctx
        .store
        .list_reverse_thread_links(&engineer_thread.info.thread_id)
        .await?;
    let source_link = source_links
        .iter()
        .find(|link| link.link_kind == ENGINEER_LINK_KIND);
    let Some(source_thread_id) = source_link.map(|link| link.source_thread_id.as_str()) else {
        warn!(
            engineer_thread_id = %engineer_thread.info.thread_id,
            "support-bot: debug-report source thread link is missing"
        );
        return Ok(vec![ThreadEffect::Reply {
            target: ThreadTarget::CurrentThread,
            message: "Cannot find source support thread for this engineer thread.".to_string(),
            metadata: debug_response_metadata(),
        }]);
    };
    info!(
        engineer_thread_id = %engineer_thread.info.thread_id,
        source_thread_id = %source_thread_id,
        "support-bot: exporting debug-report"
    );
    let Some(record) = ctx.store.get_thread(source_thread_id).await? else {
        warn!(
            source_thread_id = %source_thread_id,
            "support-bot: debug-report source thread not found in store"
        );
        return Ok(vec![ThreadEffect::Reply {
            target: ThreadTarget::CurrentThread,
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
            let trace = load_message_trace(&message.metadata).ok()?;
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
    let summary = build_report_summary(&record.metadata, &traces_by_post);

    let html = render_thread_html_report(
        &record.thread_id,
        &record.channel_id,
        &record.root_post_id,
        &summary,
        &posts,
        &traces_by_post,
    );
    let html_size = html.len();
    DebugReportPoster::new(ctx.config.clone())
        .post_html_attachment_to_thread(
            &engineer_thread.info.channel_id,
            &engineer_thread.info.root_post_id,
            &format!("support-thread-{}.html", record.thread_id),
            html,
            format!(
                "Attached: support thread HTML export for `{}`.",
                record.thread_id
            ),
            metadata_value(&SupportMetadata::thread_html_report(&record.thread_id))?,
        )
        .await?;
    info!(
        source_thread_id = %record.thread_id,
        engineer_thread_id = %engineer_thread.info.thread_id,
        html_bytes = html_size,
        "support-bot: debug-report uploaded to engineer thread"
    );

    Ok(vec![ThreadEffect::Reply {
        target: ThreadTarget::CurrentThread,
        message: format!(
            "Debug report exported for support thread `{}`.",
            record.thread_id
        ),
        metadata: debug_response_metadata(),
    }])
}

pub(crate) fn debug_response_metadata() -> serde_json::Value {
    metadata_value(&SupportMetadata::new(SupportMetadataKind::DebugResponse))
        .unwrap_or(serde_json::Value::Null)
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
    thread_metadata: &serde_json::Value,
    traces: &[SupportReportTrace],
) -> SupportReportSummary {
    let decoded_state = load_thread_state(thread_metadata);
    let support_status = decoded_state
        .as_ref()
        .map(|state| support_status_label(&state.status).to_string())
        .unwrap_or_else(|error| format!("unknown: {error}"));
    let state_json = match decoded_state {
        Ok(state) => serde_json::to_string_pretty(&state).unwrap_or_else(|_| "{}".to_string()),
        Err(error) => format!("failed to decode support state: {error}"),
    };
    let thread_status = "tracked".to_string();

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
        thread_status,
        support_status,
        state_json,
        tool_errors,
        truncated_results,
        tool_calls,
    }
}

fn support_status_label(status: &SupportThreadStatus) -> &'static str {
    match status {
        SupportThreadStatus::Active => "active",
        SupportThreadStatus::Ignored => "ignored",
        SupportThreadStatus::Finished => "finished",
        SupportThreadStatus::Stopped => "stopped",
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
