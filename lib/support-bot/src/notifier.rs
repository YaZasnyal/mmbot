use crate::error::{Result, SupportBotError};
use crate::metadata::{metadata_value, SupportMetadata, SupportMetadataKind};
use mattermost_api::apis::posts_api;
use mattermost_api::{apis::configuration::Configuration, models};
use std::sync::Arc;
use thread_bot::Thread;

const MIRRORED_MESSAGE_MAX_CHARS: usize = 4000;

pub(crate) struct DebugReportPoster {
    config: Arc<Configuration>,
}

impl DebugReportPoster {
    pub(crate) fn new(config: Arc<Configuration>) -> Self {
        Self { config }
    }

    pub(crate) async fn post_html_attachment_to_thread(
        &self,
        channel_id: &str,
        root_post_id: &str,
        filename: &str,
        html_content: String,
        message: String,
        props: serde_json::Value,
    ) -> Result<()> {
        let url = format!(
            "{}/api/v4/files",
            self.config.base_path.trim_end_matches('/')
        );
        let part = reqwest::multipart::Part::bytes(html_content.into_bytes())
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
        let file_id = serde_json::from_str::<models::UploadFile201Response>(&body)
            .map_err(SupportBotError::Serialization)?
            .file_infos
            .and_then(|infos| infos.into_iter().find_map(|info| info.id))
            .ok_or_else(|| {
                SupportBotError::Mattermost("file upload response did not contain file id".into())
            })?;

        let mut post = models::CreatePostRequest::new(channel_id.to_string(), message);
        post.root_id = Some(root_post_id.to_string());
        post.file_ids = Some(vec![file_id]);
        post.props = Some(props);
        posts_api::create_post(&self.config, post, None)
            .await
            .map_err(|error| SupportBotError::Mattermost(error.to_string()))?;
        Ok(())
    }
}

pub fn render_thread_html_report(
    thread_id: &str,
    channel_id: &str,
    root_post_id: &str,
    summary: &SupportReportSummary,
    posts: &[SupportReportPost],
) -> String {
    let mut posts = posts.iter().collect::<Vec<_>>();
    posts.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.post_id.cmp(&right.post_id))
    });
    let post_count = posts.len();
    let messages = posts
        .into_iter()
        .map(|post| {
            format!(
                "<article><header><code>{}</code> · <code>{}</code> · <time>{}</time></header><pre>{}</pre></article>",
                escape_html(&post.post_id),
                escape_html(&post.user_id),
                escape_html(&post.created_at),
                escape_html(&post.message)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Support Thread {}</title><style>body{{font-family:ui-monospace,Menlo,monospace;background:#f5f7fb;color:#16202a;padding:24px}}main{{max-width:1100px;margin:auto}}section,article{{background:white;border:1px solid #d8dee8;border-radius:10px;padding:12px;margin-bottom:10px}}header{{font-size:12px;color:#4b5563;margin-bottom:8px}}pre{{white-space:pre-wrap;word-wrap:break-word}}</style></head><body><main><h1>Support Thread Report</h1><section><b>thread_id:</b> <code>{}</code><br><b>channel_id:</b> <code>{}</code><br><b>root_post_id:</b> <code>{}</code><br><b>support_status:</b> <code>{}</code><h2>Runtime state</h2><pre>{}</pre></section><h2>Messages ({})</h2>{}</main></body></html>",
        escape_html(thread_id),
        escape_html(thread_id),
        escape_html(channel_id),
        escape_html(root_post_id),
        escape_html(&summary.support_status),
        escape_html(&summary.state_json),
        post_count,
        messages
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
pub struct SupportReportSummary {
    pub support_status: String,
    pub state_json: String,
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
        quote_for_mattermost(
            thread
                .messages
                .iter()
                .find(|message| message.post_id == thread.info.root_post_id)
                .or_else(|| thread.messages.first())
                .map(|message| message.message.as_str())
                .unwrap_or("")
        )
    )
}

pub(crate) fn source_post_link(config: &Configuration, post_id: &str) -> String {
    format!(
        "{}/_redirect/pl/{}",
        config.base_path.trim_end_matches('/'),
        post_id
    )
}

pub(crate) fn quote_for_mattermost(message: &str) -> String {
    let truncated = message.chars().count() > MIRRORED_MESSAGE_MAX_CHARS;
    let message = message
        .chars()
        .take(MIRRORED_MESSAGE_MAX_CHARS)
        .collect::<String>();
    let longest_ticks = message
        .split(|ch| ch != '`')
        .map(str::len)
        .max()
        .unwrap_or(0);
    let fence = "`".repeat((longest_ticks + 1).max(3));
    let marker = if truncated {
        "\n\n[message truncated]"
    } else {
        ""
    };
    format!("{fence}\n{message}{marker}\n{fence}")
}

pub(crate) fn support_post_props(kind: SupportMetadataKind, thread: &Thread) -> serde_json::Value {
    metadata_value(&SupportMetadata::for_source_thread(kind, thread))
        .unwrap_or(serde_json::Value::Null)
}

#[cfg(test)]
#[path = "tests/notifier.rs"]
mod tests;
