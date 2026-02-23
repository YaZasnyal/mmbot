mod common;

use anyhow::Result;
use common::{run_bot_background, wait_for_event, MattermostTestEnv};
use mattermost_bot::types::EventType;
use mattermost_bot::Bot;
use test_log::test;

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
