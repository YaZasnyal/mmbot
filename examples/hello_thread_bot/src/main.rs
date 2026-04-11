//! Hello-world example for thread-bot Layer 3.
//!
//! Usage:
//!   MM_BASE_PATH=http://localhost:8065 MM_BEARER_TOKEN=xxx cargo run --example hello_thread_bot
//!
//! This bot:
//! - Tracks threads in channels where it's added
//! - Replies with "Processed" to the root post
//! - Marks the thread as resolved

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mattermost_api::apis::configuration::Configuration;
use mattermost_bot::{tokio_graceful, Bot};
use thread_bot::{
    types::{
        AppendReaction, ThreadMessageRecord, ThreadReaction, ThreadRecord, ThreadStatus,
        UpsertThread, UpsertThreadMessage,
    },
    ThreadEffect, ThreadHandler, ThreadStore,
};
use tokio::sync::RwLock;

struct HelloWorldHandler;

#[async_trait]
impl ThreadHandler for HelloWorldHandler {
    fn id(&self) -> &'static str {
        "hello_world_handler"
    }

    async fn should_track(
        &self,
        _thread: &thread_bot::Thread,
        _ctx: &thread_bot::ThreadContext,
    ) -> Result<bool, thread_bot::ThreadBotError> {
        Ok(true)
    }

    async fn handle(
        &self,
        thread: &thread_bot::Thread,
        _ctx: &thread_bot::ThreadContext,
    ) -> Result<Vec<ThreadEffect>, thread_bot::ThreadBotError> {
        tracing::info!(
            thread_id = %thread.info.thread_id,
            message_count = thread.messages.len(),
            "Processing thread"
        );

        let root_id = thread.info.root_post_id.clone();

        let mut effects = Vec::new();

        if !thread.messages.is_empty() {
            effects.push(ThreadEffect::Reply {
                message: "Processed".to_string(),
                metadata: serde_json::json!({
                    "handler": self.id(),
                    "root_post_id": root_id
                }),
            });
        }

        effects.push(ThreadEffect::MarkResolved);

        Ok(effects)
    }
}

struct InMemoryStore {
    threads: RwLock<HashMap<String, ThreadRecord>>,
    messages: RwLock<HashMap<String, ThreadMessageRecord>>,
}

impl InMemoryStore {
    fn new() -> Self {
        Self {
            threads: RwLock::new(HashMap::new()),
            messages: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ThreadStore for InMemoryStore {
    async fn upsert_thread(
        &self,
        input: UpsertThread,
    ) -> Result<ThreadRecord, thread_bot::ThreadBotError> {
        let now = Utc::now();
        let record = ThreadRecord {
            thread_id: input.thread_id.clone(),
            root_post_id: input.root_post_id,
            channel_id: input.channel_id,
            creator_user_id: input.creator_user_id,
            status: input.status,
            metadata: input.metadata,
            last_seen_post_id: None,
            last_seen_post_at: None,
            last_processed_post_id: None,
            last_processed_post_at: None,
            created_at: now,
            updated_at: now,
        };
        self.threads
            .write()
            .await
            .insert(input.thread_id, record.clone());
        Ok(record)
    }

    async fn get_thread(
        &self,
        thread_id: &str,
    ) -> Result<Option<ThreadRecord>, thread_bot::ThreadBotError> {
        Ok(self.threads.read().await.get(thread_id).cloned())
    }

    async fn get_thread_by_post(
        &self,
        post_id: &str,
    ) -> Result<Option<ThreadRecord>, thread_bot::ThreadBotError> {
        let threads = self.threads.read().await;
        if let Some(t) = threads.get(post_id) {
            return Ok(Some(t.clone()));
        }
        for t in threads.values() {
            if t.root_post_id == post_id {
                return Ok(Some(t.clone()));
            }
        }
        Ok(None)
    }

    async fn list_threads_by_status(
        &self,
        statuses: &[ThreadStatus],
    ) -> Result<Vec<ThreadRecord>, thread_bot::ThreadBotError> {
        let threads = self.threads.read().await;
        Ok(threads
            .values()
            .filter(|t| statuses.contains(&t.status))
            .cloned()
            .collect())
    }

    async fn update_thread_status(
        &self,
        thread_id: &str,
        status: ThreadStatus,
    ) -> Result<(), thread_bot::ThreadBotError> {
        let mut threads = self.threads.write().await;
        if let Some(t) = threads.get_mut(thread_id) {
            t.status = status;
            t.updated_at = Utc::now();
            Ok(())
        } else {
            Err(thread_bot::ThreadBotError::ThreadNotFound(
                thread_id.to_string(),
            ))
        }
    }

    async fn set_thread_metadata(
        &self,
        thread_id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), thread_bot::ThreadBotError> {
        let mut threads = self.threads.write().await;
        if let Some(t) = threads.get_mut(thread_id) {
            t.metadata = metadata;
            t.updated_at = Utc::now();
            Ok(())
        } else {
            Err(thread_bot::ThreadBotError::ThreadNotFound(
                thread_id.to_string(),
            ))
        }
    }

    async fn update_thread_seen(
        &self,
        thread_id: &str,
        post_id: &str,
        seen_at: DateTime<Utc>,
    ) -> Result<(), thread_bot::ThreadBotError> {
        let mut threads = self.threads.write().await;
        if let Some(t) = threads.get_mut(thread_id) {
            t.last_seen_post_id = Some(post_id.to_string());
            t.last_seen_post_at = Some(seen_at);
            t.updated_at = Utc::now();
            Ok(())
        } else {
            Err(thread_bot::ThreadBotError::ThreadNotFound(
                thread_id.to_string(),
            ))
        }
    }

    async fn update_thread_processed(
        &self,
        thread_id: &str,
        post_id: &str,
        processed_at: DateTime<Utc>,
    ) -> Result<(), thread_bot::ThreadBotError> {
        let mut threads = self.threads.write().await;
        if let Some(t) = threads.get_mut(thread_id) {
            t.last_processed_post_id = Some(post_id.to_string());
            t.last_processed_post_at = Some(processed_at);
            t.updated_at = Utc::now();
            Ok(())
        } else {
            Err(thread_bot::ThreadBotError::ThreadNotFound(
                thread_id.to_string(),
            ))
        }
    }

    async fn upsert_message(
        &self,
        input: UpsertThreadMessage,
    ) -> Result<ThreadMessageRecord, thread_bot::ThreadBotError> {
        let now = Utc::now();
        let record = ThreadMessageRecord {
            post_id: input.post_id.clone(),
            thread_id: input.thread_id.clone(),
            user_id: input.user_id,
            is_bot_message: input.is_bot_message,
            root_id: input.root_id,
            parent_post_id: input.parent_post_id,
            metadata: input.metadata,
            post_created_at: input.post_created_at,
            post_updated_at: input.post_updated_at,
            post_deleted_at: input.post_deleted_at,
            created_at: now,
            updated_at: now,
        };
        self.messages
            .write()
            .await
            .insert(input.post_id, record.clone());
        Ok(record)
    }

    async fn get_message(
        &self,
        post_id: &str,
    ) -> Result<Option<ThreadMessageRecord>, thread_bot::ThreadBotError> {
        Ok(self.messages.read().await.get(post_id).cloned())
    }

    async fn list_thread_messages(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadMessageRecord>, thread_bot::ThreadBotError> {
        Ok(self
            .messages
            .read()
            .await
            .values()
            .filter(|m| m.thread_id == thread_id)
            .cloned()
            .collect())
    }

    async fn set_message_metadata(
        &self,
        post_id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), thread_bot::ThreadBotError> {
        let mut messages = self.messages.write().await;
        if let Some(m) = messages.get_mut(post_id) {
            m.metadata = metadata;
            m.updated_at = Utc::now();
            Ok(())
        } else {
            Err(thread_bot::ThreadBotError::MessageNotFound(
                post_id.to_string(),
            ))
        }
    }

    async fn append_reaction(
        &self,
        _input: AppendReaction,
    ) -> Result<(), thread_bot::ThreadBotError> {
        Ok(())
    }

    async fn list_thread_reactions(
        &self,
        _thread_id: &str,
    ) -> Result<Vec<ThreadReaction>, thread_bot::ThreadBotError> {
        Ok(Vec::new())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = Configuration {
        base_path: std::env::var("MM_BASE_PATH")
            .unwrap_or_else(|_| "http://localhost:8065".to_string()),
        bearer_access_token: Some(
            std::env::var("MM_BEARER_TOKEN").expect("Set MM_BEARER_TOKEN env var"),
        ),
        ..Default::default()
    };

    let store: Arc<dyn ThreadStore> = Arc::new(InMemoryStore::new());
    let plugin = thread_bot::ThreadBotPlugin::new(HelloWorldHandler, store);

    let mut bot = Bot::with_config(config)?.with_plugin(plugin);

    tracing::info!("Starting hello-thread-bot...");

    let shutdown = tokio_graceful::Shutdown::builder()
        .with_signal(tokio::signal::ctrl_c())
        .build();
    let guard = shutdown.guard();

    bot.run(guard).await;

    Ok(())
}
