use std::sync::Arc;

use futures_util::{SinkExt, TryStreamExt};
use mattermost_api::apis::configuration::Configuration;
use plugin::Event;
use reqwest_websocket::{Message, RequestBuilderExt};

mod nested_decoder;
mod plugin;
mod types;

pub use plugin::{Event, Plugin};

// reexports
pub use async_trait::async_trait;

pub struct Bot {
    config: Arc<Configuration>,

    plugins: Vec<Box<dyn Plugin>>,
}

impl Bot {
    pub fn new(config: Configuration) -> Self {
        Self {
            config: Arc::new(config),
            plugins: Default::default(),
        }
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) {
        self.plugins.push(Box::new(plugin));
    }

    pub async fn run(&mut self) {
        loop {
            match self.run_ws().await {
                Ok(_) => { /* add tracing log */ }
                Err(e) => {
                    // TODO: add tracing log
                    eprintln!("ws error: {:?}", e);
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    async fn run_ws(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let url = if self.config.base_path.starts_with("https://") {
            self.config.base_path.replace("https", "wss")
        } else if self.config.base_path.starts_with("http://") {
            self.config.base_path.replace("http", "ws")
        } else {
            panic!("unknown schema");
        };
        let url = format!("{}/api/v4/websocket", url);

        let response = self.config.client.get(url).upgrade().send().await?;
        let mut websocket = response.into_websocket().await?;

        // TODO: make normal auth
        websocket
            .send(Message::Text(format!(
                r#"{{
            "seq": 1,
            "action": "authentication_challenge",
            "data": {{
              "token": "{}"
            }}
          }}"#,
                self.config.bearer_access_token.as_ref().unwrap()
            )))
            .await?;

        while let Some(message) = websocket.try_next().await? {
            if let Message::Text(text) = message {
                println!("received: {text}");
                let event = serde_json::from_str::<Event>(&text);
                match event {
                    Ok(e) => {
                        println!("parsed: {e:?}");
                    }
                    Err(e) => {
                        eprintln!("unable to parse: {:?}", e);
                    }
                }
            } else if let Message::Ping(p) = message {
                websocket.send(Message::Pong(p)).await?;
            } else {
                println!("received unknown: {message:?}");
            }
        }

        Ok(())
    }
}
