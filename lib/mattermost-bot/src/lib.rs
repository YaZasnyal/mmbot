use std::sync::Arc;

use futures_util::{SinkExt, TryStreamExt};
pub use mattermost_api::apis::configuration::Configuration;
use reqwest_websocket::{Message, RequestBuilderExt};

pub use mattermost_api;
pub mod error;
pub mod nested_decoder;
pub mod plugin;
pub mod types;

pub use error::{BotError, Result};
pub use plugin::{Event, Plugin};

// reexports
pub use async_trait::async_trait;

pub struct Bot {
    config: Arc<Configuration>,

    plugins: Vec<Box<dyn Plugin>>,
}

impl Bot {
    /// Create a new bot with the given configuration
    ///
    /// # Errors
    ///
    /// Returns `BotError::MissingToken` if `bearer_access_token` is not set in the configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mattermost_bot::Bot;
    /// use mattermost_api::apis::configuration::Configuration;
    ///
    /// // Using the builder pattern
    /// let mut config = Configuration::default();
    /// config.base_path = "https://your-mattermost.com".to_string();
    /// config.bearer_access_token = Some("your_token".to_string());
    ///
    /// let bot = Bot::with_config(config)?
    ///   .with_plugin(MyPlugin {})
    /// ```
    pub fn with_config(config: Configuration) -> Result<Self> {
        if config.bearer_access_token.is_none() {
            return Err(BotError::MissingToken);
        }

        Ok(Self {
            config: Arc::new(config),
            plugins: Default::default(),
        })
    }

    /// Add a plugin to the bot
    ///
    /// This method consumes self and returns it back, allowing for fluent chaining.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let bot = Bot::with_config(config)?
    ///     .with_plugin(MyPlugin {})
    ///     .with_plugin(AnotherPlugin {});
    /// ```
    pub fn with_plugin(mut self, plugin: impl Plugin) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    /// Run the bot with graceful shutdown support
    ///
    /// This method will run the bot until a shutdown signal is received.
    /// It automatically handles reconnection on WebSocket errors.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mattermost_bot::Bot;
    /// use mattermost_api::apis::configuration::Configuration;
    /// use tokio_graceful::Shutdown;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     // Setup graceful shutdown with Ctrl+C handler
    ///     let shutdown = Shutdown::builder()
    ///         .with_delay(Duration::from_secs(5))
    ///         .with_overwrite_fn(tokio::signal::ctrl_c)
    ///         .build();
    ///
    ///     let guard = shutdown.guard();
    ///
    ///     // Create and configure bot using fluent API
    ///     let mut config = Configuration::default();
    ///     config.base_path = "https://your-mattermost.com".to_string();
    ///     config.bearer_access_token = Some("your_token".to_string());
    ///
    ///     let mut bot = Bot::with_config(config)?
    ///         .with_plugin(MyPlugin {});
    ///
    ///     // Run bot with graceful shutdown
    ///     bot.run(guard).await;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn run(&mut self, guard: tokio_graceful::ShutdownGuard) {
        loop {
            tokio::select! {
                _ = guard.cancelled() => {
                    tracing::info!("Shutdown signal received, stopping bot...");
                    break;
                }
                result = self.run_ws() => {
                    match result {
                        Ok(_) => {
                            tracing::info!("WebSocket connection closed gracefully, reconnecting...");
                        }
                        Err(e) => {
                            tracing::error!("WebSocket error: {}, reconnecting in 1s...", e);
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }

        tracing::info!("Bot stopped gracefully");
    }

    async fn run_ws(&mut self) -> Result<()> {
        let url = if let Some(rest) = self.config.base_path.strip_prefix("https://") {
            format!("wss://{}", rest)
        } else if let Some(rest) = self.config.base_path.strip_prefix("http://") {
            format!("ws://{}", rest)
        } else {
            return Err(BotError::InvalidSchema(self.config.base_path.clone()));
        };
        let url = format!("{}/api/v4/websocket", url);

        let response = self.config.client.get(url).upgrade().send().await?;
        let mut websocket = response.into_websocket().await?;

        // Send authentication challenge
        // SAFETY: Token presence is already validated in Bot::new()
        let token = self.config.bearer_access_token.as_ref().unwrap();
        let auth_message = serde_json::json!({
            "seq": 1,
            "action": "authentication_challenge",
            "data": {
                "token": token
            }
        });

        let auth_text = serde_json::to_string(&auth_message)
            .map_err(|e| BotError::MessageSerialization(e.to_string()))?;

        websocket.send(Message::Text(auth_text)).await?;

        while let Some(message) = websocket.try_next().await? {
            if let Message::Text(text) = message {
                tracing::debug!(text, "received");
                let event = serde_json::from_str::<Event>(&text);
                let event = match event {
                    Ok(e) => {
                        tracing::info!(event=?e, "parsed event successfully");
                        e
                    }
                    Err(e) => {
                        tracing::error!("unable to parse: {:?}; {}", e, text);
                        continue;
                    }
                };

                let event = Arc::new(event);

                // Process events in parallel across all interested plugins
                let futures: Vec<_> = self
                    .plugins
                    .iter()
                    .filter(|plugin| plugin.filter(&event))
                    .map(|plugin| {
                        let event = Arc::clone(&event);
                        let config = Arc::clone(&self.config);
                        async move {
                            plugin.process_event(&event, &config).await;
                        }
                    })
                    .collect();

                // Wait for all plugin handlers to complete
                // Errors in individual plugins won't affect others
                futures_util::future::join_all(futures).await;
            } else if let Message::Ping(p) = message {
                websocket.send(Message::Pong(p)).await?;
            } else {
                tracing::warn!(?message, "Received unknown WebSocket message type");
            }
        }

        Ok(())
    }
}
