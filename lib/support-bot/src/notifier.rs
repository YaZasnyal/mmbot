use crate::error::{Result, SupportBotError};
use crate::metadata::{metadata_value, SupportMetadata, SupportMetadataKind};
use crate::state::SupportThreadStatus;
use mattermost_api::apis::posts_api;
use mattermost_api::{apis::configuration::Configuration, models};
use std::sync::Arc;
use thread_bot::Thread;
use tracing::info;

const MIRRORED_MESSAGE_MAX_CHARS: usize = 4000;

pub(crate) struct DebugReportPoster {
    config: Arc<Configuration>,
}

impl DebugReportPoster {
    pub(crate) fn new(config: Arc<Configuration>) -> Self {
        Self { config }
    }

    // Intentional direct Mattermost API side effect for debug-report HTML upload.
    // Keep this narrow and avoid expanding it into normal support workflow paths.
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(channel_id = %channel_id, root_post_id = %root_post_id)
    )]
    pub(crate) async fn post_html_attachment_to_thread(
        &self,
        channel_id: &str,
        root_post_id: &str,
        filename: &str,
        html_content: String,
        message: String,
        props: serde_json::Value,
    ) -> Result<()> {
        info!(
            channel_id = %channel_id,
            root_post_id = %root_post_id,
            filename = %filename,
            html_bytes = html_content.len(),
            "support-bot: uploading html report attachment"
        );
        let file_id = self
            .upload_html_file(channel_id, filename, html_content.into_bytes())
            .await?;
        info!(
            channel_id = %channel_id,
            root_post_id = %root_post_id,
            file_id = %file_id,
            "support-bot: html report file uploaded"
        );

        let mut request = models::CreatePostRequest::new(channel_id.to_string(), message);
        request.root_id = Some(root_post_id.to_string());
        request.file_ids = Some(vec![file_id]);
        request.props = Some(props);

        posts_api::create_post(&self.config, request, None)
            .await
            .map_err(|error| SupportBotError::Mattermost(error.to_string()))?;
        info!(
            channel_id = %channel_id,
            root_post_id = %root_post_id,
            "support-bot: html report post created"
        );
        Ok(())
    }

    async fn upload_html_file(
        &self,
        channel_id: &str,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<String> {
        let url = format!(
            "{}/api/v4/files",
            self.config.base_path.trim_end_matches('/')
        );
        let part = reqwest::multipart::Part::bytes(bytes)
            .file_name(filename.to_string())
            .mime_str("text/html; charset=utf-8")
            .map_err(SupportBotError::Http)?;
        let form = reqwest::multipart::Form::new()
            .text("channel_id", channel_id.to_string())
            .part("files", part);

        let mut request = self.config.client.post(url).multipart(form);
        if let Some(token) = &self.config.bearer_access_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await.map_err(SupportBotError::Http)?;
        let status = response.status();
        let body = response.text().await.map_err(SupportBotError::Http)?;
        if !status.is_success() {
            return Err(SupportBotError::Mattermost(format!(
                "file upload failed with {status}: {body}"
            )));
        }

        let uploaded: models::UploadFile201Response =
            serde_json::from_str(&body).map_err(SupportBotError::Serialization)?;
        uploaded
            .file_infos
            .and_then(|infos| infos.into_iter().find_map(|info| info.id))
            .ok_or_else(|| {
                SupportBotError::Mattermost("file upload response did not contain file id".into())
            })
    }
}

pub fn render_thread_html_report(
    thread_id: &str,
    channel_id: &str,
    root_post_id: &str,
    summary: &SupportReportSummary,
    posts: &[SupportReportPost],
    traces: &[SupportReportTrace],
) -> String {
    let tool_rows = if summary.tool_calls.is_empty() {
        "<tr><td colspan=\"6\">No tool calls captured</td></tr>".to_string()
    } else {
        summary
            .tool_calls
            .iter()
            .map(|call| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    call.round,
                    escape_html(&call.post_id),
                    escape_html(&call.name),
                    escape_html(&call.call_id),
                    escape_html(&call.status),
                    if call.truncated { "yes" } else { "no" }
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let traces_by_post = traces
        .iter()
        .map(|trace| (trace.post_id.as_str(), trace.entries.as_slice()))
        .collect::<std::collections::HashMap<_, _>>();
    let mut sorted_posts = posts.iter().collect::<Vec<_>>();
    sorted_posts.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.post_id.cmp(&right.post_id))
    });

    let items = sorted_posts
        .iter()
        .map(|post| {
            let trace_block = if let Some(entries) = traces_by_post.get(post.post_id.as_str()) {
                let details = entries
                    .iter()
                    .enumerate()
                    .map(|(idx, item)| {
                        format!(
                            "<details class=\"trace\"><summary>tool_trace[{idx}]</summary><pre>{}</pre></details>",
                            escape_html(item)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("<section class=\"tool-trace\"><h3>Tool Trace (bound to this message)</h3>{details}</section>")
            } else {
                String::new()
            };
            format!(
                "<article class=\"msg\"><header><code>{}</code> · <code>{}</code> · <time>{}</time></header><pre>{}</pre>{}</article>",
                escape_html(&post.post_id),
                escape_html(&post.user_id),
                escape_html(&post.created_at),
                escape_html(&post.message),
                trace_block
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Support Thread {thread_id}</title>\
         <style>body{{font-family:ui-monospace,Menlo,Consolas,monospace;background:#f5f7fb;color:#16202a;margin:0;padding:24px}}\
         .wrap{{max-width:1100px;margin:0 auto}}h1{{margin:0 0 8px}}.meta{{background:#fff;border:1px solid #d8dee8;border-radius:10px;padding:12px;margin-bottom:16px}}\
         .msg{{background:#fff;border:1px solid #d8dee8;border-radius:10px;padding:12px;margin:0 0 10px}}header{{font-size:12px;color:#4b5563;margin-bottom:8px}}\
         .trace{{background:#fff;border:1px solid #d8dee8;border-radius:10px;padding:8px 12px;margin:0 0 10px}}\
         .tool-trace{{margin-top:12px;background:#fff8e8;border:1px solid #f1d08b;border-radius:10px;padding:10px}}\
         .tool-trace h3{{margin:0 0 8px;font-size:13px;color:#7c4f00}}\
         .summary-grid{{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:8px;margin:12px 0}}\
         .summary-item{{background:#f8fafc;border:1px solid #d8dee8;border-radius:8px;padding:10px}}\
         .summary-item b{{display:block;font-size:11px;color:#64748b;text-transform:uppercase;margin-bottom:4px}}\
         table{{width:100%;border-collapse:collapse;background:#fff;margin:10px 0 16px}}th,td{{border:1px solid #d8dee8;padding:6px 8px;text-align:left;font-size:12px}}th{{background:#f8fafc;color:#475569}}\
         pre{{white-space:pre-wrap;word-wrap:break-word;margin:0;font-family:inherit}}</style></head><body><div class=\"wrap\">\
         <h1>Support Thread Report</h1><div class=\"meta\"><div><b>thread_id:</b> <code>{thread_id}</code></div>\
         <div><b>channel_id:</b> <code>{channel_id}</code></div><div><b>root_post_id:</b> <code>{root_post_id}</code></div>\
         <div><b>messages:</b> <code>{count}</code></div><div><b>trace_posts:</b> <code>{trace_posts}</code></div>\
         <h2>Summary</h2><div class=\"summary-grid\"><div class=\"summary-item\"><b>Thread status</b><code>{thread_status}</code></div>\
         <div class=\"summary-item\"><b>Support status</b><code>{support_status}</code></div>\
         <div class=\"summary-item\"><b>Tool errors</b><code>{tool_errors}</code></div>\
         <div class=\"summary-item\"><b>Truncated results</b><code>{truncated_results}</code></div></div>\
         <h3>State</h3><pre>{state_json}</pre><h3>Tool Calls</h3><table><thead><tr><th>Round</th><th>Post</th><th>Tool</th><th>Call</th><th>Status</th><th>Truncated</th></tr></thead><tbody>{tool_rows}</tbody></table>\
         <h2>Messages</h2>{items}\
         <script>document.querySelectorAll('pre').forEach((p)=>{{if(p.textContent.length>3000){{p.dataset.full=p.textContent;p.textContent=p.textContent.slice(0,3000)+'\\n...[trimmed in view]';}}}});</script>\
         </div></body></html>",
        thread_id = escape_html(thread_id),
        channel_id = escape_html(channel_id),
        root_post_id = escape_html(root_post_id),
        count = posts.len(),
        items = items,
        trace_posts = traces.len(),
        thread_status = escape_html(&summary.thread_status),
        support_status = escape_html(&summary.support_status),
        tool_errors = summary.tool_errors,
        truncated_results = summary.truncated_results,
        state_json = escape_html(&summary.state_json),
        tool_rows = tool_rows,
    )
}

#[derive(Debug, Clone)]
pub struct SupportReportPost {
    pub post_id: String,
    pub user_id: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct SupportReportTrace {
    pub post_id: String,
    pub entries: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SupportReportSummary {
    pub thread_status: String,
    pub support_status: String,
    pub state_json: String,
    pub tool_errors: usize,
    pub truncated_results: usize,
    pub tool_calls: Vec<SupportReportToolCall>,
}

#[derive(Debug, Clone)]
pub struct SupportReportToolCall {
    pub post_id: String,
    pub round: usize,
    pub call_id: String,
    pub name: String,
    pub status: String,
    pub truncated: bool,
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) fn engineer_thread_root_message(thread: &Thread, source_link: String) -> String {
    format!(
        "New support request\n\nSource: {source_link}\n\n{}",
        quote_for_mattermost(source_message(thread))
    )
}

fn source_message(thread: &Thread) -> &str {
    thread
        .messages
        .iter()
        .find(|message| message.post_id == thread.info.root_post_id)
        .or_else(|| thread.messages.first())
        .map(|message| message.message.as_str())
        .unwrap_or("")
}

pub(crate) fn status_update_message(
    thread: &Thread,
    status: &SupportThreadStatus,
    summary: Option<&str>,
) -> String {
    let mut message = format!(
        "**Support thread status**\n\nstatus: `{}`\nsource: {}",
        support_status_label(status),
        thread.info.thread_id
    );
    if let Some(summary) = summary.and_then(non_empty_trimmed) {
        message.push_str("\nsummary:\n");
        message.push_str(&quote_for_mattermost(summary));
    }
    message
}

fn support_status_label(status: &SupportThreadStatus) -> &'static str {
    match status {
        SupportThreadStatus::Active => "active",
        SupportThreadStatus::Ignored => "ignored",
        SupportThreadStatus::Finished => "finished",
        SupportThreadStatus::Stopped => "stopped",
    }
}

fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

pub(crate) fn source_post_link(config: &Configuration, post_id: &str) -> String {
    format!(
        "{}/_redirect/pl/{}",
        config.base_path.trim_end_matches('/'),
        post_id
    )
}

pub(crate) fn quote_for_mattermost(message: &str) -> String {
    let (message, truncated) = truncate_chars(message, MIRRORED_MESSAGE_MAX_CHARS);
    let fence = markdown_fence_for(&message);
    let truncation_marker = if truncated {
        "\n\n[message truncated]"
    } else {
        ""
    };

    format!("{fence}\n{message}{truncation_marker}\n{fence}")
}

fn truncate_chars(message: &str, max_chars: usize) -> (String, bool) {
    let mut chars = message.chars();
    let truncated = message.chars().count() > max_chars;
    let value = chars.by_ref().take(max_chars).collect();
    (value, truncated)
}

fn markdown_fence_for(message: &str) -> String {
    let mut longest_run = 0;
    let mut current_run = 0;

    for ch in message.chars() {
        if ch == '`' {
            current_run += 1;
            longest_run = longest_run.max(current_run);
        } else {
            current_run = 0;
        }
    }

    "`".repeat((longest_run + 1).max(3))
}

pub(crate) fn support_post_props(kind: SupportMetadataKind, thread: &Thread) -> serde_json::Value {
    metadata_value(&SupportMetadata::for_source_thread(kind, thread))
        .unwrap_or(serde_json::Value::Null)
}

#[cfg(test)]
#[path = "tests/notifier.rs"]
mod tests;
