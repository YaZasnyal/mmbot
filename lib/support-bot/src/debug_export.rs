use crate::handler::ENGINEER_LINK_KIND;
use crate::metadata::{load_thread_state, metadata_value, SupportMetadata, SupportMetadataKind};
use crate::notifier::{
    render_thread_html_report, DebugReportPoster, SupportReportPost, SupportReportSummary,
};
use crate::state::SupportThreadStatus;
use thread_bot::{Thread, ThreadBotError, ThreadContext, ThreadEffect, ThreadTarget};
use tracing::{info, warn};

// Direct API use is limited to uploading the engineer-requested HTML attachment;
// ThreadEffect has no file-attachment variant.
pub(crate) async fn handle_debug_export_html(
    engineer_thread: &Thread,
    ctx: &ThreadContext,
) -> Result<Vec<ThreadEffect>, ThreadBotError> {
    let source_thread_id = ctx
        .store
        .list_reverse_thread_links(&engineer_thread.info.thread_id)
        .await?
        .into_iter()
        .find(|link| link.link_kind == ENGINEER_LINK_KIND)
        .map(|link| link.source_thread_id);
    let Some(source_thread_id) = source_thread_id else {
        warn!("support-bot: debug-report source thread link is missing");
        return Ok(vec![debug_reply(
            "Cannot find source support thread for this engineer thread.".to_string(),
        )]);
    };
    let Some(record) = ctx.store.get_thread(&source_thread_id).await? else {
        return Ok(vec![debug_reply(format!(
            "Source support thread not found: {source_thread_id}"
        ))]);
    };
    let source_thread = ctx.build_thread_snapshot(&record.thread_id).await?;
    let posts = source_thread
        .messages
        .iter()
        .map(|message| SupportReportPost {
            post_id: message.post_id.clone(),
            user_id: message.user_id.clone(),
            message: message.message.clone(),
            created_at: message.created_at.to_rfc3339(),
        })
        .collect::<Vec<_>>();
    let summary = report_summary(&record.metadata);
    let html = render_thread_html_report(
        &record.thread_id,
        &record.channel_id,
        &record.root_post_id,
        &summary,
        &posts,
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
        html_bytes = html_size,
        "support-bot: debug-report uploaded"
    );
    Ok(vec![debug_reply(format!(
        "Debug report exported for support thread `{}`.",
        record.thread_id
    ))])
}

fn debug_reply(message: String) -> ThreadEffect {
    ThreadEffect::Reply {
        target: ThreadTarget::CurrentThread,
        message,
        metadata: metadata_value(&SupportMetadata::new(SupportMetadataKind::DebugResponse))
            .unwrap_or(serde_json::Value::Null),
    }
}

fn report_summary(thread_metadata: &serde_json::Value) -> SupportReportSummary {
    match load_thread_state(thread_metadata) {
        Ok(state) => SupportReportSummary {
            support_status: status_label(&state.status).to_string(),
            state_json: serde_json::to_string_pretty(&state).unwrap_or_else(|_| "{}".to_string()),
        },
        Err(error) => SupportReportSummary {
            support_status: format!("unknown: {error}"),
            state_json: format!("failed to decode support state: {error}"),
        },
    }
}

fn status_label(status: &SupportThreadStatus) -> &'static str {
    match status {
        SupportThreadStatus::Active => "active",
        SupportThreadStatus::Ignored => "ignored",
        SupportThreadStatus::Finished => "finished",
        SupportThreadStatus::Stopped => "stopped",
    }
}
