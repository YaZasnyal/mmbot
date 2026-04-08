use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThreadBotError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Mattermost API error: {0}")]
    MattermostApi(String),

    #[error("Thread not found: {0}")]
    ThreadNotFound(String),

    #[error("Message not found: {0}")]
    MessageNotFound(String),

    #[error("Invalid thread state: {0}")]
    InvalidState(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl ThreadBotError {
    /// Create an internal error from any error type
    pub fn internal(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Internal(Box::new(err))
    }

    /// Create a Mattermost API error from any error type
    pub fn mattermost_api(err: impl std::fmt::Display) -> Self {
        Self::MattermostApi(err.to_string())
    }
}
