use std::sync::Arc;
use serde::Deserialize;

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
    fn filter(&self, event: Arc<Event>) -> bool {
        true
    }

    /// Indicated the unique queue that must be used to post events to
    ///
    /// Allows to make processing serializable if long running events are used
    fn queue(&self, event: Arc<Event>) -> Option<String> {
        None
    }

    /// Process event
    ///
    /// Launched asynchronously
    async fn process_event(&self, event: Arc<Event>, config: Arc<Configuration>);
}
