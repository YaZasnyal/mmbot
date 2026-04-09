use mattermost_bot::types::EventType;
use mattermost_bot::{async_trait, Bot, Event, Plugin};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

// Re-export shared test helpers
pub use mattermost_test_helpers::{AuthenticatedClient, MattermostTestEnv};

/// Test plugin that forwards all events to a channel
pub struct EventChannelPlugin {
    tx: mpsc::UnboundedSender<Arc<Event>>,
}

impl EventChannelPlugin {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Arc<Event>>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { tx }, rx)
    }
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

/// Wait for a specific event type from the channel
pub async fn wait_for_event<F>(
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
///
/// Returns the event receiver and a shutdown handle.
/// The bot will automatically stop after 5 seconds or when shutdown.shutdown() is called.
pub fn run_bot_background(
    bot: Bot,
) -> (
    mpsc::UnboundedReceiver<Arc<Event>>,
    tokio::sync::oneshot::Sender<()>,
) {
    let (plugin, events_rx) = EventChannelPlugin::new();
    let bot = bot.with_plugin(plugin);

    // Setup shutdown with timeout (will shutdown after 5 seconds if not stopped manually)
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let shutdown = tokio_graceful::Shutdown::builder()
        .with_signal(async move {
            rx.await.ok();
        })
        .build();
    let guard = shutdown.guard();

    // Run bot in background
    tokio::spawn(async move {
        let mut bot = bot;
        let fut = bot.run(guard);
        let timeout = tokio::time::sleep(Duration::from_secs(5));
        tokio::select! {
            _ = timeout => {
                tracing::error!("bot reached test run timeout")
            },
            _ = fut => {
                tracing::info!("bot finished")
            }
        }
    });

    (events_rx, tx)
}
