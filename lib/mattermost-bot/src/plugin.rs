use serde::Deserialize;
use std::sync::Arc;

use mattermost_api::apis::configuration::Configuration;

use crate::types::EventType;

#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    #[serde(flatten)]
    data: EventType,
}

#[async_trait::async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Fast event filter
    fn filter(&self, _event: Arc<Event>) -> bool {
        true
    }

    /// Indicated the unique queue that must be used to post events to
    ///
    /// Allows to make processing serializable if long running events are used
    fn queue(&self, _event: Arc<Event>) -> Option<String> {
        None
    }

    /// Function that is called when bot is instatiated
    async fn on_setup(&self) {}

    /// Function that is called when bot is terminating
    async fn on_destroy(&self) {}

    /// Process event
    ///
    /// Launched asynchronously
    async fn process_event(&self, event: Arc<Event>, config: Arc<Configuration>);
}
