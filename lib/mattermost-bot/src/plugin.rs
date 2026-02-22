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
    /// ```no_run
    /// fn setup_cron(
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
