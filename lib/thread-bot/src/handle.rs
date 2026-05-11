//! [`ThreadBotHandle`] — facade for accessing thread-bot from cron jobs.
//!
//! Provides read access to thread data via [`ThreadStore`] and controlled
//! write access via actor commands (`Wake`).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use mattermost_api::apis::configuration::Configuration;

use crate::actor::{build_thread_snapshot, ThreadCommand};
use crate::actor_registry::ActorRegistry;
use crate::error::ThreadBotError;
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
    registry: ActorRegistry,
    bot_user_id: Arc<RwLock<Option<String>>>,
}

impl ThreadBotHandle {
    /// Create a new handle sharing state with a `ThreadBotPlugin`.
    pub(crate) fn new(
        store: Arc<dyn ThreadStore>,
        config: Arc<Configuration>,
        registry: ActorRegistry,
        bot_user_id: Arc<RwLock<Option<String>>>,
    ) -> Self {
        Self {
            store,
            config,
            registry,
            bot_user_id,
        }
    }

    // ── Read operations (delegate to ThreadStore) ────────────────────────

    /// List tracked threads, optionally filtered by `updated_at` range.
    ///
    /// Pass `None` for time bounds to skip filtering.
    pub async fn list_threads(
        &self,
        updated_after: Option<DateTime<Utc>>,
        updated_before: Option<DateTime<Utc>>,
    ) -> Result<Vec<ThreadRecord>, ThreadBotError> {
        self.store.list_threads(updated_after, updated_before).await
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
    /// actor exists.
    pub async fn wake_thread(&self, thread_id: &str) -> Result<bool, ThreadBotError> {
        let sent = self
            .registry
            .send_if_active(thread_id, ThreadCommand::Wake)
            .await?;
        if !sent {
            tracing::debug!(
                thread_id = %thread_id,
                "wake_thread: no actor found"
            );
        }
        Ok(sent)
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
