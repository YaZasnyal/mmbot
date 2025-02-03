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
}
