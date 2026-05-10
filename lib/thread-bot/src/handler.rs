use crate::actor::build_thread_snapshot;
use crate::error::ThreadBotError;
use crate::handle::ThreadBotHandle;
use crate::types::{ReactionChange, Thread, ThreadInvocation, ThreadRecord};
use async_trait::async_trait;
use mattermost_bot::{chrono, cron_tab};
use std::sync::Arc;

/// Target thread for effects that can address another tracked thread.
#[derive(Debug, Clone)]
pub enum ThreadTarget {
    /// The thread currently being handled.
    CurrentThread,
    /// A specific tracked thread.
    Thread { thread_id: String },
    /// All threads linked from the current thread by `link_kind`.
    LinkedThreads { link_kind: String },
}

/// Context provided to thread handlers
#[derive(Clone)]
pub struct ThreadContext {
    pub config: Arc<mattermost_api::apis::configuration::Configuration>,
    pub store: Arc<dyn ThreadStore>,
    pub plugin_id: &'static str,
    pub bot_user_id: Option<String>,
}

impl ThreadContext {
    /// Build a full normalized thread snapshot with live Mattermost messages
    /// and DB-backed metadata.
    ///
    /// Expensive: makes HTTP calls to Mattermost. Use for report generation
    /// or workflows that need full message text outside the active handler
    /// snapshot.
    pub async fn build_thread_snapshot(&self, thread_id: &str) -> Result<Thread, ThreadBotError> {
        build_thread_snapshot(thread_id, &*self.store, &self.config).await
    }
}

/// Side effect to be applied by Layer 3
#[derive(Debug, Clone)]
pub enum ThreadEffect {
    /// No operation
    Noop,
    /// Reply to a target thread.
    Reply {
        target: ThreadTarget,
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
    /// Set message-level metadata in DB (full replacement)
    SetMessageMetadata {
        post_id: String,
        metadata: serde_json::Value,
    },
    /// Create a typed linked root thread.
    ///
    /// Layer 3 creates a root post in `channel_id`, tracks it as a thread,
    /// stores the created root message, and records the link.
    EnsureLinkedThread {
        link_kind: String,
        channel_id: String,
        thread_kind: Option<String>,
        message: String,
        metadata: serde_json::Value,
        thread_metadata: serde_json::Value,
        link_metadata: serde_json::Value,
    },
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

    /// Optional free-string classifier stored on newly tracked threads.
    ///
    /// Layer 3 stores and indexes this value, but does not interpret it.
    /// Future tracking-decision APIs will let handlers return this together
    /// with initial metadata.
    fn thread_kind(&self, _thread: &Thread) -> Option<String> {
        None
    }

    /// Main thread processing logic.
    ///
    /// Called after debounce with a lightweight thread record and trigger.
    /// Build a full snapshot explicitly with
    /// [`ThreadContext::build_thread_snapshot`] only when message/reaction
    /// history is needed.
    /// Returns a list of effects to apply.
    async fn handle(
        &self,
        invocation: &ThreadInvocation,
        ctx: &ThreadContext,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError>;

    /// Synchronous control reaction handler for root post.
    ///
    /// Cannot do async work directly; return effects for Layer 3 to execute.
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
        _change: &ReactionChange,
    ) -> Option<ThreadEffect> {
        None
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

// Re-export ThreadStore trait here for convenience
pub use crate::store::ThreadStore;
