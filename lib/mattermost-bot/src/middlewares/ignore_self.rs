//! Middleware that filters out messages from the bot itself
//!
//! This prevents the bot from responding to its own messages in a loop.
//!
//! # Example
//!
//! ```ignore
//! use mattermost_bot::{Bot, Plugin, middlewares::IgnoreSelf};
//!
//! struct MyPlugin;
//!
//! // Wrap the plugin with IgnoreSelf middleware
//! let plugin = MyPlugin;
//! let plugin = IgnoreSelf::new(plugin);
//!
//! let bot = Bot::with_config(config)?
//!     .with_plugin(plugin);
//! ```

use crate::types::EventType;
use crate::{async_trait, Event, Plugin, Result};
use mattermost_api::apis::configuration::Configuration;
use std::sync::Arc;

/// Middleware that ignores events from the bot itself
///
/// Extracts bot's user ID from the first `Hello` event and filters out
/// all `Posted` events where the author is the bot.
pub struct IgnoreSelf<P> {
    inner: Arc<P>,
    bot_user_id: std::sync::Mutex<Option<String>>,
}

impl<P> IgnoreSelf<P> {
    /// Create a new IgnoreSelf middleware wrapping the given plugin
    pub fn new(plugin: P) -> Self {
        Self {
            inner: Arc::new(plugin),
            bot_user_id: std::sync::Mutex::new(None),
        }
    }

    /// Check if the event is from the bot itself
    fn is_self_message(&self, event: &Event) -> bool {
        let bot_user_id = self.bot_user_id.lock().unwrap();

        if let Some(bot_id) = bot_user_id.as_ref() {
            if let EventType::Posted(posted) = &event.data {
                if let Some(user_id) = &posted.post.inner.user_id {
                    return user_id == bot_id;
                }
            }
        }

        false
    }
}

#[async_trait]
impl<P: Plugin> Plugin for IgnoreSelf<P> {
    fn id(&self) -> &'static str {
        self.inner.id()
    }

    async fn on_start(&self, config: &Arc<Configuration>) -> Result<()> {
        // Fetch bot's own user_id using /users/me endpoint
        if let Ok(user) = mattermost_api::apis::users_api::get_user(config, "me").await {
            if let Some(user_id) = user.id {
                tracing::debug!(user_id = %user_id, "Bot user ID retrieved");
                *self.bot_user_id.lock().unwrap() = Some(user_id);
            }
        }

        self.inner.on_start(config).await
    }

    async fn on_shutdown(&self, config: &Arc<Configuration>) -> Result<()> {
        self.inner.on_shutdown(config).await
    }

    fn filter(&self, event: &Arc<Event>) -> bool {
        // Filter out messages from the bot itself
        if self.is_self_message(event) {
            tracing::debug!("Filtering out self message");
            return false;
        }

        // Delegate to inner plugin's filter
        self.inner.filter(event)
    }

    async fn process_event(&self, event: &Arc<Event>, config: &Arc<Configuration>) {
        self.inner.process_event(event, config).await
    }

    fn setup_cron(
        self: Arc<Self>,
        scheduler: &mut cron_tab::Cron<chrono::Utc>,
        config: Arc<Configuration>,
    ) {
        // Proxy to inner plugin's cron setup
        Arc::clone(&self.inner).setup_cron(scheduler, config);
    }
}
