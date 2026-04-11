//! Hello-world example for thread-bot Layer 3.
//!
//! Usage:
//!   MM_BASE_PATH=http://localhost:8065 MM_BEARER_TOKEN=xxx cargo run --example hello_thread_bot
//!
//! This bot:
//! - Tracks threads in channels where it's added
//! - Replies with "Processed" to the root post
//! - Marks the thread as resolved

use std::sync::Arc;

use async_trait::async_trait;
use mattermost_api::apis::configuration::Configuration;
use mattermost_bot::{tokio_graceful, Bot};
use thread_bot::{ThreadEffect, ThreadHandler, ThreadStore};

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

        mattermost_api::apis::reactions_api::save_reaction(
            &_ctx.config,
            mattermost_api::models::Reaction {
                user_id: None,
                post_id: Some(root_id.clone()),
                emoji_name: Some(":white_check_mark:".to_string()),
                create_at: None,
            },
        )
        .await
        .ok();

        effects.push(ThreadEffect::MarkResolved);

        Ok(effects)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config = Configuration {
        base_path: std::env::var("MM_BASE_PATH")
            .unwrap_or_else(|_| "http://localhost:8065".to_string()),
        bearer_access_token: Some(
            std::env::var("MM_BEARER_TOKEN").expect("Set MM_BEARER_TOKEN env var"),
        ),
        ..Default::default()
    };

    let pool = thread_bot::sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://test:test@localhost:5433/thread_bot_test")
        .await?;
    let store: Arc<dyn ThreadStore> = Arc::new(thread_bot::PgThreadStore::new(pool).await?);
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
