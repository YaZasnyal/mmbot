// Public API
mod actor;
pub mod error;
pub mod handler;
pub mod pg_store;
pub mod runtime;
pub mod store;
pub mod types;

#[cfg(test)]
mod actor_tests;
#[cfg(test)]
mod testutil;

// Re-exports for convenience
pub use error::ThreadBotError;
pub use handler::{
    default_control_reactions, ThreadCloseReason, ThreadContext, ThreadEffect, ThreadHandler,
};
pub use pg_store::PgThreadStore;
pub use runtime::{ThreadBotConfig, ThreadBotPlugin};
pub use store::ThreadStore;
pub use types::{
    AppendReaction, ChannelCheckpoint, ReactionAction, ReactionChange, Thread, ThreadInfo,
    ThreadMessage, ThreadMessageRecord, ThreadReaction, ThreadRecord, ThreadStatus, UpsertThread,
    UpsertThreadMessage,
};

// Re-export async_trait for convenience
pub use async_trait::async_trait;

pub use sqlx;
