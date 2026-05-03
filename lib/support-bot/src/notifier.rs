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

    pub async fn send_debug_message(
        &self,
        thread: &Thread,
        current_thread: Option<&EngineerThreadRef>,
        message: String,
    ) -> Result<Option<EngineerThreadRef>> {
        self.post_engineer_thread_message(thread, current_thread, "debug", message)
            .await
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
