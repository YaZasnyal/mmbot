use crate::error::ThreadBotError;
use crate::types::{ReactionChange, Thread, ThreadRecord};
use async_trait::async_trait;
use std::sync::Arc;

/// Context provided to thread handlers
#[derive(Clone)]
pub struct ThreadContext {
    pub config: Arc<mattermost_api::apis::configuration::Configuration>,
    pub store: Arc<dyn ThreadStore>,
    pub plugin_id: &'static str,
}

/// Side effect to be applied by Layer 3
#[derive(Debug, Clone)]
pub enum ThreadEffect {
    /// No operation
    Noop,
    /// Reply to the thread
    Reply {
        message: String,
        metadata: serde_json::Value,
    },
    /// Update an existing message
    UpdateMessage {
        post_id: String,
        message: Option<String>,
        metadata: Option<serde_json::Value>,
    },
    /// Set thread-level metadata
    SetThreadMetadata { metadata: serde_json::Value },
    /// Mark thread as resolved
    MarkResolved,
    /// Mark thread as stopped
    MarkStopped,
}

/// Reason why thread was closed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadCloseReason {
    ResolvedByReaction,
    StoppedByReaction,
    ResolvedByHandler,
    StoppedByHandler,
}

/// Thread handler trait - implement this to create a thread-based bot
#[async_trait]
pub trait ThreadHandler: Send + Sync + 'static {
    /// Unique identifier for this handler
    fn id(&self) -> &'static str;

    /// Decide whether to start tracking this thread
    async fn should_track(
        &self,
        thread: &Thread,
        ctx: &ThreadContext,
    ) -> Result<bool, ThreadBotError>;

    /// Main thread processing logic
    ///
    /// Called after debounce with full thread snapshot from Mattermost API.
    /// Returns a list of effects to apply.
    async fn handle(
        &self,
        thread: &Thread,
        ctx: &ThreadContext,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError>;

    /// Synchronous control reaction handler for root post
    ///
    /// Called ONLY if thread is in New or Active status.
    /// Cannot do async work - return ThreadEffect for that.
    ///
    /// # Parameters
    /// - `thread_record`: lightweight thread record from DB (no messages)
    /// - `change`: reaction change event (who, what, when)
    ///
    /// # Returns
    /// - `Some(effect)` to perform an action
    /// - `None` to ignore the reaction
    fn on_control_reaction(
        &self,
        _thread_record: &ThreadRecord,
        change: &ReactionChange,
    ) -> Option<ThreadEffect> {
        default_control_reactions(change)
    }

    /// Determine if an emoji is a feedback reaction
    ///
    /// Feedback reactions are saved to DB only for bot messages
    /// (with is_bot_message = true) for later analytics.
    ///
    /// Default: 👍 and 👎
    fn is_feedback_reaction(&self, emoji_name: &str) -> bool {
        matches!(emoji_name, "thumbsup" | "thumbsdown" | "+1" | "-1")
    }

    /// Called when thread is closed (resolved or stopped)
    async fn on_thread_closed(
        &self,
        thread: &Thread,
        reason: ThreadCloseReason,
        ctx: &ThreadContext,
    ) -> Result<(), ThreadBotError> {
        let _ = (thread, reason, ctx);
        Ok(())
    }
}

/// Default control reaction logic
///
/// - ✅ (white_check_mark) on Added → MarkResolved
/// - 🛑 (stop_sign) on Added → MarkStopped
/// - Everything else → None
pub fn default_control_reactions(change: &ReactionChange) -> Option<ThreadEffect> {
    use crate::types::ReactionAction;

    match (&change.action, change.emoji_name.as_str()) {
        (ReactionAction::Added, "white_check_mark") => Some(ThreadEffect::MarkResolved),
        (ReactionAction::Added, "stop_sign") => Some(ThreadEffect::MarkStopped),
        _ => None,
    }
}

// Re-export ThreadStore trait here for convenience
pub use crate::store::ThreadStore;
