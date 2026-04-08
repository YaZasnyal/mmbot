use crate::error::ThreadBotError;
use crate::types::{
    AppendReaction, ThreadMessageRecord, ThreadReaction, ThreadRecord, ThreadStatus, UpsertThread,
    UpsertThreadMessage,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Thread storage abstraction
#[async_trait]
pub trait ThreadStore: Send + Sync + 'static {
    /// Create or update a thread
    async fn upsert_thread(&self, input: UpsertThread) -> Result<ThreadRecord, ThreadBotError>;

    /// Get thread by thread_id
    async fn get_thread(&self, thread_id: &str) -> Result<Option<ThreadRecord>, ThreadBotError>;

    /// Find thread by any post_id in the thread (lightweight query)
    ///
    /// Used for routing reactions and messages to the correct thread.
    async fn get_thread_by_post(
        &self,
        post_id: &str,
    ) -> Result<Option<ThreadRecord>, ThreadBotError>;

    /// List threads by status
    async fn list_threads_by_status(
        &self,
        statuses: &[ThreadStatus],
    ) -> Result<Vec<ThreadRecord>, ThreadBotError>;

    /// Update thread status
    async fn update_thread_status(
        &self,
        thread_id: &str,
        status: ThreadStatus,
    ) -> Result<(), ThreadBotError>;

    /// Set thread metadata (full replacement)
    async fn set_thread_metadata(
        &self,
        thread_id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError>;

    /// Update thread seen position
    async fn update_thread_seen(
        &self,
        thread_id: &str,
        post_id: &str,
        seen_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError>;

    /// Update thread processed position
    async fn update_thread_processed(
        &self,
        thread_id: &str,
        post_id: &str,
        processed_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError>;

    /// Create or update a thread message
    async fn upsert_message(
        &self,
        input: UpsertThreadMessage,
    ) -> Result<ThreadMessageRecord, ThreadBotError>;

    /// Get message by post_id
    async fn get_message(
        &self,
        post_id: &str,
    ) -> Result<Option<ThreadMessageRecord>, ThreadBotError>;

    /// List all messages in a thread
    async fn list_thread_messages(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadMessageRecord>, ThreadBotError>;

    /// Set message metadata (full replacement)
    async fn set_message_metadata(
        &self,
        post_id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError>;

    /// Append a reaction event
    async fn append_reaction(&self, input: AppendReaction) -> Result<(), ThreadBotError>;

    /// Get list of reactions for a thread (for building Thread snapshot)
    async fn list_thread_reactions(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadReaction>, ThreadBotError>;
}
