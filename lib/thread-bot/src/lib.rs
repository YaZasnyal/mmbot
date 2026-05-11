// Public API
mod actor;
mod actor_registry;
mod channel_messages;
pub mod error;
pub mod handle;
pub mod handler;
pub mod metrics;
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
pub use handle::ThreadBotHandle;
pub use handler::{ThreadContext, ThreadEffect, ThreadHandler, ThreadMetadataTarget, ThreadTarget};
pub use metrics::{ThreadBotMetrics, ThreadBotMetricsHandle};
pub use pg_store::PgThreadStore;
pub use runtime::{ThreadBotConfig, ThreadBotPlugin};
pub use store::ThreadStore;
pub use types::{
    AppendReaction, ChannelCheckpoint, ReactionAction, ReactionChange, Thread, ThreadInfo,
    ThreadInvocation, ThreadLink, ThreadMessage, ThreadMessageRecord, ThreadReaction, ThreadRecord,
    ThreadTrigger, UpsertThread, UpsertThreadLink, UpsertThreadMessage,
};

// Re-export async_trait for convenience
pub use async_trait::async_trait;

pub use mattermost_bot::cron_tab;
pub use prometheus_client;
pub use sqlx;
