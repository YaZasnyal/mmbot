//! Thread bot runtime — bridges Layer 2 events to [`ThreadHandler`].
//!
//! Provides [`ThreadBotPlugin`], a [`mattermost_bot::Plugin`] implementation
//! that routes WebSocket events to a [`ThreadHandler`] with per-thread
//! serialization and configurable debounce.
//!
//! # Architecture
//!
//! ```text
//! WebSocket event
//!   → ThreadBotPlugin::process_event()
//!     → route to per-thread actor (mpsc channel)
//!       → debounce timer
//!         → build Thread snapshot (Mattermost API + DB)
//!           → handler.handle()  ← can be interrupted by new events
//!             → execute effects (reply, update, close...)
//! ```
//!
//! Each tracked thread gets its own tokio task (actor) that processes
//! events sequentially. Different threads run in parallel.
//!
//! # Example
//!
//! ```ignore
//! use std::time::Duration;
//! use thread_bot::{ThreadBotPlugin, PgThreadStore};
//!
//! let store = PgThreadStore::new(pool).await?;
//! let plugin = ThreadBotPlugin::new(MyHandler::new(), Arc::new(store))
//!     .with_debounce(Duration::from_secs(3));
//!
//! let bot = Bot::with_config(config)?
//!     .with_plugin(plugin);
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, Mutex, RwLock};

use mattermost_api::apis::configuration::Configuration;
use mattermost_api::apis::{posts_api, users_api};
use mattermost_api::models;
use mattermost_bot::plugin::Event;
use mattermost_bot::types::EventType;

use crate::actor::{
    get_root_post_id, is_root_post, ms_to_datetime, post_to_thread_message, thread_actor, ActorCtx,
    ThreadCommand,
};
use crate::handler::{ThreadContext, ThreadHandler};
use crate::store::ThreadStore;
use crate::types::*;

// ─── Configuration ───────────────────────────────────────────────────────────

/// Configuration for the thread bot runtime.
#[derive(Clone)]
pub struct ThreadBotConfig {
    /// How long to wait after the last message before calling the handler.
    ///
    /// When multiple messages arrive in quick succession, the handler is
    /// called once after the debounce period with all messages in the snapshot.
    ///
    /// Default: 2 seconds
    pub debounce: Duration,

    /// Buffer size for per-thread command channels.
    ///
    /// Default: 64
    pub channel_buffer: usize,

    /// Maximum gap since the last seen message to attempt reconciliation.
    ///
    /// If the bot was offline longer than this window, active threads are
    /// force-closed (status → Stopped) instead of reconciled, and the bot
    /// starts fresh from the current moment.
    ///
    /// Default: `None` (no limit — always reconcile)
    pub max_reconcile_window: Option<Duration>,
}

impl Default for ThreadBotConfig {
    fn default() -> Self {
        Self {
            debounce: Duration::from_secs(2),
            channel_buffer: 64,
            max_reconcile_window: None,
        }
    }
}

// ─── ThreadBotPlugin ─────────────────────────────────────────────────────────

/// Runtime that bridges Layer 2 ([`mattermost_bot::Plugin`]) events to a
/// [`ThreadHandler`] with per-thread serialization and debounce.
///
/// Add to a bot via [`Bot::with_plugin`](mattermost_bot::Bot::with_plugin).
pub struct ThreadBotPlugin<H: ThreadHandler> {
    handler: Arc<H>,
    store: Arc<dyn ThreadStore>,
    config: ThreadBotConfig,

    /// Per-thread command senders, keyed by root_post_id.
    actors: Arc<Mutex<HashMap<String, mpsc::Sender<ThreadCommand>>>>,

    /// Bot's own user ID, fetched in [`Plugin::on_start`].
    bot_user_id: Arc<RwLock<Option<String>>>,
}

// Manual Clone — H doesn't need Clone because it's behind Arc.
impl<H: ThreadHandler> Clone for ThreadBotPlugin<H> {
    fn clone(&self) -> Self {
        Self {
            handler: Arc::clone(&self.handler),
            store: Arc::clone(&self.store),
            config: self.config.clone(),
            actors: Arc::clone(&self.actors),
            bot_user_id: Arc::clone(&self.bot_user_id),
        }
    }
}

impl<H: ThreadHandler> ThreadBotPlugin<H> {
    /// Create a new thread bot plugin with the given handler and store.
    pub fn new(handler: H, store: Arc<dyn ThreadStore>) -> Self {
        Self {
            handler: Arc::new(handler),
            store,
            config: ThreadBotConfig::default(),
            actors: Arc::new(Mutex::new(HashMap::new())),
            bot_user_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the debounce duration (default: 2 seconds).
    pub fn with_debounce(mut self, debounce: Duration) -> Self {
        self.config.debounce = debounce;
        self
    }

    /// Set the per-thread channel buffer size (default: 64).
    pub fn with_channel_buffer(mut self, size: usize) -> Self {
        self.config.channel_buffer = size;
        self
    }

    /// Set the maximum reconciliation window.
    ///
    /// If the bot was offline longer than this, active threads are
    /// force-closed instead of reconciled.
    pub fn with_max_reconcile_window(mut self, window: Duration) -> Self {
        self.config.max_reconcile_window = Some(window);
        self
    }

    // ── Event routing ────────────────────────────────────────────────────

    /// Route a `Posted` event to the appropriate thread actor.
    async fn handle_posted(
        &self,
        posted: &mattermost_bot::types::PostedEvent,
        mm_config: &Arc<Configuration>,
    ) {
        let post = &posted.post.inner;

        if is_root_post(post) {
            self.handle_new_thread(post, mm_config).await;
        } else {
            let root_post_id = get_root_post_id(post);
            self.handle_thread_reply(post, root_post_id, mm_config)
                .await;
        }
    }

    /// Handle a new root post — decide whether to track this thread.
    async fn handle_new_thread(&self, post: &models::Post, mm_config: &Arc<Configuration>) {
        let thread_id = post.id.clone();
        let channel_id = post.channel_id.clone().unwrap_or_default();
        let user_id = post.user_id.clone().unwrap_or_default();
        let post_at = post
            .create_at
            .map(ms_to_datetime)
            .unwrap_or_else(chrono::Utc::now);

        // Check if thread already exists in DB (idempotency for reconnection replays)
        match self.store.get_thread(&thread_id).await {
            Ok(Some(record)) => {
                match record.status {
                    ThreadStatus::New | ThreadStatus::Active => {
                        // Already tracked — ensure actor exists (may be a new session)
                        self.ensure_or_spawn_actor(&thread_id, mm_config).await;
                        // Advance channel checkpoint (harmless if replayed — GREATEST keeps max)
                        if let Err(e) = self
                            .store
                            .advance_channel_checkpoint(&channel_id, post_at)
                            .await
                        {
                            tracing::error!(channel_id = %channel_id, error = %e, "Failed to advance channel checkpoint");
                        }
                        return;
                    }
                    ThreadStatus::Resolved | ThreadStatus::Stopped => {
                        return;
                    }
                }
            }
            Ok(None) => {
                // Not tracked yet — proceed with should_track
            }
            Err(e) => {
                tracing::error!(thread_id = %thread_id, error = %e, "Failed to check thread existence");
                return;
            }
        }

        // Build a minimal Thread snapshot for should_track
        let now = chrono::Utc::now();
        let thread = Thread {
            info: ThreadInfo {
                thread_id: thread_id.clone(),
                root_post_id: thread_id.clone(),
                channel_id: channel_id.clone(),
                creator_user_id: user_id.clone(),
                status: ThreadStatus::New,
                metadata: serde_json::Value::Null,
                last_seen_post_id: None,
                last_seen_post_at: None,
                last_processed_post_id: None,
                last_processed_post_at: None,
                created_at: now,
                updated_at: now,
            },
            messages: vec![{
                let mut msg = post_to_thread_message(post, &thread_id);
                msg.is_new = true;
                msg
            }],
            reactions: vec![],
        };

        let ctx = ThreadContext {
            config: Arc::clone(mm_config),
            store: Arc::clone(&self.store),
            plugin_id: self.handler.id(),
        };

        // Ask handler if it wants to track this thread
        match self.handler.should_track(&thread, &ctx).await {
            Ok(true) => {
                tracing::info!(
                    thread_id = %thread_id,
                    channel_id = %channel_id,
                    "Tracking new thread"
                );
            }
            Ok(false) => {
                tracing::debug!(thread_id = %thread_id, "Handler declined to track thread");
                return;
            }
            Err(e) => {
                tracing::error!(thread_id = %thread_id, error = %e, "should_track failed");
                return;
            }
        }

        // Create thread record in DB
        let upsert = UpsertThread {
            thread_id: thread_id.clone(),
            root_post_id: thread_id.clone(),
            channel_id: channel_id.clone(),
            creator_user_id: user_id,
            status: ThreadStatus::New,
            metadata: serde_json::Value::Null,
        };

        if let Err(e) = self.store.upsert_thread(upsert).await {
            tracing::error!(thread_id = %thread_id, error = %e, "Failed to create thread record");
            return;
        }

        // Register channel checkpoint (first tracked message in this channel)
        if let Err(e) = self
            .store
            .upsert_channel_checkpoint(&channel_id, post_at)
            .await
        {
            tracing::error!(channel_id = %channel_id, error = %e, "Failed to upsert channel checkpoint");
        }

        // Spawn actor and send the initial message
        let tx = {
            let mut actors = self.actors.lock().await;
            let tx = self.spawn_actor(thread_id.clone(), Arc::clone(mm_config));
            actors.insert(thread_id.clone(), tx.clone());
            tx
        };

        if let Err(e) = tx
            .send(ThreadCommand::NewMessage { post: post.clone() })
            .await
        {
            tracing::error!(thread_id = %thread_id, error = %e, "Failed to send initial message to actor");
        }
    }

    /// Handle a reply to an existing (tracked) thread.
    async fn handle_thread_reply(
        &self,
        post: &models::Post,
        root_post_id: &str,
        mm_config: &Arc<Configuration>,
    ) {
        // Look up thread in DB
        let thread_record = match self.store.get_thread(root_post_id).await {
            Ok(Some(record)) => record,
            Ok(None) => return, // Not a tracked thread
            Err(e) => {
                tracing::error!(root_post_id = %root_post_id, error = %e, "Failed to look up thread");
                return;
            }
        };

        // Only process active threads
        if !matches!(
            thread_record.status,
            ThreadStatus::New | ThreadStatus::Active
        ) {
            return;
        }

        // Advance channel checkpoint (covers both bot and user messages)
        let post_at = post
            .create_at
            .map(ms_to_datetime)
            .unwrap_or_else(chrono::Utc::now);
        if let Err(e) = self
            .store
            .advance_channel_checkpoint(&thread_record.channel_id, post_at)
            .await
        {
            tracing::error!(
                channel_id = %thread_record.channel_id,
                error = %e,
                "Failed to advance channel checkpoint"
            );
        }

        // Bot's own messages: save to DB but don't trigger handler re-run
        {
            let bot_user_id = self.bot_user_id.read().await;
            if let Some(bot_id) = bot_user_id.as_ref() {
                if post.user_id.as_deref() == Some(bot_id.as_str()) {
                    let bot_id = bot_id.clone();
                    drop(bot_user_id);

                    let msg = UpsertThreadMessage {
                        post_id: post.id.clone(),
                        thread_id: root_post_id.to_string(),
                        user_id: bot_id,
                        is_bot_message: true,
                        root_id: post.root_id.clone(),
                        parent_post_id: post.root_id.clone(),
                        metadata: serde_json::Value::Null,
                        post_created_at: post
                            .create_at
                            .map(ms_to_datetime)
                            .unwrap_or_else(chrono::Utc::now),
                        post_updated_at: post.update_at.map(ms_to_datetime),
                        post_deleted_at: post.delete_at.and_then(|ts| {
                            if ts == 0 {
                                None
                            } else {
                                Some(ms_to_datetime(ts))
                            }
                        }),
                    };

                    if let Err(e) = self.store.upsert_message(msg).await {
                        tracing::error!(post_id = %post.id, error = %e, "Failed to save bot message");
                    }
                    return;
                }
            }
        }

        // Send to actor (creates one if needed)
        let tx = self.ensure_or_spawn_actor(root_post_id, mm_config).await;
        if let Err(e) = tx
            .send(ThreadCommand::NewMessage { post: post.clone() })
            .await
        {
            tracing::error!(
                root_post_id = %root_post_id,
                error = %e,
                "Failed to send message to actor"
            );
            self.actors.lock().await.remove(root_post_id);
        }
    }

    /// Handle a reaction event (added or removed).
    async fn handle_reaction(
        &self,
        post_id: &str,
        user_id: &str,
        emoji_name: &str,
        action: ReactionAction,
        create_at: i64,
        mm_config: &Arc<Configuration>,
    ) {
        // Look up which thread this post belongs to
        let thread_record = match self.store.get_thread_by_post(post_id).await {
            Ok(Some(record)) => record,
            Ok(None) => return,
            Err(e) => {
                tracing::error!(post_id = %post_id, error = %e, "Failed to look up thread for reaction");
                return;
            }
        };

        // Only process active threads
        if !matches!(
            thread_record.status,
            ThreadStatus::New | ThreadStatus::Active
        ) {
            return;
        }

        let created_at = ms_to_datetime(create_at);
        let change = ReactionChange {
            post_id: post_id.to_string(),
            user_id: user_id.to_string(),
            emoji_name: emoji_name.to_string(),
            action,
            created_at,
        };

        let is_root_reaction = post_id == thread_record.root_post_id;

        if is_root_reaction {
            // Control reaction on root post
            let effect = self.handler.on_control_reaction(&thread_record, &change);

            // Always save the reaction to DB
            let append = AppendReaction {
                thread_id: Some(thread_record.thread_id.clone()),
                post_id: post_id.to_string(),
                user_id: user_id.to_string(),
                emoji_name: emoji_name.to_string(),
                action,
            };
            if let Err(e) = self.store.append_reaction(append).await {
                tracing::error!(post_id = %post_id, error = %e, "Failed to save control reaction");
            }

            // If handler returned an effect, send it to the actor
            if let Some(effect) = effect {
                let tx = self
                    .ensure_or_spawn_actor(&thread_record.root_post_id, mm_config)
                    .await;
                if let Err(e) = tx
                    .send(ThreadCommand::ControlReaction { effect, change })
                    .await
                {
                    tracing::error!(
                        thread_id = %thread_record.thread_id,
                        error = %e,
                        "Failed to send control reaction to actor"
                    );
                    self.actors.lock().await.remove(&thread_record.root_post_id);
                }
            }
        } else {
            // Non-root reaction — save feedback reactions on bot messages to DB
            if self.handler.is_feedback_reaction(emoji_name) {
                if let Ok(Some(msg_record)) = self.store.get_message(post_id).await {
                    if msg_record.is_bot_message {
                        let append = AppendReaction {
                            thread_id: Some(thread_record.thread_id.clone()),
                            post_id: post_id.to_string(),
                            user_id: user_id.to_string(),
                            emoji_name: emoji_name.to_string(),
                            action,
                        };
                        if let Err(e) = self.store.append_reaction(append).await {
                            tracing::error!(
                                post_id = %post_id,
                                error = %e,
                                "Failed to save feedback reaction"
                            );
                        }
                    }
                }
            }
        }
    }

    // ── Reconciliation ───────────────────────────────────────────────────

    /// Reconcile state after a (re)connection.
    ///
    /// Called on every `Hello` event — i.e. on initial connect **and** on
    /// every WebSocket reconnect.  Because `process_event` blocks the WS
    /// read loop, incoming events are buffered by the TCP/WS layer while
    /// reconciliation runs and are processed normally afterwards.
    ///
    /// 1. Snapshot channel checkpoints from DB (timestamps won't move during
    ///    reconciliation because `is_reconciled` is set to `false`).
    /// 2. Load active threads → ensure actors exist → send `Reconcile`.
    /// 3. Scan each checkpointed channel for root posts missed during downtime.
    /// 4. Mark each channel as reconciled — normal processing resumes advancing
    ///    the checkpoint.
    async fn reconcile(&self, config: &Arc<Configuration>) {
        // Snapshot checkpoints before marking them as not-reconciled.
        // This gives us stable scan points that won't be moved by concurrent
        // message processing.
        let checkpoints = match self.store.list_channel_checkpoints().await {
            Ok(cps) => cps,
            Err(e) => {
                tracing::error!(error = %e, "Failed to load channel checkpoints");
                vec![]
            }
        };

        // Block normal checkpoint advances while we scan
        if let Err(e) = self.store.set_all_channels_not_reconciled().await {
            tracing::error!(error = %e, "Failed to mark channels as not reconciled");
        }

        let threads = match self
            .store
            .list_threads_by_status(&[ThreadStatus::New, ThreadStatus::Active])
            .await
        {
            Ok(threads) => threads,
            Err(e) => {
                tracing::error!(error = %e, "Failed to load active threads for reconciliation");
                // Still mark channels reconciled so normal flow resumes
                self.mark_all_channels_reconciled(&checkpoints).await;
                return;
            }
        };

        // Use the oldest checkpoint timestamp for max_reconcile_window check
        let oldest_checkpoint = checkpoints.iter().map(|cp| cp.last_seen_post_at).min();

        // If the gap exceeds max_reconcile_window, force-close everything
        // and start fresh instead of replaying a huge backlog.
        if let (Some(window), Some(since)) = (self.config.max_reconcile_window, oldest_checkpoint) {
            let gap = chrono::Utc::now() - since;
            if let Ok(window_chrono) = chrono::Duration::from_std(window) {
                if gap > window_chrono {
                    tracing::warn!(
                        gap_secs = gap.num_seconds(),
                        window_secs = window_chrono.num_seconds(),
                        threads = threads.len(),
                        "Reconciliation gap exceeds max window, force-closing active threads"
                    );
                    for thread in &threads {
                        if let Err(e) = self
                            .store
                            .update_thread_status(&thread.thread_id, ThreadStatus::Stopped)
                            .await
                        {
                            tracing::error!(
                                thread_id = %thread.thread_id,
                                error = %e,
                                "Failed to force-close thread"
                            );
                        }
                    }
                    // Mark all channels reconciled so normal flow resumes
                    self.mark_all_channels_reconciled(&checkpoints).await;
                    return;
                }
            }
        }

        // Ensure actors exist (idempotent on reconnect) and send Reconcile
        if !threads.is_empty() {
            let mut senders = Vec::with_capacity(threads.len());
            for thread in &threads {
                let tx = self.ensure_or_spawn_actor(&thread.thread_id, config).await;
                senders.push((thread.thread_id.clone(), tx));
            }

            for (thread_id, tx) in &senders {
                if let Err(e) = tx.send(ThreadCommand::Reconcile).await {
                    tracing::error!(
                        thread_id = %thread_id,
                        error = %e,
                        "Failed to send reconcile to actor"
                    );
                }
            }

            tracing::info!(count = senders.len(), "Reconciled active threads");
        }

        // Scan each checkpointed channel for new threads missed during downtime
        if !checkpoints.is_empty() {
            tracing::info!(
                channels = checkpoints.len(),
                "Scanning channels for missed threads"
            );

            for cp in &checkpoints {
                let since_ms = cp.last_seen_post_at.timestamp_millis();

                let post_list = match posts_api::get_posts_for_channel(
                    config,
                    &cp.channel_id,
                    None,
                    Some(200),
                    Some(since_ms),
                    None,
                    None,
                    None,
                )
                .await
                {
                    Ok(pl) => pl,
                    Err(e) => {
                        tracing::error!(
                            channel_id = %cp.channel_id,
                            error = %e,
                            "Failed to scan channel for missed threads"
                        );
                        // Mark this channel reconciled anyway to unblock normal flow
                        if let Err(e) = self.store.set_channel_reconciled(&cp.channel_id).await {
                            tracing::error!(channel_id = %cp.channel_id, error = %e, "Failed to mark channel reconciled");
                        }
                        continue;
                    }
                };

                if let (Some(order), Some(posts)) = (post_list.order, post_list.posts) {
                    for post_id in &order {
                        if let Some(post) = posts.get(post_id) {
                            if !is_root_post(post) {
                                continue;
                            }
                            match self.store.get_thread(&post.id).await {
                                Ok(Some(_)) => continue,
                                Ok(None) => {}
                                Err(e) => {
                                    tracing::error!(
                                        post_id = %post.id,
                                        error = %e,
                                        "Failed to check thread existence during scan"
                                    );
                                    continue;
                                }
                            }
                            self.handle_new_thread(post, config).await;
                        }
                    }
                }

                // Channel scanned — mark reconciled so normal flow can advance it
                if let Err(e) = self.store.set_channel_reconciled(&cp.channel_id).await {
                    tracing::error!(channel_id = %cp.channel_id, error = %e, "Failed to mark channel reconciled");
                }
            }
        }
    }

    /// Helper: mark all checkpointed channels as reconciled.
    async fn mark_all_channels_reconciled(&self, checkpoints: &[crate::types::ChannelCheckpoint]) {
        for cp in checkpoints {
            if let Err(e) = self.store.set_channel_reconciled(&cp.channel_id).await {
                tracing::error!(channel_id = %cp.channel_id, error = %e, "Failed to mark channel reconciled");
            }
        }
    }

    // ── Actor management ─────────────────────────────────────────────────

    /// Get an existing actor or spawn a new one for the given thread.
    async fn ensure_or_spawn_actor(
        &self,
        thread_id: &str,
        mm_config: &Arc<Configuration>,
    ) -> mpsc::Sender<ThreadCommand> {
        let mut actors = self.actors.lock().await;

        if let Some(tx) = actors.get(thread_id) {
            if !tx.is_closed() {
                return tx.clone();
            }
            actors.remove(thread_id);
        }

        let tx = self.spawn_actor(thread_id.to_string(), Arc::clone(mm_config));
        actors.insert(thread_id.to_string(), tx.clone());
        tx
    }

    /// Spawn a new per-thread actor task.
    fn spawn_actor(
        &self,
        thread_id: String,
        mm_config: Arc<Configuration>,
    ) -> mpsc::Sender<ThreadCommand> {
        let (tx, rx) = mpsc::channel(self.config.channel_buffer);

        let actors = Arc::clone(&self.actors);

        let actor_ctx = ActorCtx {
            handler: Arc::clone(&self.handler),
            store: Arc::clone(&self.store),
            ctx: ThreadContext {
                config: Arc::clone(&mm_config),
                store: Arc::clone(&self.store),
                plugin_id: self.handler.id(),
            },
            mm_config,
            debounce: self.config.debounce,
            thread_id: thread_id.clone(),
            bot_user_id: Arc::clone(&self.bot_user_id),
        };

        tokio::spawn(async move {
            thread_actor(rx, actor_ctx).await;
            actors.lock().await.remove(&thread_id);
        });

        tx
    }
}

// ─── Plugin trait implementation ─────────────────────────────────────────────

#[async_trait::async_trait]
impl<H: ThreadHandler> mattermost_bot::Plugin for ThreadBotPlugin<H> {
    fn id(&self) -> &'static str {
        self.handler.id()
    }

    async fn on_start(&self, config: &Arc<Configuration>) -> mattermost_bot::Result<()> {
        match users_api::get_user(config, "me").await {
            Ok(user) => {
                if let Some(id) = user.id {
                    tracing::info!(bot_user_id = %id, "Fetched bot user ID");
                    *self.bot_user_id.write().await = Some(id);
                }
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to fetch bot user ID; bot message detection will be disabled"
                );
            }
        }

        // Reconciliation is triggered by the Hello event (see process_event),
        // which fires on every WebSocket connection — including reconnects.

        Ok(())
    }

    async fn on_shutdown(&self, _config: &Arc<Configuration>) -> mattermost_bot::Result<()> {
        let senders: Vec<mpsc::Sender<ThreadCommand>> = {
            let actors = self.actors.lock().await;
            actors.values().cloned().collect()
        };

        tracing::info!(
            actor_count = senders.len(),
            "Sending shutdown to thread actors"
        );

        for tx in senders {
            let _ = tx.send(ThreadCommand::Shutdown).await;
        }

        Ok(())
    }

    fn filter(&self, event: &Arc<Event>) -> bool {
        matches!(
            event.data,
            EventType::Hello(_)
                | EventType::Posted(_)
                | EventType::ReactionAdded(_)
                | EventType::ReactionRemoved(_)
        )
    }

    async fn process_event(&self, event: &Arc<Event>, config: &Arc<Configuration>) {
        match &event.data {
            EventType::Hello(_) => {
                let plugin = self.clone();
                let config = Arc::clone(config);
                tokio::spawn(async move {
                    tracing::info!("Starting reconcile");
                    plugin.reconcile(&config).await;
                });
            }
            EventType::Posted(posted) => {
                self.handle_posted(posted, config).await;
            }
            EventType::ReactionAdded(reaction) => {
                let r = &reaction.reaction.inner;
                if let (Some(post_id), Some(user_id), Some(emoji_name)) =
                    (&r.post_id, &r.user_id, &r.emoji_name)
                {
                    self.handle_reaction(
                        post_id,
                        user_id,
                        emoji_name,
                        ReactionAction::Added,
                        r.create_at.unwrap_or(0),
                        config,
                    )
                    .await;
                }
            }
            EventType::ReactionRemoved(reaction) => {
                let r = &reaction.reaction.inner;
                if let (Some(post_id), Some(user_id), Some(emoji_name)) =
                    (&r.post_id, &r.user_id, &r.emoji_name)
                {
                    self.handle_reaction(
                        post_id,
                        user_id,
                        emoji_name,
                        ReactionAction::Removed,
                        r.create_at.unwrap_or(0),
                        config,
                    )
                    .await;
                }
            }
            _ => {}
        }
    }
}
