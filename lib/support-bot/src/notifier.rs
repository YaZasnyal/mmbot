use crate::config::EngineerNotificationTarget;
use crate::error::{Result, SupportBotError};
use crate::state::{EngineerThreadRef, SupportThreadStatus};
use mattermost_api::apis::posts_api;
use mattermost_api::{apis::configuration::Configuration, models};
use std::sync::Arc;
use thread_bot::Thread;
use tracing::info;

const MIRRORED_MESSAGE_MAX_CHARS: usize = 4000;

pub struct MattermostSupportNotifier {
    config: Arc<Configuration>,
    target: EngineerNotificationTarget,
}

impl MattermostSupportNotifier {
    pub fn new(config: Arc<Configuration>, target: EngineerNotificationTarget) -> Self {
        Self { config, target }
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(thread_id = %thread.info.thread_id, source_root_post_id = %thread.info.root_post_id)
    )]
    pub async fn ensure_engineer_thread(
        &self,
        thread: &Thread,
        current_thread: Option<&EngineerThreadRef>,
    ) -> Result<Option<EngineerThreadRef>> {
        match &self.target {
            EngineerNotificationTarget::SameThread => Ok(None),
            EngineerNotificationTarget::MattermostChannel { channel_id } => {
                if let Some(current_thread) = current_thread {
                    info!(
                        source_thread_id = %thread.info.thread_id,
                        source_root_post_id = %thread.info.root_post_id,
                        engineer_channel_id = %channel_id,
                        engineer_root_post_id = %current_thread.root_post_id,
                        created_or_reused = "reused",
                        "support-bot: engineer thread reused"
                    );
                    return Ok(Some(current_thread.clone()));
                }

                let request = engineer_thread_root_request(
                    channel_id,
                    thread,
                    source_post_link(&self.config, &thread.info.root_post_id),
                );
                let created = posts_api::create_post(&self.config, request, None)
                    .await
                    .map_err(|error| SupportBotError::Mattermost(error.to_string()))?;
                info!(
                    source_thread_id = %thread.info.thread_id,
                    source_root_post_id = %thread.info.root_post_id,
                    engineer_channel_id = %channel_id,
                    engineer_root_post_id = %created.id,
                    created_or_reused = "created",
                    "support-bot: engineer thread created"
                );

                Ok(Some(EngineerThreadRef {
                    channel_id: channel_id.clone(),
                    root_post_id: created.id,
                }))
            }
        }
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(thread_id = %thread.info.thread_id, source_root_post_id = %thread.info.root_post_id)
    )]
    pub async fn notify_engineer(
        &self,
        thread: &Thread,
        current_thread: Option<&EngineerThreadRef>,
        message: String,
    ) -> Result<Option<EngineerThreadRef>> {
        match &self.target {
            EngineerNotificationTarget::SameThread => Ok(None),
            EngineerNotificationTarget::MattermostChannel { channel_id } => {
                let thread_ref = self
                    .ensure_engineer_thread(thread, current_thread)
                    .await?
                    .ok_or_else(|| {
                        SupportBotError::Internal("engineer thread was not created".into())
                    })?;
                let message_bytes = message.len();
                let request = engineer_thread_reply_request(
                    channel_id,
                    &thread_ref.root_post_id,
                    thread,
                    message,
                );

                posts_api::create_post(&self.config, request, None)
                    .await
                    .map_err(|error| SupportBotError::Mattermost(error.to_string()))?;
                info!(
                    source_thread_id = %thread.info.thread_id,
                    engineer_root_post_id = %thread_ref.root_post_id,
                    message_bytes,
                    "support-bot: engineer notification posted"
                );

                Ok(Some(thread_ref))
            }
        }
    }

    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(thread_id = %thread.info.thread_id, source_post_id = %message.post_id)
    )]
    pub async fn mirror_user_message(
        &self,
        thread: &Thread,
        current_thread: Option<&EngineerThreadRef>,
        message: &thread_bot::ThreadMessage,
    ) -> Result<Option<EngineerThreadRef>> {
        self.post_engineer_thread_message(
            thread,
            current_thread,
            "user_message",
            format!(
                "**User message** ([source]({}))\n\n{}",
                source_post_link(&self.config, &message.post_id),
                quote_for_mattermost(&message.message)
            ),
        )
        .await
    }

    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(thread_id = %thread.info.thread_id)
    )]
    pub async fn mirror_bot_message(
        &self,
        thread: &Thread,
        current_thread: Option<&EngineerThreadRef>,
        message: String,
    ) -> Result<Option<EngineerThreadRef>> {
        self.post_engineer_thread_message(
            thread,
            current_thread,
            "bot_message",
            format!("**Bot message**\n\n{}", quote_for_mattermost(&message)),
        )
        .await
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(thread_id = %thread.info.thread_id, status = support_status_label(status))
    )]
    pub async fn notify_status_update(
        &self,
        thread: &Thread,
        current_thread: Option<&EngineerThreadRef>,
        status: &SupportThreadStatus,
        summary: Option<&str>,
    ) -> Result<Option<EngineerThreadRef>> {
        match &self.target {
            EngineerNotificationTarget::SameThread => Ok(None),
            EngineerNotificationTarget::MattermostChannel { channel_id } => {
                let thread_ref = self
                    .ensure_engineer_thread(thread, current_thread)
                    .await?
                    .ok_or_else(|| {
                        SupportBotError::Internal("engineer thread was not created".into())
                    })?;
                let message = status_update_message(thread, status, summary);
                let message_bytes = message.len();
                let mut request = engineer_thread_reply_request(
                    channel_id,
                    &thread_ref.root_post_id,
                    thread,
                    message,
                );
                request.props = Some(support_post_props("status_update", thread));

                posts_api::create_post(&self.config, request, None)
                    .await
                    .map_err(|error| SupportBotError::Mattermost(error.to_string()))?;
                info!(
                    source_thread_id = %thread.info.thread_id,
                    engineer_root_post_id = %thread_ref.root_post_id,
                    status = support_status_label(status),
                    message_bytes,
                    "support-bot: status update posted to engineer thread"
                );

                Ok(Some(thread_ref))
            }
        }
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(channel_id = %channel_id, root_post_id = %root_post_id)
    )]
    pub async fn post_html_attachment_to_thread(
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

    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(thread_id = %thread.info.thread_id, kind = %kind)
    )]
    async fn post_engineer_thread_message(
        &self,
        thread: &Thread,
        current_thread: Option<&EngineerThreadRef>,
        kind: &str,
        message: String,
    ) -> Result<Option<EngineerThreadRef>> {
        match &self.target {
            EngineerNotificationTarget::SameThread => Ok(None),
            EngineerNotificationTarget::MattermostChannel { channel_id } => {
                let thread_ref = self
                    .ensure_engineer_thread(thread, current_thread)
                    .await?
                    .ok_or_else(|| {
                        SupportBotError::Internal("engineer thread was not created".into())
                    })?;
                let message_bytes = message.len();
                let mut request = engineer_thread_reply_request(
                    channel_id,
                    &thread_ref.root_post_id,
                    thread,
                    message,
                );
                request.props = Some(support_post_props(kind, thread));

                posts_api::create_post(&self.config, request, None)
                    .await
                    .map_err(|error| SupportBotError::Mattermost(error.to_string()))?;
                info!(
                    source_thread_id = %thread.info.thread_id,
                    engineer_root_post_id = %thread_ref.root_post_id,
                    kind = %kind,
                    message_bytes,
                    "support-bot: mirrored message posted to engineer thread"
                );

                Ok(Some(thread_ref))
            }
        }
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

fn engineer_thread_root_request(
    channel_id: &str,
    thread: &Thread,
    source_link: String,
) -> models::CreatePostRequest {
    let mut request = models::CreatePostRequest::new(
        channel_id.to_string(),
        format!(
            "New support request\n\nSource: {source_link}\n\n{}",
            quote_for_mattermost(source_message(thread))
        ),
    );
    request.props = Some(support_post_props("engineer_thread_root", thread));
    request
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

fn engineer_thread_reply_request(
    channel_id: &str,
    root_post_id: &str,
    thread: &Thread,
    message: String,
) -> models::CreatePostRequest {
    let mut request = models::CreatePostRequest::new(channel_id.to_string(), message);
    request.root_id = Some(root_post_id.to_string());
    request.props = Some(support_post_props("engineer_notification", thread));
    request
}

fn status_update_message(
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

fn source_post_link(config: &Configuration, post_id: &str) -> String {
    format!(
        "{}/_redirect/pl/{}",
        config.base_path.trim_end_matches('/'),
        post_id
    )
}

fn quote_for_mattermost(message: &str) -> String {
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

fn support_post_props(kind: &str, thread: &Thread) -> serde_json::Value {
    serde_json::json!({
        "support_bot": {
            "kind": kind,
            "source_thread_id": thread.info.thread_id,
            "source_root_post_id": thread.info.root_post_id,
            "source_channel_id": thread.info.channel_id
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use thread_bot::{ThreadInfo, ThreadMessage};

    fn thread() -> Thread {
        Thread {
            info: ThreadInfo {
                thread_id: "thread-1".to_string(),
                root_post_id: "root-1".to_string(),
                channel_id: "users".to_string(),
                creator_user_id: "user-1".to_string(),
                metadata: serde_json::json!({}),
                last_seen_post_id: None,
                last_seen_post_at: None,
                last_processed_post_id: None,
                last_processed_post_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            messages: vec![ThreadMessage {
                post_id: "root-1".to_string(),
                thread_id: "thread-1".to_string(),
                user_id: "user-1".to_string(),
                message: "service is down".to_string(),
                root_id: None,
                parent_post_id: None,
                props: serde_json::json!({}),
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                is_new: true,
            }],
            reactions: Vec::new(),
        }
    }

    #[test]
    fn engineer_root_request_links_and_quotes_source_post() {
        let request = engineer_thread_root_request(
            "engineers",
            &thread(),
            "http://localhost:8065/_redirect/pl/root-1".to_string(),
        );

        assert_eq!(request.channel_id, "engineers");
        assert_eq!(request.root_id, None);
        assert!(request
            .message
            .contains("Source: http://localhost:8065/_redirect/pl/root-1"));
        assert!(request.message.contains("```\nservice is down\n```"));
        assert_eq!(
            request.props.unwrap()["support_bot"]["kind"],
            "engineer_thread_root"
        );
    }

    #[test]
    fn engineer_reply_request_posts_into_existing_thread() {
        let request = engineer_thread_reply_request(
            "engineers",
            "eng-root",
            &thread(),
            "debug info".to_string(),
        );

        assert_eq!(request.channel_id, "engineers");
        assert_eq!(request.root_id.as_deref(), Some("eng-root"));
        assert_eq!(request.message, "debug info");
    }

    #[test]
    fn status_update_message_includes_status_and_summary() {
        let message = status_update_message(
            &thread(),
            &SupportThreadStatus::Finished,
            Some("resolved by cache flush"),
        );

        assert!(message.contains("status: `finished`"));
        assert!(message.contains("source: thread-1"));
        assert!(message.contains("resolved by cache flush"));
    }

    #[test]
    fn source_link_uses_mattermost_redirect_url() {
        let config = Configuration {
            base_path: "http://localhost:8065/".to_string(),
            ..Configuration::default()
        };

        assert_eq!(
            source_post_link(&config, "post-1"),
            "http://localhost:8065/_redirect/pl/post-1"
        );
    }

    #[test]
    fn mattermost_quote_uses_fence_that_survives_backticks() {
        let quoted = quote_for_mattermost("before\n```json\n{}\n```\nafter");

        assert!(quoted.starts_with("````\n"));
        assert!(quoted.ends_with("\n````"));
        assert!(quoted.contains("```json"));
    }

    #[test]
    fn mattermost_quote_truncates_large_messages() {
        let quoted = quote_for_mattermost(&"x".repeat(MIRRORED_MESSAGE_MAX_CHARS + 1));

        assert!(quoted.contains("[message truncated]"));
        assert!(quoted.len() < MIRRORED_MESSAGE_MAX_CHARS + 100);
    }

    #[test]
    fn html_report_includes_summary_and_tool_table() {
        let summary = SupportReportSummary {
            thread_status: "resolved".to_string(),
            support_status: "finished".to_string(),
            state_json: "{\"status\":\"finished\"}".to_string(),
            tool_errors: 1,
            truncated_results: 1,
            tool_calls: vec![SupportReportToolCall {
                post_id: "post-1".to_string(),
                round: 1,
                call_id: "call-1".to_string(),
                name: "instructions".to_string(),
                status: "error".to_string(),
                truncated: true,
            }],
        };
        let html = render_thread_html_report(
            "thread-1",
            "users",
            "root-1",
            &summary,
            &[SupportReportPost {
                post_id: "post-1".to_string(),
                user_id: "user-1".to_string(),
                message: "help".to_string(),
                created_at: "now".to_string(),
            }],
            &[],
        );

        assert!(html.contains("<h2>Summary</h2>"));
        assert!(html.contains("Thread status"));
        assert!(html.contains("<code>resolved</code>"));
        assert!(html.contains("Support status"));
        assert!(html.contains("<code>finished</code>"));
        assert!(html.contains("instructions"));
        assert!(html.contains("call-1"));
        assert!(html.contains("Tool errors"));
    }

    #[test]
    fn html_report_orders_messages_by_created_at() {
        let summary = SupportReportSummary {
            thread_status: "active".to_string(),
            support_status: "active".to_string(),
            state_json: "{}".to_string(),
            tool_errors: 0,
            truncated_results: 0,
            tool_calls: Vec::new(),
        };
        let html = render_thread_html_report(
            "thread-1",
            "users",
            "root-1",
            &summary,
            &[
                SupportReportPost {
                    post_id: "post-2".to_string(),
                    user_id: "user-1".to_string(),
                    message: "second".to_string(),
                    created_at: "2026-05-04T20:00:02+00:00".to_string(),
                },
                SupportReportPost {
                    post_id: "post-1".to_string(),
                    user_id: "user-1".to_string(),
                    message: "first".to_string(),
                    created_at: "2026-05-04T20:00:01+00:00".to_string(),
                },
            ],
            &[],
        );

        assert!(html.find("first").unwrap() < html.find("second").unwrap());
    }
}
