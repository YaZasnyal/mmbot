use crate::config::EngineerNotificationTarget;
use crate::error::{Result, SupportBotError};
use crate::state::EngineerThreadRef;
use mattermost_api::apis::posts_api;
use mattermost_api::{apis::configuration::Configuration, models};
use std::sync::Arc;
use thread_bot::Thread;

pub struct MattermostSupportNotifier {
    config: Arc<Configuration>,
    target: EngineerNotificationTarget,
}

impl MattermostSupportNotifier {
    pub fn new(config: Arc<Configuration>, target: EngineerNotificationTarget) -> Self {
        Self { config, target }
    }

    pub async fn ensure_engineer_thread(
        &self,
        thread: &Thread,
        current_thread: Option<&EngineerThreadRef>,
    ) -> Result<Option<EngineerThreadRef>> {
        match &self.target {
            EngineerNotificationTarget::SameThread => Ok(None),
            EngineerNotificationTarget::MattermostChannel { channel_id } => {
                if let Some(current_thread) = current_thread {
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

                Ok(Some(EngineerThreadRef {
                    channel_id: channel_id.clone(),
                    root_post_id: created.id,
                }))
            }
        }
    }

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
                let request = engineer_thread_reply_request(
                    channel_id,
                    &thread_ref.root_post_id,
                    thread,
                    message,
                );

                posts_api::create_post(&self.config, request, None)
                    .await
                    .map_err(|error| SupportBotError::Mattermost(error.to_string()))?;

                Ok(Some(thread_ref))
            }
        }
    }

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

    pub async fn post_html_attachment_to_thread(
        &self,
        channel_id: &str,
        root_post_id: &str,
        filename: &str,
        html_content: String,
        message: String,
        props: serde_json::Value,
    ) -> Result<()> {
        let file_id = self
            .upload_html_file(channel_id, filename, html_content.into_bytes())
            .await?;

        let mut request = models::CreatePostRequest::new(channel_id.to_string(), message);
        request.root_id = Some(root_post_id.to_string());
        request.file_ids = Some(vec![file_id]);
        request.props = Some(props);

        posts_api::create_post(&self.config, request, None)
            .await
            .map_err(|error| SupportBotError::Mattermost(error.to_string()))?;
        Ok(())
    }

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
    posts: &[SupportReportPost],
    trace_post_id: Option<&str>,
    tool_trace: &[String],
) -> String {
    let items = posts
        .iter()
        .map(|post| {
            let trace_block = if trace_post_id.is_some_and(|id| id == post.post_id) && !tool_trace.is_empty() {
                let details = tool_trace
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
         pre{{white-space:pre-wrap;word-wrap:break-word;margin:0;font-family:inherit}}</style></head><body><div class=\"wrap\">\
         <h1>Support Thread Report</h1><div class=\"meta\"><div><b>thread_id:</b> <code>{thread_id}</code></div>\
         <div><b>channel_id:</b> <code>{channel_id}</code></div><div><b>root_post_id:</b> <code>{root_post_id}</code></div>\
         <div><b>messages:</b> <code>{count}</code></div><div><b>trace_entries:</b> <code>{trace_count}</code></div>\
         <div><b>trace_bound_post_id:</b> <code>{trace_post_id}</code></div></div>\
         <h2>Messages</h2>{items}\
         <script>document.querySelectorAll('pre').forEach((p)=>{{if(p.textContent.length>3000){{p.dataset.full=p.textContent;p.textContent=p.textContent.slice(0,3000)+'\\n...[trimmed in view]';}}}});</script>\
         </div></body></html>",
        thread_id = escape_html(thread_id),
        channel_id = escape_html(channel_id),
        root_post_id = escape_html(root_post_id),
        count = posts.len(),
        items = items,
        trace_count = tool_trace.len(),
        trace_post_id = escape_html(trace_post_id.unwrap_or("n/a"))
    )
}

#[derive(Debug, Clone)]
pub struct SupportReportPost {
    pub post_id: String,
    pub user_id: String,
    pub message: String,
    pub created_at: String,
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

fn source_post_link(config: &Configuration, post_id: &str) -> String {
    format!(
        "{}/_redirect/pl/{}",
        config.base_path.trim_end_matches('/'),
        post_id
    )
}

fn quote_for_mattermost(message: &str) -> String {
    message
        .lines()
        .map(|line| format!("> {line}"))
        .collect::<Vec<_>>()
        .join("\n")
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
    use thread_bot::{ThreadInfo, ThreadMessage, ThreadStatus};

    fn thread() -> Thread {
        Thread {
            info: ThreadInfo {
                thread_id: "thread-1".to_string(),
                root_post_id: "root-1".to_string(),
                channel_id: "users".to_string(),
                creator_user_id: "user-1".to_string(),
                status: ThreadStatus::Active,
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
        assert!(request.message.contains("> service is down"));
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
}
