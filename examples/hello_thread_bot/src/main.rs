//! Hello-world example for thread-bot Layer 3.
//!
//! Usage:
//!   MM_BASE_PATH=http://localhost:8065 MM_BEARER_TOKEN=xxx cargo run --example hello_thread_bot
//!
//! This bot:
//! - Tracks threads in channels where it's added
//! - Replies with "Processed" to the root post
//! - Stores example lifecycle metadata on finished/stale threads

use std::{ops::Sub, sync::Arc};

use async_trait::async_trait;
use mattermost_api::{apis::configuration::Configuration, models::CreatePostRequest};
use mattermost_bot::{Bot, tokio_graceful};
use thread_bot::{
    ThreadBotHandle, ThreadEffect, ThreadHandler, ThreadMetadataTarget, ThreadStore, ThreadTarget,
    cron_tab,
};

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

    fn thread_kind(&self, _thread: &thread_bot::Thread) -> Option<String> {
        Some("hello_world".to_string())
    }

    async fn handle(
        &self,
        invocation: &thread_bot::ThreadInvocation,
        ctx: &thread_bot::ThreadContext,
    ) -> Result<Vec<ThreadEffect>, thread_bot::ThreadBotError> {
        let thread = ctx
            .build_thread_snapshot(&invocation.thread.thread_id)
            .await?;
        tracing::info!(
            thread_id = %thread.info.thread_id,
            message_count = thread.messages.len(),
            "Processing thread"
        );

        let root_id = thread.info.root_post_id.clone();

        let mut effects = Vec::new();

        if thread.messages.len() < 3 {
            effects.push(ThreadEffect::Reply {
                target: ThreadTarget::CurrentThread,
                message: "Processed".to_string(),
                metadata: serde_json::json!({
                    "handler": self.id(),
                    "root_post_id": root_id
                }),
            });
        } else {
            effects.push(ThreadEffect::Reply {
                target: ThreadTarget::CurrentThread,
                message: "Finished".to_string(),
                metadata: serde_json::json!({
                    "handler": self.id(),
                    "root_post_id": root_id
                }),
            });
            effects.push(ThreadEffect::SetThreadMetadata {
                target: ThreadMetadataTarget::CurrentThread,
                metadata: serde_json::json!({
                    "hello_thread_bot": {
                        "status": "finished"
                    }
                }),
            });

            mattermost_api::apis::reactions_api::save_reaction(
                &ctx.config,
                mattermost_api::models::Reaction {
                    user_id: ctx.bot_user_id.clone(),
                    post_id: Some(root_id.clone()),
                    emoji_name: Some("white_check_mark".to_string()),
                    create_at: None,
                },
            )
            .await
            .ok();
        }

        Ok(effects)
    }

    fn setup_cron(
        self: Arc<Self>,
        scheduler: &mut cron_tab::Cron<chrono::Utc>,
        handle: ThreadBotHandle,
    ) {
        let runtime = tokio::runtime::Handle::current();
        scheduler
            .add_fn("0 * * * * * *", move || {
                runtime.block_on(close_stale_threads(handle.clone()));
            })
            .unwrap();
    }
}

async fn close_stale_threads(handle: ThreadBotHandle) {
    let Ok(tracked_threads) = handle
        .list_threads(None, None)
        .await
        .inspect_err(|e| tracing::error!("unable to list tracked threads: {e:?}"))
    else {
        return;
    };

    let now = chrono::Utc::now();
    for thread in tracked_threads {
        if thread
            .metadata
            .get("hello_thread_bot")
            .and_then(|value| value.get("status"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|status| status != "active")
        {
            continue;
        }

        if let Some(ts) = &thread.last_processed_post_at
            && now.sub(ts).num_minutes() > 1
        {
            handle
                .set_thread_metadata(
                    &thread.thread_id,
                    serde_json::json!({
                        "hello_thread_bot": {
                            "status": "stale"
                        }
                    }),
                )
                .await
                .inspect_err(|e| {
                    tracing::error!(
                        thread_id = thread.thread_id,
                        "unable to mark thread stale: {e:?}"
                    );
                })
                .ok();

            mattermost_api::apis::posts_api::create_post(
                handle.config(),
                CreatePostRequest {
                    channel_id: thread.channel_id.clone(),
                    message: "Thread is closed by timeout".to_string(),
                    root_id: Some(thread.root_post_id.clone()),
                    ..Default::default()
                },
                None,
            )
            .await
            .inspect_err(|e| {
                tracing::error!("unable to send resolution post: {e:?}");
            })
            .ok();
        }
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
