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
//! ```no_run
//! # use std::sync::Arc;
//! # use thread_bot::{async_trait, ThreadHandler, ThreadEffect, ThreadBotError, ThreadContext, ThreadInvocation};
//! # struct MyHandler;
//! # impl MyHandler { fn new() -> Self { Self } }
//! # #[async_trait]
//! # impl ThreadHandler for MyHandler {
//! #     fn id(&self) -> &'static str { "my" }
//! #     async fn handle(&self, _: &ThreadInvocation, _: &ThreadContext) -> Result<Vec<ThreadEffect>, ThreadBotError> { Ok(vec![]) }
//! # }
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let pool: thread_bot::sqlx::PgPool = todo!();
//! # let config: mattermost_bot::Configuration = todo!();
//! use std::time::Duration;
//! use thread_bot::{ThreadBotPlugin, PgThreadStore};
//!
//! let store = PgThreadStore::new(pool).await?;
//! let plugin = ThreadBotPlugin::new(MyHandler::new(), Arc::new(store))
//!     .with_debounce(Duration::from_secs(3));
//!
//! # use mattermost_bot::Bot;
//! let bot = Bot::with_config(config)?
//!     .with_plugin(plugin);
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use tokio::sync::{Mutex, RwLock};

use mattermost_api::apis::configuration::Configuration;
use mattermost_api::apis::users_api;
use mattermost_api::models;
use mattermost_bot::plugin::Event;
use mattermost_bot::types::EventType;
use mattermost_bot::{chrono, cron_tab};

use crate::handle::ThreadBotHandle;

use crate::actor::{
    get_root_post_id, is_root_post, ms_to_datetime, post_to_upsert_thread_message, ThreadCommand,
};
use crate::actor_registry::{ActorRegistry, ActorSpawnConfig};
use crate::channel_messages::{channel_messages_after, latest_channel_post};
use crate::error::ThreadBotError;
use crate::handler::{ThreadEffect, ThreadHandler};
use crate::metrics::ThreadBotMetricsHandle;
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
    /// If the bot was offline longer than this window, the channel backlog is
    /// skipped and the checkpoint is advanced to the latest channel post.
    ///
    /// Default: `None` (no limit — always reconcile)
    pub max_reconcile_window: Option<Duration>,

    /// How long an idle per-thread actor stays alive after all work completes.
    ///
    /// `Duration::ZERO` means the actor exits as soon as it has no pending
    /// work, running handler, or reschedule.
    ///
    /// Default: 0 seconds
    pub actor_idle_timeout: Duration,
}

impl Default for ThreadBotConfig {
    fn default() -> Self {
        Self {
            debounce: Duration::from_secs(2),
            channel_buffer: 64,
            max_reconcile_window: None,
            actor_idle_timeout: Duration::ZERO,
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

    /// Per-thread actor command registry, keyed by root_post_id.
    registry: ActorRegistry,

    /// Bot's own user ID, fetched in [`Plugin::on_start`].
    bot_user_id: Arc<RwLock<Option<String>>>,

    /// Ensures reconnect storms cannot run overlapping reconciliation scans.
    reconciliation_gate: Arc<Mutex<()>>,

    metrics: ThreadBotMetricsHandle,
}

// Manual Clone — H doesn't need Clone because it's behind Arc.
impl<H: ThreadHandler> Clone for ThreadBotPlugin<H> {
    fn clone(&self) -> Self {
        Self {
            handler: Arc::clone(&self.handler),
            store: Arc::clone(&self.store),
            config: self.config.clone(),
            registry: self.registry.clone(),
            bot_user_id: Arc::clone(&self.bot_user_id),
            reconciliation_gate: Arc::clone(&self.reconciliation_gate),
            metrics: self.metrics.clone(),
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
            registry: ActorRegistry::new(),
            bot_user_id: Arc::new(RwLock::new(None)),
            reconciliation_gate: Arc::new(Mutex::new(())),
            metrics: ThreadBotMetricsHandle::noop(),
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
    /// If the bot was offline longer than this, channel backlog is skipped.
    pub fn with_max_reconcile_window(mut self, window: Duration) -> Self {
        self.config.max_reconcile_window = Some(window);
        self
    }

    /// Set how long idle per-thread actors stay alive after work completes.
    pub fn with_actor_idle_timeout(mut self, timeout: Duration) -> Self {
        self.config.actor_idle_timeout = timeout;
        self
    }

    pub fn with_metrics(mut self, metrics: ThreadBotMetricsHandle) -> Self {
        self.metrics = metrics;
        self
    }

    // ── Event routing ────────────────────────────────────────────────────

    /// Route a `Posted` event to the appropriate thread actor.
    async fn handle_posted(
        &self,
        posted: &mattermost_bot::types::PostedEvent,
        mm_config: &Arc<Configuration>,
    ) {
        self.route_post(&posted.post.inner, mm_config).await;
    }

    /// Route a Mattermost post to the appropriate thread actor.
    async fn route_post(&self, post: &models::Post, mm_config: &Arc<Configuration>) {
        if is_root_post(post) {
            self.handle_new_thread(post, mm_config).await;
        } else {
            let root_post_id = get_root_post_id(post);
            self.handle_thread_reply(post, root_post_id, mm_config)
                .await;
        }
    }

    /// Handle a new root post by persisting and routing the thread.
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
            Ok(Some(_)) => {
                // Already tracked — this is an idempotent replay, so only move
                // the channel checkpoint. Replies/reactions spawn actors when
                // there is actual work to route.
                self.advance_channel_if_reconciled(&channel_id, post, post_at)
                    .await;
                return;
            }
            Ok(None) => {}
            Err(e) => {
                tracing::error!(thread_id = %thread_id, error = %e, "Failed to check thread existence");
                return;
            }
        }

        tracing::info!(
            thread_id = %thread_id,
            channel_id = %channel_id,
            "Persisting new thread"
        );

        // Create thread record in DB
        let upsert = UpsertThread {
            thread_id: thread_id.clone(),
            root_post_id: thread_id.clone(),
            channel_id: channel_id.clone(),
            creator_user_id: user_id,
            thread_kind: None,
            metadata: serde_json::Value::Null,
        };

        if let Err(e) = self.store.upsert_thread(upsert).await {
            tracing::error!(thread_id = %thread_id, error = %e, "Failed to create thread record");
            return;
        }

        self.register_channel_if_needed(&channel_id, post, post_at)
            .await;

        if let Err(e) = self
            .registry
            .send_or_spawn(
                ThreadCommand::NewMessage { post: post.clone() },
                self.actor_spawn_config(&thread_id, mm_config),
            )
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

        // Advance channel checkpoint (covers both bot and user messages)
        let post_at = post
            .create_at
            .map(ms_to_datetime)
            .unwrap_or_else(chrono::Utc::now);
        self.advance_channel_if_reconciled(&thread_record.channel_id, post, post_at)
            .await;

        // Bot's own messages: save to DB but don't trigger handler re-run
        {
            let bot_user_id = self.bot_user_id.read().await;
            if let Some(bot_id) = bot_user_id.as_ref() {
                if post.user_id.as_deref() == Some(bot_id.as_str()) {
                    drop(bot_user_id);

                    let msg = post_to_upsert_thread_message(post, root_post_id, true);

                    if let Err(e) = self.store.upsert_message(msg).await {
                        tracing::error!(post_id = %post.id, error = %e, "Failed to save bot message");
                    }
                    return;
                }
            }
        }

        if let Err(e) = self
            .registry
            .send_or_spawn(
                ThreadCommand::NewMessage { post: post.clone() },
                self.actor_spawn_config(root_post_id, mm_config),
            )
            .await
        {
            tracing::error!(
                root_post_id = %root_post_id,
                error = %e,
                "Failed to send message to actor"
            );
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

        let created_at = ms_to_datetime(create_at);
        let change = ReactionChange {
            post_id: post_id.to_string(),
            user_id: user_id.to_string(),
            emoji_name: emoji_name.to_string(),
            action,
            created_at,
        };

        if change.post_id == thread_record.root_post_id {
            if let Some(effect) = self.handler.on_control_reaction(&thread_record, &change) {
                self.dispatch_control_reaction(&thread_record, change, effect, mm_config)
                    .await;
            }
        }
    }

    async fn dispatch_control_reaction(
        &self,
        thread_record: &ThreadRecord,
        change: ReactionChange,
        effect: ThreadEffect,
        mm_config: &Arc<Configuration>,
    ) {
        if let Err(e) = self
            .registry
            .send_or_spawn(
                ThreadCommand::ControlReaction { effect, change },
                self.actor_spawn_config(&thread_record.root_post_id, mm_config),
            )
            .await
        {
            tracing::error!(
                thread_id = %thread_record.thread_id,
                error = %e,
                "Failed to send control reaction to actor"
            );
        }
    }

    // ── Reconciliation ───────────────────────────────────────────────────

    /// Reconcile state after a (re)connection.
    ///
    /// Called after a `Hello` event — i.e. on initial connect **and** on every
    /// WebSocket reconnect.
    ///
    /// Reconciliation runs in a background task so live events may still route
    /// while this scan is active. Channel checkpoints are marked unreconciled
    /// before scanning so normal live processing does not advance them until
    /// each channel is finalized. A single-flight gate prevents overlapping
    /// reconciliation tasks during reconnect storms.
    ///
    /// 1. Snapshot channel checkpoints from DB (timestamps won't move during
    ///    reconciliation because `is_reconciled` is set to `false`).
    /// 2. Scan each checkpointed channel for posts missed during downtime.
    /// 3. Replay each missed post through the normal post routing path.
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

        // Scan each checkpointed channel for posts missed during downtime.
        // Reconciliation is proportional to missed posts, not tracked threads:
        // old closed/stale/debug threads do not spawn actors unless a new
        // message for that specific thread is replayed.
        if !checkpoints.is_empty() {
            tracing::info!(
                channels = checkpoints.len(),
                "Scanning channels for missed posts"
            );

            for cp in &checkpoints {
                self.reconcile_channel(config, cp).await;
            }
        }
    }

    async fn reconcile_channel(&self, config: &Arc<Configuration>, checkpoint: &ChannelCheckpoint) {
        if self.reconcile_gap_exceeds_window(checkpoint) {
            self.skip_channel_backlog(config, checkpoint).await;
            return;
        }

        let (replayed_count, last_replayed) = self.replay_channel_posts(config, checkpoint).await;

        tracing::info!(
            channel_id = %checkpoint.channel_id,
            count = replayed_count,
            "Replayed missed posts"
        );

        self.finish_channel_reconciliation(checkpoint, last_replayed.as_ref())
            .await;
    }

    async fn skip_channel_backlog(
        &self,
        config: &Arc<Configuration>,
        checkpoint: &ChannelCheckpoint,
    ) {
        let now = chrono::Utc::now();
        tracing::warn!(
            channel_id = %checkpoint.channel_id,
            last_seen_post_at = %checkpoint.last_seen_post_at,
            skipped_to = %now,
            "Reconciliation gap exceeds max window, skipping backlog"
        );
        if let Err(e) = self
            .advance_skipped_reconciliation_checkpoint(config, checkpoint, now)
            .await
        {
            tracing::error!(channel_id = %checkpoint.channel_id, error = %e, "Failed to advance skipped reconciliation checkpoint");
        }
        self.mark_channel_reconciled(&checkpoint.channel_id).await;
    }

    async fn replay_channel_posts(
        &self,
        config: &Arc<Configuration>,
        checkpoint: &ChannelCheckpoint,
    ) -> (usize, Option<models::Post>) {
        let mut channel_messages = channel_messages_after(Arc::clone(config), checkpoint);
        let mut replayed_count = 0usize;
        let mut last_replayed = None;

        while let Some(result) = channel_messages.next().await {
            let post = match result {
                Ok(post) => post,
                Err(e) => {
                    tracing::error!(
                        channel_id = %checkpoint.channel_id,
                        error = %e,
                        "Failed to scan channel for missed posts"
                    );
                    break;
                }
            };
            self.route_post(&post, config).await;
            replayed_count += 1;
            last_replayed = Some(post);
        }

        (replayed_count, last_replayed)
    }

    async fn finish_channel_reconciliation(
        &self,
        checkpoint: &ChannelCheckpoint,
        last_replayed: Option<&models::Post>,
    ) {
        if let Some(post) = last_replayed {
            let post_at = post
                .create_at
                .map(ms_to_datetime)
                .unwrap_or_else(chrono::Utc::now);
            self.force_checkpoint_to_post(&checkpoint.channel_id, post, post_at)
                .await;
        }

        // Channel scanned — mark reconciled so normal flow can advance it.
        self.mark_channel_reconciled(&checkpoint.channel_id).await;
    }

    async fn mark_channel_reconciled(&self, channel_id: &str) {
        if let Err(e) = self.store.set_channel_reconciled(channel_id).await {
            tracing::error!(channel_id = %channel_id, error = %e, "Failed to mark channel reconciled");
        }
    }

    async fn reconcile_single_flight(&self, config: &Arc<Configuration>) {
        let Ok(_guard) = self.reconciliation_gate.try_lock() else {
            tracing::info!("Reconciliation already running, skipping duplicate start");
            return;
        };

        tracing::info!("Starting reconcile");
        self.reconcile(config).await;
    }

    fn reconcile_gap_exceeds_window(&self, checkpoint: &ChannelCheckpoint) -> bool {
        let Some(window) = self.config.max_reconcile_window else {
            return false;
        };

        let Ok(window_chrono) = chrono::Duration::from_std(window) else {
            return false;
        };

        chrono::Utc::now() - checkpoint.last_seen_post_at > window_chrono
    }

    async fn advance_skipped_reconciliation_checkpoint(
        &self,
        config: &Arc<Configuration>,
        checkpoint: &ChannelCheckpoint,
        skipped_to: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ThreadBotError> {
        let Some(post) = latest_channel_post(config, &checkpoint.channel_id).await? else {
            return Ok(());
        };
        let post_at = post.create_at.map(ms_to_datetime).unwrap_or(skipped_to);
        self.store
            .upsert_channel_checkpoint(&checkpoint.channel_id, &post.id, post_at)
            .await
    }

    async fn register_channel_if_needed(
        &self,
        channel_id: &str,
        post: &models::Post,
        post_at: chrono::DateTime<chrono::Utc>,
    ) {
        if let Err(e) = self
            .store
            .upsert_channel_checkpoint(channel_id, &post.id, post_at)
            .await
        {
            tracing::error!(channel_id = %channel_id, error = %e, "Failed to upsert channel checkpoint");
        }
    }

    async fn advance_channel_if_reconciled(
        &self,
        channel_id: &str,
        post: &models::Post,
        post_at: chrono::DateTime<chrono::Utc>,
    ) {
        if let Err(e) = self
            .store
            .advance_channel_checkpoint(channel_id, &post.id, post_at)
            .await
        {
            tracing::error!(channel_id = %channel_id, error = %e, "Failed to advance channel checkpoint");
        }
    }

    async fn force_checkpoint_to_post(
        &self,
        channel_id: &str,
        post: &models::Post,
        post_at: chrono::DateTime<chrono::Utc>,
    ) {
        if let Err(e) = self
            .store
            .upsert_channel_checkpoint(channel_id, &post.id, post_at)
            .await
        {
            tracing::error!(channel_id = %channel_id, error = %e, "Failed to advance reconciled channel checkpoint");
        }
    }

    // ── Actor management ─────────────────────────────────────────────────

    fn actor_spawn_config(
        &self,
        thread_id: &str,
        mm_config: &Arc<Configuration>,
    ) -> ActorSpawnConfig<H> {
        ActorSpawnConfig {
            handler: Arc::clone(&self.handler),
            store: Arc::clone(&self.store),
            mm_config: Arc::clone(mm_config),
            debounce: self.config.debounce,
            channel_buffer: self.config.channel_buffer,
            actor_idle_timeout: self.config.actor_idle_timeout,
            thread_id: thread_id.to_string(),
            bot_user_id: Arc::clone(&self.bot_user_id),
            metrics: self.metrics.clone(),
        }
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
        let actor_count = self.registry.send_shutdown_to_all().await;

        tracing::info!(actor_count, "Sending shutdown to thread actors");

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
                    plugin.reconcile_single_flight(&config).await;
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

    fn setup_cron(
        self: Arc<Self>,
        scheduler: &mut cron_tab::Cron<chrono::Utc>,
        config: Arc<Configuration>,
    ) {
        let handle = ThreadBotHandle::new(
            Arc::clone(&self.store),
            config,
            self.registry.clone(),
            Arc::clone(&self.bot_user_id),
        );
        Arc::clone(&self.handler).setup_cron(scheduler, handle);
    }
}
