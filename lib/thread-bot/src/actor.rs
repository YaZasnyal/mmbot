//! Per-thread actor with debounce and handler interruption.
//!
//! Each tracked thread gets a dedicated tokio task that processes events
//! sequentially via an mpsc channel. The handler can be interrupted mid-execution
//! if a new message or control reaction arrives — the handler future is simply
//! dropped and replaced with a pending placeholder.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::sync::{mpsc, RwLock};
use tokio::time::Instant;

use mattermost_api::apis::configuration::Configuration;
use mattermost_api::apis::{posts_api, reactions_api};
use mattermost_api::models;

use crate::error::ThreadBotError;
use crate::handler::{ThreadCloseReason, ThreadContext, ThreadEffect, ThreadHandler};
use crate::store::ThreadStore;
use crate::types::*;

// ─── Public (crate) types ────────────────────────────────────────────────────

/// Command sent to a per-thread actor via mpsc channel.
pub(crate) enum ThreadCommand {
    /// New message posted in the thread — triggers debounced handler run.
    NewMessage { post: models::Post },

    /// Control reaction on root post — effect already determined by
    /// [`ThreadHandler::on_control_reaction`].
    ControlReaction {
        effect: ThreadEffect,
        change: ReactionChange,
    },

    /// Startup reconciliation — triggers a debounced handler run to pick up
    /// messages that arrived while the bot was offline.
    Reconcile,

    /// Graceful shutdown signal.
    Shutdown,

    /// External wake — triggers a handler re-run without cancelling
    /// a currently running handler. Sent from [`ThreadBotHandle::wake_thread`].
    Wake,

    /// External close — closes the thread from outside the actor
    /// (e.g. cron job). Sent from [`ThreadBotHandle::resolve_thread`]
    /// or [`ThreadBotHandle::stop_thread`].
    Close { reason: ThreadCloseReason },
}

/// Shared context passed to the per-thread actor.
pub(crate) struct ActorCtx<H: ThreadHandler> {
    pub handler: Arc<H>,
    pub store: Arc<dyn ThreadStore>,
    pub ctx: ThreadContext,
    pub mm_config: Arc<Configuration>,
    pub debounce: Duration,
    pub thread_id: String,
    pub bot_user_id: Arc<RwLock<Option<String>>>,
}

// ─── Private types ───────────────────────────────────────────────────────────

/// Tracks whether a close effect originated from a reaction or the handler.
#[derive(Clone, Copy)]
enum CloseSource {
    Reaction,
    Handler,
}

/// Output of the boxed handler future: snapshot is returned alongside the
/// result so it can be used for effect execution and post-processing.
type HandlerResult = (Thread, Result<Vec<ThreadEffect>, ThreadBotError>);

/// A boxed, Send future that yields [`HandlerResult`].
type HandlerFuture = Pin<Box<dyn Future<Output = HandlerResult> + Send>>;

/// Create a future that never completes — used as a placeholder when no
/// handler is running.
fn pending_handler() -> HandlerFuture {
    Box::pin(std::future::pending())
}

/// Result of [`handle_reconcile`] — tells the actor what to do next.
pub(crate) enum ReconcileOutcome {
    /// Thread was closed by a control reaction found during reconciliation.
    ThreadClosed,
    /// New non-bot messages found — handler should run with this snapshot.
    RunHandler(Box<Thread>),
    /// Nothing to do.
    NoAction,
}

// ─── Actor main loop ─────────────────────────────────────────────────────────

/// Per-thread actor — runs as a tokio task, processes events sequentially.
///
/// The main loop has three branches in a single `select!`:
///
/// 1. **Commands** (highest priority, biased): new messages, control reactions,
///    shutdown. A new message cancels any running handler and restarts the
///    debounce timer.
/// 2. **Handler result**: fires when the handler future completes. The future
///    is a pending placeholder when no handler is running.
/// 3. **Debounce timer**: fires after no new messages for the configured
///    duration. Builds a snapshot and starts the handler future.
pub(crate) async fn thread_actor<H: ThreadHandler>(
    mut rx: mpsc::Receiver<ThreadCommand>,
    a: ActorCtx<H>,
) {
    let mut has_pending = false;
    let mut debounce_deadline = Instant::now() + a.debounce;
    let mut handler_fut: HandlerFuture = pending_handler();
    let mut handler_running = false;

    tracing::debug!(thread_id = %a.thread_id, "Thread actor started");

    loop {
        tokio::select! {
            biased;

            cmd = rx.recv() => {
                match cmd {
                    Some(ThreadCommand::NewMessage { post }) => {
                        let should_trigger = save_incoming_message(&a, &post).await;
                        if should_trigger {
                            has_pending = true;
                            debounce_deadline = Instant::now() + a.debounce;
                            // Cancel running handler — will re-run with fresh snapshot
                            if handler_running {
                                handler_fut = pending_handler();
                                handler_running = false;
                            }
                        }
                    }

                    Some(ThreadCommand::ControlReaction { effect, change }) => {
                        // Cancel running handler
                        if handler_running {
                            handler_fut = pending_handler();
                            handler_running = false;
                        }
                        if handle_control_reaction(&effect, &change, &a).await {
                            break;
                        }
                    }

                    Some(ThreadCommand::Reconcile) => {
                        if handler_running {
                            handler_fut = pending_handler();
                            handler_running = false;
                        }
                        match handle_reconcile(&a).await {
                            ReconcileOutcome::ThreadClosed => break,
                            ReconcileOutcome::RunHandler(snapshot) => {
                                let handler = Arc::clone(&a.handler);
                                let ctx = a.ctx.clone();
                                handler_fut = Box::pin(async move {
                                    let result =
                                        handler.handle(&snapshot, &ctx).await;
                                    (*snapshot, result)
                                });
                                handler_running = true;
                            }
                            ReconcileOutcome::NoAction => {}
                        }
                    }

                    Some(ThreadCommand::Shutdown) => {
                        tracing::info!(thread_id = %a.thread_id, "Thread actor shutting down");
                        break;
                    }

                    Some(ThreadCommand::Wake) => {
                        // Don't cancel running handler — just ensure re-run after it completes
                        has_pending = true;
                        debounce_deadline = Instant::now();
                        tracing::debug!(thread_id = %a.thread_id, "Thread woken by external trigger");
                    }

                    Some(ThreadCommand::Close { reason }) => {
                        tracing::info!(
                            thread_id = %a.thread_id,
                            reason = ?reason,
                            "Thread closed by external trigger"
                        );

                        // Running handler (if any) will be dropped when we break.

                        let target_status = match reason {
                            ThreadCloseReason::ResolvedExternally => ThreadStatus::Resolved,
                            _ => ThreadStatus::Stopped,
                        };

                        if let Err(e) = a.store
                            .update_thread_status(&a.thread_id, target_status)
                            .await
                        {
                            tracing::error!(
                                thread_id = %a.thread_id,
                                error = %e,
                                "Failed to update thread status on external close"
                            );
                        }

                        match build_thread_snapshot(&a.thread_id, &*a.store, &a.mm_config).await {
                            Ok(snapshot) => {
                                if let Err(e) = a.handler
                                    .on_thread_closed(&snapshot, reason, &a.ctx)
                                    .await
                                {
                                    tracing::error!(
                                        thread_id = %a.thread_id,
                                        reason = ?reason,
                                        error = %e,
                                        "on_thread_closed failed"
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    thread_id = %a.thread_id,
                                    error = %e,
                                    "Failed to build snapshot for on_thread_closed"
                                );
                            }
                        }

                        break;
                    }

                    None => {
                        tracing::info!(thread_id = %a.thread_id, "Thread actor channel closed");
                        break;
                    }
                }
            }

            result = &mut handler_fut, if handler_running => {
                handler_running = false;
                handler_fut = pending_handler();
                if handle_handler_result(result, &a, &mut has_pending, &mut debounce_deadline).await {
                    break;
                }
            }

            _ = tokio::time::sleep_until(debounce_deadline), if has_pending => {
                has_pending = false;

                let snapshot = match build_thread_snapshot(
                    &a.thread_id, &*a.store, &a.mm_config,
                ).await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!(
                            thread_id = %a.thread_id,
                            error = %e,
                            "Failed to build thread snapshot"
                        );
                        continue;
                    }
                };

                // Check for control reactions before running handler.
                // User might have added ✅/🛑 during the debounce window
                // but the WebSocket event hasn't arrived yet.
                if let Some(effect) = check_api_control_reactions(&a, &snapshot).await {
                    tracing::info!(
                        thread_id = %a.thread_id,
                        effect = ?effect,
                        "Control reaction found before handler run, closing thread"
                    );
                    let (should_exit, _) =
                        execute_effects(vec![effect], &snapshot, &a, CloseSource::Reaction).await;
                    if should_exit {
                        break;
                    }
                }

                // Start handler — move snapshot in, get it back with the result
                let handler = Arc::clone(&a.handler);
                let ctx = a.ctx.clone();
                handler_fut = Box::pin(async move {
                    let result = handler.handle(&snapshot, &ctx).await;
                    (snapshot, result)
                });
                handler_running = true;
            }
        }
    }

    tracing::debug!(thread_id = %a.thread_id, "Thread actor stopped");
}

// ─── Handler result processing ───────────────────────────────────────────────

/// Process the completed handler result: execute effects, update DB state.
///
/// Returns `true` if the actor should exit (thread closed).
async fn handle_handler_result<H: ThreadHandler>(
    (snapshot, result): HandlerResult,
    a: &ActorCtx<H>,
    has_pending: &mut bool,
    debounce_deadline: &mut Instant,
) -> bool {
    match result {
        Ok(effects) => {
            let (should_exit, should_reschedule) =
                execute_effects(effects, &snapshot, a, CloseSource::Handler).await;

            // Update processed position to the last message
            if let Some(last_msg) = snapshot.messages.last() {
                if let Err(e) = a
                    .store
                    .update_thread_processed(&a.thread_id, &last_msg.post_id, last_msg.created_at)
                    .await
                {
                    tracing::error!(
                        thread_id = %a.thread_id,
                        error = %e,
                        "Failed to update thread processed position"
                    );
                }
            }

            // Transition New → Active after first successful handler run
            if matches!(snapshot.info.status, ThreadStatus::New) && !should_exit {
                if let Err(e) = a
                    .store
                    .update_thread_status(&a.thread_id, ThreadStatus::Active)
                    .await
                {
                    tracing::error!(
                        thread_id = %a.thread_id,
                        error = %e,
                        "Failed to update thread status to Active"
                    );
                }
            }

            if should_reschedule && !should_exit {
                *has_pending = true;
                *debounce_deadline = Instant::now();
            }

            should_exit
        }
        Err(e) => {
            tracing::error!(
                thread_id = %a.thread_id,
                error = %e,
                "Handler failed"
            );
            false
        }
    }
}

// ─── Control reaction handling ───────────────────────────────────────────────

/// Process a control reaction. Returns `true` if the actor should exit.
async fn handle_control_reaction<H: ThreadHandler>(
    effect: &ThreadEffect,
    change: &ReactionChange,
    a: &ActorCtx<H>,
) -> bool {
    tracing::info!(
        thread_id = %a.thread_id,
        emoji = %change.emoji_name,
        effect = ?effect,
        "Processing control reaction"
    );

    let is_close = matches!(
        effect,
        ThreadEffect::MarkResolved | ThreadEffect::MarkStopped
    );

    if is_close {
        let snapshot = match build_thread_snapshot(&a.thread_id, &*a.store, &a.mm_config).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(
                    thread_id = %a.thread_id,
                    error = %e,
                    "Failed to build snapshot for control reaction"
                );
                return false;
            }
        };

        let (should_exit, _) =
            execute_effects(vec![effect.clone()], &snapshot, a, CloseSource::Reaction).await;

        return should_exit;
    }

    false
}

// ─── Reconciliation ─────────────────────────────────────────────────────────

/// Check Mattermost API for control reactions on the root post.
///
/// Fetches live reactions and checks each against
/// [`ThreadHandler::on_control_reaction`]. Returns the first closing effect
/// found (`MarkResolved` or `MarkStopped`), or `None`.
///
/// Used both before handler runs (debounce path) and during reconciliation
/// to catch reactions that arrived before the WebSocket event was processed.
async fn check_api_control_reactions<H: ThreadHandler>(
    a: &ActorCtx<H>,
    snapshot: &Thread,
) -> Option<ThreadEffect> {
    let reactions =
        match reactions_api::get_reactions(&a.mm_config, &snapshot.info.root_post_id).await {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!(
                    thread_id = %a.thread_id,
                    error = %e,
                    "Failed to fetch reactions for control check"
                );
                return None;
            }
        };

    let thread_record = match a.store.get_thread(&a.thread_id).await {
        Ok(Some(r)) => r,
        _ => return None,
    };

    for reaction in &reactions {
        if let (Some(user_id), Some(emoji_name)) = (&reaction.user_id, &reaction.emoji_name) {
            let change = ReactionChange {
                post_id: snapshot.info.root_post_id.clone(),
                user_id: user_id.clone(),
                emoji_name: emoji_name.clone(),
                action: ReactionAction::Added,
                created_at: reaction
                    .create_at
                    .map(ms_to_datetime)
                    .unwrap_or_else(Utc::now),
            };
            if let Some(effect) = a.handler.on_control_reaction(&thread_record, &change) {
                if matches!(
                    effect,
                    ThreadEffect::MarkResolved | ThreadEffect::MarkStopped
                ) {
                    return Some(effect);
                }
            }
        }
    }

    None
}

/// Reconcile a thread after (re)connection.
///
/// 1. Build snapshot from Mattermost API.
/// 2. Fetch reactions on the root post — if a control reaction (✅ / 🛑)
///    is found, close the thread immediately.
/// 3. Otherwise check for new non-bot messages and schedule a handler run.
pub(crate) async fn handle_reconcile<H: ThreadHandler>(a: &ActorCtx<H>) -> ReconcileOutcome {
    // Build snapshot
    let snapshot = match build_thread_snapshot(&a.thread_id, &*a.store, &a.mm_config).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(
                thread_id = %a.thread_id,
                error = %e,
                "Failed to build snapshot for reconciliation"
            );
            return ReconcileOutcome::NoAction;
        }
    };

    // Check control reactions on root post (may have arrived during downtime)
    if let Some(effect) = check_api_control_reactions(a, &snapshot).await {
        tracing::info!(
            thread_id = %a.thread_id,
            effect = ?effect,
            "Reconciliation: control reaction found, closing thread"
        );
        let (exit, _) = execute_effects(vec![effect], &snapshot, a, CloseSource::Reaction).await;
        if exit {
            return ReconcileOutcome::ThreadClosed;
        }
    }

    // Check for new non-bot messages
    let bot_id = a.bot_user_id.read().await;
    let has_new = snapshot
        .messages
        .iter()
        .any(|m| m.is_new && bot_id.as_deref() != Some(m.user_id.as_str()));
    drop(bot_id);

    if has_new {
        tracing::info!(
            thread_id = %a.thread_id,
            "Reconciliation: unprocessed messages found, scheduling handler"
        );
        ReconcileOutcome::RunHandler(Box::new(snapshot))
    } else {
        tracing::debug!(thread_id = %a.thread_id, "Reconciliation: nothing new");
        ReconcileOutcome::NoAction
    }
}

// ─── Effect execution ────────────────────────────────────────────────────────

/// Execute a list of effects returned by the handler or a control reaction.
///
/// Returns `(should_exit, should_reschedule)`.
async fn execute_effects<H: ThreadHandler>(
    effects: Vec<ThreadEffect>,
    thread: &Thread,
    a: &ActorCtx<H>,
    close_source: CloseSource,
) -> (bool, bool) {
    let mut should_exit = false;
    let mut should_reschedule = false;

    for effect in effects {
        match effect {
            ThreadEffect::Noop => {}

            ThreadEffect::Reply { message, metadata } => {
                let mut req =
                    models::CreatePostRequest::new(thread.info.channel_id.clone(), message);
                req.root_id = Some(thread.info.root_post_id.clone());
                if metadata != serde_json::Value::Null {
                    req.props = Some(metadata);
                }

                match posts_api::create_post(&a.mm_config, req, None).await {
                    Ok(created) => {
                        tracing::debug!(
                            thread_id = %a.thread_id,
                            post_id = %created.id,
                            "Posted reply"
                        );
                        // Bot reply is saved to DB when the WebSocket Posted
                        // event arrives back in handle_thread_reply.
                    }
                    Err(e) => {
                        tracing::error!(
                            thread_id = %a.thread_id,
                            error = %e,
                            "Failed to post reply"
                        );
                    }
                }
            }

            ThreadEffect::UpdateMessage {
                post_id,
                message,
                metadata,
            } => {
                let mut req = models::PatchPostRequest::new();
                req.message = message;
                if let Some(meta) = metadata {
                    req.props = Some(meta.to_string());
                }

                if let Err(e) = posts_api::patch_post(&a.mm_config, &post_id, req).await {
                    tracing::error!(
                        thread_id = %a.thread_id,
                        post_id = %post_id,
                        error = %e,
                        "Failed to update message"
                    );
                }
            }

            ThreadEffect::SetThreadMetadata { metadata } => {
                if let Err(e) = a.store.set_thread_metadata(&a.thread_id, metadata).await {
                    tracing::error!(
                        thread_id = %a.thread_id,
                        error = %e,
                        "Failed to set thread metadata"
                    );
                }
            }

            ThreadEffect::Reschedule => {
                should_reschedule = true;
            }

            ThreadEffect::MarkResolved => {
                let reason = match close_source {
                    CloseSource::Reaction => ThreadCloseReason::ResolvedByReaction,
                    CloseSource::Handler => ThreadCloseReason::ResolvedByHandler,
                };

                if let Err(e) = a
                    .store
                    .update_thread_status(&a.thread_id, ThreadStatus::Resolved)
                    .await
                {
                    tracing::error!(
                        thread_id = %a.thread_id,
                        error = %e,
                        "Failed to update thread status to Resolved"
                    );
                }

                if let Err(e) = a.handler.on_thread_closed(thread, reason, &a.ctx).await {
                    tracing::error!(
                        thread_id = %a.thread_id,
                        reason = ?reason,
                        error = %e,
                        "on_thread_closed failed"
                    );
                }

                tracing::info!(thread_id = %a.thread_id, reason = ?reason, "Thread closed");
                should_exit = true;
            }

            ThreadEffect::MarkStopped => {
                let reason = match close_source {
                    CloseSource::Reaction => ThreadCloseReason::StoppedByReaction,
                    CloseSource::Handler => ThreadCloseReason::StoppedByHandler,
                };

                if let Err(e) = a
                    .store
                    .update_thread_status(&a.thread_id, ThreadStatus::Stopped)
                    .await
                {
                    tracing::error!(
                        thread_id = %a.thread_id,
                        error = %e,
                        "Failed to update thread status to Stopped"
                    );
                }

                if let Err(e) = a.handler.on_thread_closed(thread, reason, &a.ctx).await {
                    tracing::error!(
                        thread_id = %a.thread_id,
                        reason = ?reason,
                        error = %e,
                        "on_thread_closed failed"
                    );
                }

                tracing::info!(thread_id = %a.thread_id, reason = ?reason, "Thread closed");
                should_exit = true;
            }
        }
    }

    (should_exit, should_reschedule)
}

// ─── Thread snapshot builder ─────────────────────────────────────────────────

/// Build a full [`Thread`] snapshot from Mattermost API and DB.
///
/// Each message and reaction is marked with `is_new = true` if it arrived
/// after `last_processed_post_at`.
pub async fn build_thread_snapshot(
    thread_id: &str,
    store: &dyn ThreadStore,
    mm_config: &Configuration,
) -> Result<Thread, ThreadBotError> {
    let record = store
        .get_thread(thread_id)
        .await?
        .ok_or_else(|| ThreadBotError::ThreadNotFound(thread_id.to_string()))?;

    // Get posts from Mattermost API
    let post_list = posts_api::get_post_thread(
        mm_config, thread_id, None, None, None, None, None, None, None,
    )
    .await
    .map_err(ThreadBotError::mattermost_api)?;

    let processed_at = record.last_processed_post_at;

    // Convert posts to ThreadMessages, mark is_new
    let mut messages = Vec::new();
    if let (Some(order), Some(posts)) = (&post_list.order, &post_list.posts) {
        for post_id in order {
            if let Some(post) = posts.get(post_id) {
                let mut msg = post_to_thread_message(post, thread_id);
                msg.is_new = match &processed_at {
                    Some(ts) => msg.created_at > *ts,
                    None => true, // First run — everything is new
                };
                messages.push(msg);
            }
        }
    }

    // Get reactions from DB, mark is_new
    let mut reactions = store.list_thread_reactions(thread_id).await?;
    for reaction in &mut reactions {
        reaction.is_new = match &processed_at {
            Some(ts) => reaction.created_at > *ts,
            None => true,
        };
    }

    Ok(Thread {
        info: record.into(),
        messages,
        reactions,
    })
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Save an incoming message to DB and update the thread seen position.
///
/// Returns `true` if the message should trigger a handler run (non-bot message).
async fn save_incoming_message(a: &ActorCtx<impl ThreadHandler>, post: &models::Post) -> bool {
    let bot_id = a.bot_user_id.read().await;
    let is_bot = bot_id.as_deref() == post.user_id.as_deref();
    drop(bot_id);

    let msg = UpsertThreadMessage {
        post_id: post.id.clone(),
        thread_id: a.thread_id.clone(),
        user_id: post.user_id.clone().unwrap_or_default(),
        is_bot_message: is_bot,
        root_id: post.root_id.clone(),
        parent_post_id: post.root_id.clone(),
        metadata: serde_json::Value::Null,
        post_created_at: post.create_at.map(ms_to_datetime).unwrap_or_else(Utc::now),
        post_updated_at: post.update_at.map(ms_to_datetime),
        post_deleted_at: post.delete_at.and_then(|ts| {
            if ts == 0 {
                None
            } else {
                Some(ms_to_datetime(ts))
            }
        }),
    };

    if let Err(e) = a.store.upsert_message(msg).await {
        tracing::error!(
            post_id = %post.id,
            thread_id = %a.thread_id,
            error = %e,
            "Failed to save message to DB"
        );
    }

    let seen_at = post.create_at.map(ms_to_datetime).unwrap_or_else(Utc::now);
    if let Err(e) = a
        .store
        .update_thread_seen(&a.thread_id, &post.id, seen_at)
        .await
    {
        tracing::error!(
            thread_id = %a.thread_id,
            error = %e,
            "Failed to update thread seen position"
        );
    }

    !is_bot
}

/// Convert a Mattermost API [`Post`](models::Post) to a [`ThreadMessage`].
///
/// `is_new` defaults to `false`; callers set it based on context.
pub(crate) fn post_to_thread_message(post: &models::Post, thread_id: &str) -> ThreadMessage {
    ThreadMessage {
        post_id: post.id.clone(),
        thread_id: thread_id.to_string(),
        user_id: post.user_id.clone().unwrap_or_default(),
        message: post.message.clone().unwrap_or_default(),
        root_id: post.root_id.clone(),
        parent_post_id: post.root_id.clone(),
        props: post.props.clone().unwrap_or(serde_json::Value::Null),
        metadata: serde_json::Value::Null,
        created_at: post.create_at.map(ms_to_datetime).unwrap_or_default(),
        updated_at: post.update_at.map(ms_to_datetime).unwrap_or_default(),
        is_new: false,
    }
}

/// Convert a millisecond Unix timestamp to a [`DateTime<Utc>`].
pub(crate) fn ms_to_datetime(ms: i64) -> DateTime<Utc> {
    DateTime::from_timestamp_millis(ms).unwrap_or_default()
}

/// Check if a post is a root post (not a reply).
pub(crate) fn is_root_post(post: &models::Post) -> bool {
    !matches!(&post.root_id, Some(root_id) if !root_id.is_empty())
}

/// Get the root post ID from a post.
///
/// Returns the post's own ID if it's a root post, or its `root_id` if it's a reply.
pub(crate) fn get_root_post_id(post: &models::Post) -> &str {
    match &post.root_id {
        Some(root_id) if !root_id.is_empty() => root_id,
        _ => &post.id,
    }
}
