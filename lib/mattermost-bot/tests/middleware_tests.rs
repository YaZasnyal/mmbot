mod common;

use anyhow::Result;
use common::{run_bot_background, wait_for_event, MattermostTestEnv};
use mattermost_bot::middlewares::IgnoreSelf;
use mattermost_bot::types::EventType;
use mattermost_bot::{async_trait, Bot, Event, Plugin};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use test_log::test;

/// Test plugin that counts all Posted events
struct CounterPlugin {
    count: Arc<AtomicUsize>,
}

#[async_trait]
impl Plugin for CounterPlugin {
    fn id(&self) -> &'static str {
        "Counter"
    }

    fn filter(&self, event: &Arc<Event>) -> bool {
        matches!(event.data, EventType::Posted(_))
    }

    async fn process_event(
        &self,
        _event: &Arc<Event>,
        _config: &Arc<mattermost_api::apis::configuration::Configuration>,
    ) {
        self.count.fetch_add(1, Ordering::SeqCst);
        tracing::info!(count = self.count.load(Ordering::SeqCst), "Event counted");
    }
}

#[test(tokio::test)]
async fn test_ignore_self_middleware() -> Result<()> {
    let env = MattermostTestEnv::new().await?;

    // Create counter plugin wrapped with IgnoreSelf middleware
    let counter = Arc::new(AtomicUsize::new(0));
    let plugin = CounterPlugin {
        count: Arc::clone(&counter),
    };
    let plugin = IgnoreSelf::new(plugin);

    let bot = Bot::with_config((*env.bot_config()).clone())?.with_plugin(plugin);

    let (mut events_rx, _shutdown) = run_bot_background(bot);

    // Wait for Hello event
    let hello = wait_for_event(&mut events_rx, |e| matches!(e, EventType::Hello(_))).await;
    assert!(hello.is_some(), "Should receive Hello event");

    tracing::info!("Bot connected, sending messages...");

    // Get admin and bot clients
    let admin_client = env.http_client(None, None);
    let bot_client = env.http_client(Some(env.bot_token()), Some(env.bot_user_id()));

    // Send message from admin - should be counted
    admin_client
        .post_message(&env.channel_id, "Message from admin")
        .await?;

    // Wait for the Posted event
    wait_for_event(&mut events_rx, |e| matches!(e, EventType::Posted(_))).await;

    // Give plugin time to process
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Should have counted 1 message (admin's message, not the "bot added" system message)
    let count_after_admin = counter.load(Ordering::SeqCst);
    tracing::info!(count = count_after_admin, "Count after admin message");
    assert!(
        count_after_admin >= 1,
        "Should count at least admin's message"
    );

    // Send message from bot itself - should be filtered out
    bot_client
        .post_message(&env.channel_id, "Message from bot itself")
        .await?;

    // Wait for the Posted event to arrive
    wait_for_event(&mut events_rx, |e| matches!(e, EventType::Posted(_))).await;

    // Give plugin time to process (it shouldn't process this one)
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Count should not have increased
    let count_after_bot = counter.load(Ordering::SeqCst);
    tracing::info!(
        count_before = count_after_admin,
        count_after = count_after_bot,
        "Count after bot message"
    );
    assert_eq!(
        count_after_admin, count_after_bot,
        "Bot's own message should be filtered out"
    );

    tracing::info!("Test completed successfully!");
    Ok(())
}
