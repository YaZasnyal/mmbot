use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Thread status in the lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadStatus {
    /// Transient state: thread is tracked but first handler run hasn't completed
    New,
    /// Thread is actively tracked and processed
    Active,
    /// Thread was resolved (e.g., by ✅ reaction)
    Resolved,
    /// Thread was stopped (e.g., by 🛑 reaction)
    Stopped,
}

/// Lightweight thread record from database (without messages)
#[derive(Debug, Clone)]
pub struct ThreadRecord {
    pub thread_id: String,
    pub root_post_id: String,
    pub channel_id: String,
    pub creator_user_id: String,
    pub status: ThreadStatus,
    pub metadata: serde_json::Value,
    pub last_seen_post_id: Option<String>,
    pub last_seen_post_at: Option<DateTime<Utc>>,
    pub last_processed_post_id: Option<String>,
    pub last_processed_post_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Full thread snapshot for handler (expensive to build)
#[derive(Debug, Clone)]
pub struct Thread {
    pub info: ThreadInfo,
    pub messages: Vec<ThreadMessage>,
    pub reactions: Vec<ThreadReaction>,
}

/// Thread metadata (converted from ThreadRecord)
#[derive(Debug, Clone)]
pub struct ThreadInfo {
    pub thread_id: String,
    pub root_post_id: String,
    pub channel_id: String,
    pub creator_user_id: String,
    pub status: ThreadStatus,
    pub metadata: serde_json::Value,
    pub last_seen_post_id: Option<String>,
    pub last_seen_post_at: Option<DateTime<Utc>>,
    pub last_processed_post_id: Option<String>,
    pub last_processed_post_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ThreadRecord> for ThreadInfo {
    fn from(record: ThreadRecord) -> Self {
        Self {
            thread_id: record.thread_id,
            root_post_id: record.root_post_id,
            channel_id: record.channel_id,
            creator_user_id: record.creator_user_id,
            status: record.status,
            metadata: record.metadata,
            last_seen_post_id: record.last_seen_post_id,
            last_seen_post_at: record.last_seen_post_at,
            last_processed_post_id: record.last_processed_post_id,
            last_processed_post_at: record.last_processed_post_at,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// Thread message snapshot (from Mattermost API + DB metadata)
#[derive(Debug, Clone)]
pub struct ThreadMessage {
    pub post_id: String,
    pub thread_id: String,
    pub user_id: String,
    pub message: String,
    pub root_id: Option<String>,
    pub parent_post_id: Option<String>,
    pub props: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Thread reaction event
#[derive(Debug, Clone)]
pub struct ThreadReaction {
    pub post_id: String,
    pub user_id: String,
    pub emoji_name: String,
    pub action: ReactionAction,
    pub created_at: DateTime<Utc>,
}

/// Reaction action type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReactionAction {
    Added,
    Removed,
}

/// Reaction change event passed to on_control_reaction()
#[derive(Debug, Clone)]
pub struct ReactionChange {
    pub post_id: String,
    pub user_id: String,
    pub emoji_name: String,
    pub action: ReactionAction,
    pub created_at: DateTime<Utc>,
}

/// Thread message record from database
#[derive(Debug, Clone)]
pub struct ThreadMessageRecord {
    pub post_id: String,
    pub thread_id: String,
    pub user_id: String,
    pub is_bot_message: bool,
    pub root_id: Option<String>,
    pub parent_post_id: Option<String>,
    pub metadata: serde_json::Value,
    pub post_created_at: DateTime<Utc>,
    pub post_updated_at: Option<DateTime<Utc>>,
    pub post_deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for upserting a thread
#[derive(Debug, Clone)]
pub struct UpsertThread {
    pub thread_id: String,
    pub root_post_id: String,
    pub channel_id: String,
    pub creator_user_id: String,
    pub status: ThreadStatus,
    pub metadata: serde_json::Value,
}

/// Input for upserting a thread message
#[derive(Debug, Clone)]
pub struct UpsertThreadMessage {
    pub post_id: String,
    pub thread_id: String,
    pub user_id: String,
    pub is_bot_message: bool,
    pub root_id: Option<String>,
    pub parent_post_id: Option<String>,
    pub metadata: serde_json::Value,
    pub post_created_at: DateTime<Utc>,
    pub post_updated_at: Option<DateTime<Utc>>,
    pub post_deleted_at: Option<DateTime<Utc>>,
}

/// Input for appending a reaction
#[derive(Debug, Clone)]
pub struct AppendReaction {
    pub thread_id: Option<String>,
    pub post_id: String,
    pub user_id: String,
    pub emoji_name: String,
    pub action: ReactionAction,
}
