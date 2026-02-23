mod common;

use anyhow::Result;
use common::{run_bot_background, wait_for_event, MattermostTestEnv};
use mattermost_bot::types::EventType;
use mattermost_bot::{async_trait, Bot, Event, Plugin};
use std::sync::Arc;
use test_log::test;
use tokio::sync::oneshot;

/// Test plugin that signals lifecycle hooks via oneshot channels
struct LifecycleTestPlugin {
    on_start_tx: std::sync::Mutex<Option<oneshot::Sender<()>>>,
    on_shutdown_tx: std::sync::Mutex<Option<oneshot::Sender<()>>>,
}

impl LifecycleTestPlugin {
    fn new() -> (Self, oneshot::Receiver<()>, oneshot::Receiver<()>) {
        let (start_tx, start_rx) = oneshot::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let plugin = Self {
            on_start_tx: std::sync::Mutex::new(Some(start_tx)),
            on_shutdown_tx: std::sync::Mutex::new(Some(shutdown_tx)),
        };

        (plugin, start_rx, shutdown_rx)
    }
}

#[async_trait]
impl Plugin for LifecycleTestPlugin {
    fn id(&self) -> &'static str {
        "LifecycleTest"
    }

    async fn on_start(
        &self,
        _config: &Arc<mattermost_api::apis::configuration::Configuration>,
    ) -> mattermost_bot::Result<()> {
        tracing::info!("LifecycleTestPlugin::on_start called");
        if let Some(tx) = self.on_start_tx.lock().unwrap().take() {
            let _ = tx.send(());
        }
        Ok(())
    }

    async fn on_shutdown(
        &self,
        _config: &Arc<mattermost_api::apis::configuration::Configuration>,
    ) -> mattermost_bot::Result<()> {
        tracing::info!("LifecycleTestPlugin::on_shutdown called");
        if let Some(tx) = self.on_shutdown_tx.lock().unwrap().take() {
            let _ = tx.send(());
        }
        Ok(())
    }

    async fn process_event(
        &self,
        _event: &Arc<Event>,
        _config: &Arc<mattermost_api::apis::configuration::Configuration>,
    ) {
        // No-op
    }
}

#[test(tokio::test)]
async fn test_plugin_lifecycle_hooks() -> Result<()> {
    let env = MattermostTestEnv::new().await?;

    let (plugin, mut on_start_rx, mut on_shutdown_rx) = LifecycleTestPlugin::new();

    let bot = Bot::with_config((*env.bot_config()).clone())?.with_plugin(plugin);

    let (mut events_rx, bot_shutdown) = run_bot_background(bot);

    // Wait for Hello event - this means the bot has connected and on_start must have been called
    let hello = wait_for_event(&mut events_rx, |e| matches!(e, EventType::Hello(_))).await;
    assert!(hello.is_some(), "Should receive Hello event");

    if let Some(event) = &hello {
        if let EventType::Hello(h) = &event.data {
            tracing::info!(
                "Bot connected! Connection ID: {}, Server: {}",
                h.connection_id,
                h.server_version
            );
        }
    }

    // Check that on_start was called
    assert!(
        on_start_rx.try_recv().is_ok(),
        "on_start should be called before Hello event"
    );

    // Check that on_shutdown was NOT called yet
    assert!(
        on_shutdown_rx.try_recv().is_err(),
        "on_shutdown should not be called yet"
    );

    tracing::info!("on_start verified, triggering shutdown...");

    // Trigger shutdown
    bot_shutdown.shutdown().await;

    // Wait for on_shutdown to be called
    tokio::select! {
        _ = &mut on_shutdown_rx => {
            tracing::info!("on_shutdown was called");
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
            panic!("on_shutdown was not called within 2 seconds");
        }
    }

    tracing::info!("Test completed successfully!");
    Ok(())
}
