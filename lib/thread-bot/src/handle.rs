//! [`ThreadBotHandle`] — facade for accessing thread-bot from cron jobs.
//!
//! Provides read access to thread data via [`ThreadStore`] and controlled
//! write access via actor commands (`Wake`, `Close`).

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::{mpsc, Mutex, RwLock};

use mattermost_api::apis::configuration::Configuration;

use crate::actor::{build_thread_snapshot, ThreadCommand};
use crate::error::ThreadBotError;
use crate::handler::ThreadCloseReason;
use crate::store::ThreadStore;
use crate::types::*;

/// Facade for accessing thread-bot state from cron jobs and external triggers.
///
/// Provides read access to thread data and controlled write access
/// via actor commands. `Clone`-friendly for capture in cron closures.
#[derive(Clone)]
pub struct ThreadBotHandle {
    store: Arc<dyn ThreadStore>,
    config: Arc<Configuration>,
    actors: Arc<Mutex<HashMap<String, mpsc::Sender<ThreadCommand>>>>,
    bot_user_id: Arc<RwLock<Option<String>>>,
}

impl ThreadBotHandle {
    /// Create a new handle sharing state with a `ThreadBotPlugin`.
    pub(crate) fn new(
        store: Arc<dyn ThreadStore>,
        config: Arc<Configuration>,
        actors: Arc<Mutex<HashMap<String, mpsc::Sender<ThreadCommand>>>>,
        bot_user_id: Arc<RwLock<Option<String>>>,
    ) -> Self {
        Self {
            store,
            config,
            actors,
            bot_user_id,
        }
    }

    // ── Read operations (delegate to ThreadStore) ────────────────────────

    /// List threads by status, optionally filtered by `updated_at` range.
    ///
    /// Pass `None` for time bounds to skip filtering.
    pub async fn list_threads(
        &self,
        statuses: &[ThreadStatus],
        updated_after: Option<DateTime<Utc>>,
        updated_before: Option<DateTime<Utc>>,
    ) -> Result<Vec<ThreadRecord>, ThreadBotError> {
        self.store
            .list_threads_by_status(statuses, updated_after, updated_before)
            .await
    }

    /// Get a single thread record (metadata + cursors, no messages).
    pub async fn get_thread(
        &self,
        thread_id: &str,
    ) -> Result<Option<ThreadRecord>, ThreadBotError> {
        self.store.get_thread(thread_id).await
    }

    /// Get message records from DB for a thread.
    ///
    /// Returns `ThreadMessageRecord` (references + metadata from Postgres),
    /// NOT full messages from Mattermost API. For full messages use
    /// [`build_thread_snapshot`](Self::build_thread_snapshot).
    pub async fn get_thread_messages(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadMessageRecord>, ThreadBotError> {
        self.store.list_thread_messages(thread_id).await
    }

    /// Get reactions from DB for a thread.
    pub async fn get_thread_reactions(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadReaction>, ThreadBotError> {
        self.store.list_thread_reactions(thread_id).await
    }

    // ── Snapshot (Mattermost API + DB) ───────────────────────────────────

    /// Build a full thread snapshot with live messages from Mattermost API.
    ///
    /// **Expensive**: makes HTTP calls. Use for report generation.
    /// For simple checks (timeout, status) prefer [`get_thread`](Self::get_thread).
    pub async fn build_thread_snapshot(&self, thread_id: &str) -> Result<Thread, ThreadBotError> {
        build_thread_snapshot(thread_id, &*self.store, &self.config).await
    }

    // ── Actor commands ──────────────────────────────────────────────────

    /// Wake a thread's actor — trigger a handler re-invocation.
    ///
    /// Does NOT cancel a running handler. Sets the pending flag so the
    /// handler runs again after the current invocation completes.
    ///
    /// Returns `Ok(true)` if the command was sent, `Ok(false)` if no
    /// actor exists (thread may be closed or not yet reconciled).
    pub async fn wake_thread(&self, thread_id: &str) -> Result<bool, ThreadBotError> {
        let actors = self.actors.lock().await;
        if let Some(tx) = actors.get(thread_id) {
            if !tx.is_closed() {
                let _ = tx.send(ThreadCommand::Wake).await;
                return Ok(true);
            }
        }
        tracing::debug!(
            thread_id = %thread_id,
            "wake_thread: no actor found"
        );
        Ok(false)
    }

    /// Close thread as Resolved via its actor.
    ///
    /// If actor exists: sends `Close` command (actor calls `on_thread_closed`,
    /// updates DB, shuts down). If no actor: direct DB status update
    /// (`on_thread_closed` is **not** called).
    pub async fn resolve_thread(&self, thread_id: &str) -> Result<(), ThreadBotError> {
        self.close_thread_impl(thread_id, ThreadCloseReason::ResolvedExternally)
            .await
    }

    /// Close thread as Stopped via its actor.
    ///
    /// Same semantics as [`resolve_thread`](Self::resolve_thread) but
    /// with `Stopped` status and `StoppedExternally` reason.
    pub async fn stop_thread(&self, thread_id: &str) -> Result<(), ThreadBotError> {
        self.close_thread_impl(thread_id, ThreadCloseReason::StoppedExternally)
            .await
    }

    async fn close_thread_impl(
        &self,
        thread_id: &str,
        reason: ThreadCloseReason,
    ) -> Result<(), ThreadBotError> {
        // Try sending to actor first
        {
            let actors = self.actors.lock().await;
            if let Some(tx) = actors.get(thread_id) {
                if !tx.is_closed() {
                    let _ = tx.send(ThreadCommand::Close { reason }).await;
                    return Ok(());
                }
            }
        }

        // No actor — direct DB update
        let target_status = match reason {
            ThreadCloseReason::ResolvedExternally => ThreadStatus::Resolved,
            _ => ThreadStatus::Stopped,
        };

        tracing::warn!(
            thread_id = %thread_id,
            reason = ?reason,
            "Closing thread without actor — on_thread_closed will NOT be called"
        );

        self.store
            .update_thread_status(thread_id, target_status)
            .await
    }

    // ── Direct store writes ─────────────────────────────────────────────

    /// Set thread metadata (full JSONB replacement).
    ///
    /// Direct DB write, no actor involved. Safe for handler-owned data.
    pub async fn set_thread_metadata(
        &self,
        thread_id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        self.store.set_thread_metadata(thread_id, metadata).await
    }

    // ── Getters ─────────────────────────────────────────────────────────

    /// Mattermost API configuration (for direct API calls from cron).
    pub fn config(&self) -> &Arc<Configuration> {
        &self.config
    }

    /// Bot's own user ID (if resolved during `on_start`).
    pub fn bot_user_id(&self) -> Option<String> {
        self.bot_user_id
            .try_read()
            .ok()
            .and_then(|guard| guard.clone())
    }
}
