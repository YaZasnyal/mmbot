use crate::error::ThreadBotError;
use crate::handle::ThreadBotHandle;
use crate::types::{ReactionChange, Thread, ThreadRecord};
use async_trait::async_trait;
use mattermost_bot::{chrono, cron_tab};
use std::sync::Arc;

/// Context provided to thread handlers
#[derive(Clone)]
pub struct ThreadContext {
    pub config: Arc<mattermost_api::apis::configuration::Configuration>,
    pub store: Arc<dyn ThreadStore>,
    pub plugin_id: &'static str,
    pub bot_user_id: Option<String>,
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
    /// Reschedule handler run after applying effects
    ///
    /// Useful for:
    /// - Posting initial "working on it" message, then continuing work
    /// - Saving intermediate state and resuming later
    /// - Multi-step workflows with user-visible progress
    ///
    /// After applying all effects in the batch, Layer 3 will schedule
    /// a new handler run (respecting debounce if new messages arrive).
    Reschedule,
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
    /// Thread resolved via ThreadBotHandle (cron job, admin, etc.)
    ResolvedExternally,
    /// Thread stopped via ThreadBotHandle (cron job, admin, etc.)
    StoppedExternally,
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

    /// Configure periodic tasks for this handler.
    ///
    /// Called once at bot startup. Register cron jobs using `scheduler`,
    /// and use `handle` to access thread data and actor commands.
    ///
    /// Cron closures are sync (`Fn() + Send + 'static`).
    /// For async work use `tokio::runtime::Handle::current().block_on(...)`.
    ///
    /// Clone both `self` (Arc) and `handle` before each `scheduler.add_fn()`.
    fn setup_cron(
        self: Arc<Self>,
        _scheduler: &mut cron_tab::Cron<chrono::Utc>,
        _handle: ThreadBotHandle,
    ) {
        // Default: no cron jobs
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
