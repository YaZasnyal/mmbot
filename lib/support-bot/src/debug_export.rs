use crate::config::EngineerNotificationTarget;
use crate::conversation::{load_trace, STATE_KEY};
use crate::error::SupportBotError;
use crate::notifier::{
    render_thread_html_report, MattermostSupportNotifier, SupportReportPost, SupportReportTrace,
};
use mattermost_api::apis::posts_api;
use serde_json::json;
use thread_bot::{Thread, ThreadBotError, ThreadContext, ThreadEffect};
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
    .map_err(|error| ThreadBotError::internal(SupportBotError::Mattermost(error.to_string())))?;

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
                        serde_json::to_string_pretty(entry).unwrap_or_else(|_| "{}".to_string())
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
