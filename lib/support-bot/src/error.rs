use thiserror::Error;

pub type Result<T> = std::result::Result<T, SupportBotError>;

#[derive(Debug, Error)]
pub enum SupportBotError {
    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Unknown tool: {0}")]
    UnknownTool(String),

    #[error("Instruction error: {0}")]
    Instruction(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Internal error: {0}")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl SupportBotError {
    pub fn internal(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Internal(Box::new(err))
    }
}

impl From<SupportBotError> for thread_bot::ThreadBotError {
    fn from(error: SupportBotError) -> Self {
        thread_bot::ThreadBotError::internal(error)
    }
}
