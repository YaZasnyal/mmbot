use thiserror::Error;

/// Errors that can occur during bot operation
#[derive(Debug, Error)]
pub enum BotError {
    /// WebSocket connection error
    #[error("WebSocket error: {0}")]
    WebSocket(#[source] Box<reqwest_websocket::Error>),

    /// Missing bearer access token in configuration
    #[error("Missing bearer access token in configuration")]
    MissingToken,

    /// Invalid URL schema in base_path (expected http:// or https://)
    #[error("Invalid URL schema in base_path: {0} (expected http:// or https://)")]
    InvalidSchema(String),

    /// Event parsing error
    #[error("Failed to parse event: {0}")]
    EventParse(#[from] serde_json::Error),

    /// WebSocket message serialization error
    #[error("Failed to serialize WebSocket message: {0}")]
    MessageSerialization(String),
}

/// Result type alias for bot operations
pub type Result<T> = std::result::Result<T, BotError>;

impl From<reqwest_websocket::Error> for BotError {
    fn from(err: reqwest_websocket::Error) -> Self {
        BotError::WebSocket(Box::new(err))
    }
}
