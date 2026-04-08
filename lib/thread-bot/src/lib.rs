// Public API
pub mod error;
pub mod handler;
pub mod store;
pub mod types;

// Re-exports for convenience
pub use error::ThreadBotError;
pub use handler::{
    default_control_reactions, ThreadCloseReason, ThreadContext, ThreadEffect, ThreadHandler,
};
pub use store::ThreadStore;
pub use types::{
    AppendReaction, ReactionAction, ReactionChange, Thread, ThreadInfo, ThreadMessage,
    ThreadMessageRecord, ThreadReaction, ThreadRecord, ThreadStatus, UpsertThread,
    UpsertThreadMessage,
};

// Re-export async_trait for convenience
pub use async_trait::async_trait;
