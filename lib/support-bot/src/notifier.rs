use crate::acp::AcpSessionTrace;
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
    trace: Option<&AcpSessionTrace>,
) -> String {
    let post_count = posts.len();
    let trace_count = trace.map_or(0, |trace| trace.events.len());
    let mut timeline = posts
        .iter()
        .map(|post| {
            (
                post.created_at.as_str(),
                post.post_id.as_str(),
                format!(
                    "<article class=\"event mattermost\"><header><span class=\"badge\">Mattermost</span><time>{}</time></header><h3>{} <small>{}</small></h3><pre>{}</pre></article>",
                    escape_html(&post.created_at),
                    escape_html(&post.user_id),
                    escape_html(&post.post_id),
                    escape_html(&post.message)
                ),
            )
        })
        .collect::<Vec<_>>();
    if let Some(trace) = trace {
        timeline.extend(trace.events.iter().map(|event| {
            let class = match event.kind.as_str() {
                "tool_call" | "tool_result" => "tool",
                "reasoning" => "reasoning",
                "system" => "system",
                _ => "agent",
            };
            (
                event.timestamp.as_str(),
                event.title.as_str(),
                format!(
                    "<article class=\"event {class}\"><header><span class=\"badge\">Qwen · {}</span><time>{}</time></header><h3>{}</h3><pre>{}</pre></article>",
                    escape_html(&event.kind.replace('_', " ")),
                    escape_html(&event.timestamp),
                    escape_html(&event.title),
                    escape_html(&event.body)
                ),
            )
        }));
    }
    timeline.sort_by(|left, right| left.0.cmp(right.0).then_with(|| left.1.cmp(right.1)));
    let timeline = timeline
        .into_iter()
        .map(|(_, _, html)| html)
        .collect::<Vec<_>>()
        .join("\n");
    let warnings = trace
        .filter(|trace| !trace.warnings.is_empty())
        .map(|trace| {
            format!(
                "<section class=\"warning\"><b>Session warnings</b><pre>{}</pre></section>",
                escape_html(&trace.warnings.join("\n"))
            )
        })
        .unwrap_or_default();
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Support Thread {}</title><style>:root{{color-scheme:light;font-family:Inter,ui-sans-serif,system-ui,sans-serif;color:#172033;background:#eef2f7}}*{{box-sizing:border-box}}body{{margin:0}}main{{max-width:1120px;margin:auto;padding:36px 20px 80px}}h1{{margin:0 0 6px;font-size:30px}}h2{{margin:30px 0 14px}}h3{{margin:9px 0 0;font-size:15px}}small,time{{color:#667085;font-weight:400}}.subtitle{{color:#667085;margin-bottom:24px}}section,.event{{background:#fff;border:1px solid #dce3ec;border-radius:14px;box-shadow:0 2px 8px #1822300a}}.summary{{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:1px;overflow:hidden;background:#dce3ec}}.summary div{{padding:16px;background:#fff}}.summary b{{display:block;color:#667085;font-size:12px;margin-bottom:5px}}code{{font-family:ui-monospace,SFMono-Regular,Menlo,monospace}}details{{margin-top:16px}}details pre{{max-height:360px;overflow:auto}}.event{{padding:15px 17px;margin:0 0 12px;border-left:5px solid #98a2b3}}.event.mattermost{{border-left-color:#3b82f6}}.event.agent{{border-left-color:#8b5cf6}}.event.tool{{border-left-color:#f59e0b}}.event.reasoning{{border-left-color:#64748b}}.event.system{{border-left-color:#94a3b8}}header{{display:flex;align-items:center;justify-content:space-between;gap:12px;font-size:12px}}.badge{{background:#f2f4f7;border-radius:999px;padding:4px 8px;text-transform:capitalize}}pre{{font:13px/1.55 ui-monospace,SFMono-Regular,Menlo,monospace;white-space:pre-wrap;overflow-wrap:anywhere;margin:10px 0 0}}.warning{{margin-top:16px;padding:14px;border-color:#f5c76b;background:#fffbeb}}@media(max-width:600px){{main{{padding:22px 12px}}header{{align-items:flex-start;flex-direction:column;gap:5px}}}}</style></head><body><main><h1>Support debug report</h1><div class=\"subtitle\">Mattermost conversation and Qwen session trace in one timeline</div><section class=\"summary\"><div><b>support_status</b><code>{}</code></div><div><b>Thread</b><code>{}</code></div><div><b>Channel</b><code>{}</code></div><div><b>Root post</b><code>{}</code></div><div><b>Messages</b><code>{}</code></div><div><b>Agent events</b><code>{}</code></div></section><details><summary>Runtime state</summary><pre>{}</pre></details>{}<h2>Timeline ({})</h2>{}</main></body></html>",
        escape_html(thread_id),
        escape_html(&summary.support_status),
        escape_html(thread_id),
        escape_html(channel_id),
        escape_html(root_post_id),
        post_count,
        trace_count,
        escape_html(&summary.state_json),
        warnings,
        post_count + trace_count,
        timeline
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
