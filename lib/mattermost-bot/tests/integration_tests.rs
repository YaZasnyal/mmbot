mod common;

use anyhow::Result;
use common::MattermostTestEnv;
use mattermost_bot::types::EventType;
use mattermost_bot::{async_trait, Bot, Event, Plugin};
use std::sync::Arc;
use std::time::Duration;
use test_log::test;
use tokio::sync::mpsc;

/// Test plugin that sends events to a channel
struct EventChannelPlugin {
    tx: mpsc::UnboundedSender<Arc<Event>>,
}

#[async_trait]
impl Plugin for EventChannelPlugin {
    fn id(&self) -> &'static str {
        "EventChannel"
    }

    fn filter(&self, _event: &Arc<Event>) -> bool {
        true // Forward all events
    }

    async fn process_event(
        &self,
        event: &Arc<Event>,
        _config: &Arc<mattermost_api::apis::configuration::Configuration>,
    ) {
        tracing::info!(event_type=?event.data, "Event received");
        let _ = self.tx.send(Arc::clone(event));
    }
}

impl EventChannelPlugin {
    fn new() -> (Self, mpsc::UnboundedReceiver<Arc<Event>>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { tx }, rx)
    }
}

/// Wait for a specific event type from the channel
async fn wait_for_event<F>(
    rx: &mut mpsc::UnboundedReceiver<Arc<Event>>,
    matcher: F,
) -> Option<Arc<Event>>
where
    F: Fn(&EventType) -> bool,
{
    while let Some(event) = rx.recv().await {
        tracing::info!("received event: {:?}", event.data);
        if matcher(&event.data) {
            tracing::info!("found awaited event");
            return Some(event);
        }
    }
    None
}

/// Run bot in background with automatic shutdown
/// Returns the event receiver and a shutdown handle that will stop the bot when dropped
fn run_bot_background(
    bot: Bot,
) -> (
    mpsc::UnboundedReceiver<Arc<Event>>,
    tokio_graceful::Shutdown,
) {
    let (plugin, events_rx) = EventChannelPlugin::new();
    let bot = bot.with_plugin(plugin);

    // Setup shutdown with timeout (will shutdown after 30 seconds if not stopped manually)
    let shutdown = tokio_graceful::Shutdown::new(async {
        tokio::time::sleep(Duration::from_secs(5)).await;
    });
    let guard = shutdown.guard();

    // Run bot in background
    tokio::spawn(async move {
        let mut bot = bot;
        bot.run(guard).await;
    });

    (events_rx, shutdown)
}

#[test(tokio::test)]
async fn test_bot_receives_posted_event() -> Result<()> {
    // Create test environment (waits for Mattermost, creates all resources)
    let env = MattermostTestEnv::new().await?;

    // Run bot in background
    let bot = Bot::with_config((*env.bot_config()).clone())?;
    let (mut events_rx, _shutdown) = run_bot_background(bot);

    // Wait for Hello event (Mattermost sends this on successful connection)
    let hello = wait_for_event(&mut events_rx, |e| matches!(e, EventType::Hello(_))).await;
    if let Some(event) = &hello {
        if let EventType::Hello(h) = &event.data {
            tracing::info!(
                "Bot connected! Connection ID: {}, Server: {}",
                h.connection_id,
                h.server_version
            );
        }
    }

    // Send a message to the channel using admin client
    tracing::info!("Sending test message...");
    let admin_client = env.http_client(None, None);
    admin_client
        .post_message(&env.channel_id, "Test message for bot")
        .await?;

    // Wait for Posted event
    let posted_event = wait_for_event(&mut events_rx, |e| matches!(e, EventType::Posted(_))).await;

    assert!(
        posted_event.is_some(),
        "Bot should have received a Posted event"
    );

    tracing::info!("Test completed successfully!");
    Ok(())
}

#[test(tokio::test)]
async fn test_bot_receives_reaction_added_event() -> Result<()> {
    // Create test environment
    let env = MattermostTestEnv::new().await?;

    // Run bot in background
    let bot = Bot::with_config((*env.bot_config()).clone())?;
    let (mut events_rx, _shutdown) = run_bot_background(bot);

    // Wait for Hello event (Mattermost sends this on successful connection)
    let hello = wait_for_event(&mut events_rx, |e| matches!(e, EventType::Hello(_))).await;
    if let Some(event) = &hello {
        if let EventType::Hello(h) = &event.data {
            tracing::info!(
                "Bot connected! Connection ID: {}, Server: {}",
                h.connection_id,
                h.server_version
            );
        }
    }

    // Send a message to the channel
    tracing::info!("Sending test message...");
    let admin_client = env.http_client(None, None);
    let post = admin_client
        .post_message(&env.channel_id, "Test message for reaction")
        .await?;
    let post_id = post["id"].as_str().unwrap().to_string();
    tracing::info!("Message sent with post_id: {}", post_id);

    // Add a reaction to the post
    tracing::info!("Adding reaction to post...");
    admin_client.add_reaction(&post_id, "smiley").await?;
    tracing::info!("Reaction added successfully");

    // Wait for ReactionAdded event
    let reaction_event =
        wait_for_event(&mut events_rx, |e| matches!(e, EventType::ReactionAdded(_))).await;

    assert!(
        reaction_event.is_some(),
        "Bot should have received a ReactionAdded event"
    );

    tracing::info!("Test completed successfully!");
    Ok(())
}
