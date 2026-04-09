use serde::Deserialize;
use std::sync::Arc;

use mattermost_api::apis::configuration::Configuration;

use crate::types::EventType;

#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    #[serde(flatten)]
    pub data: EventType,
}

#[async_trait::async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Unique ID that is used to store plugin progress
    fn id(&self) -> &'static str;

    /// Called once when bot starts (before connecting to WebSocket)
    ///
    /// Use this to initialize resources, open database connections,
    /// restore state from persistent storage, etc.
    ///
    /// If this method returns an error, it will be logged but won't prevent
    /// the bot from starting. Other plugins will continue to initialize.
    async fn on_start(&self, _config: &Arc<Configuration>) -> crate::Result<()> {
        Ok(())
    }

    /// Called when bot is shutting down (after WebSocket closed)
    ///
    /// Use this to clean up resources, close connections, save state, etc.
    ///
    /// Errors are logged but don't prevent shutdown from completing.
    async fn on_shutdown(&self, _config: &Arc<Configuration>) -> crate::Result<()> {
        Ok(())
    }

    /// Fast event filter
    fn filter(&self, _event: &Arc<Event>) -> bool {
        true
    }

    /// Process event
    ///
    /// Launched asynchronously
    async fn process_event(&self, event: &Arc<Event>, config: &Arc<Configuration>);

    /// Setup cron jobs for this plugin
    ///
    /// This method is called once during bot initialization.
    /// Plugin should register its cron jobs using the provided scheduler.
    ///
    /// # Example
    ///
    /// ```ignore
    ///     self: Arc<Self>,
    ///     scheduler: &mut cron_tab::Cron<chrono::Utc>,
    ///     config: Arc<Configuration>,
    /// ) {
    ///     let plugin = Arc::clone(&self);
    ///     let config = Arc::clone(&config);
    ///
    ///     scheduler.add_fn("0 0 * * * *", move || {
    ///         let plugin = Arc::clone(&plugin);
    ///         let config = Arc::clone(&config);
    ///
    ///         tokio::runtime::Handle::current().block_on(async move {
    ///             // Access plugin data here
    ///             println!("Running cron for {}", plugin.id());
    ///         });
    ///     }).unwrap();
    /// }
    /// ```
    fn setup_cron(
        self: Arc<Self>,
        _scheduler: &mut cron_tab::Cron<chrono::Utc>,
        _config: Arc<Configuration>,
    ) {
        // Default: no cron jobs
    }
}
