//! Per-thread actor with debounce and handler interruption.
//!
//! Each tracked thread gets a dedicated tokio task that processes events
//! sequentially via an mpsc channel. The handler can be interrupted mid-execution
//! if a new message or control reaction arrives — the handler future is simply
//! dropped and replaced with a pending placeholder.

use std::collections::HashMap;
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
use crate::metrics::ThreadBotMetricsHandle;
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
    pub actor_idle_timeout: Duration,
    pub thread_id: String,
    pub bot_user_id: Arc<RwLock<Option<String>>>,
    pub metrics: ThreadBotMetricsHandle,
}

// ─── Private types ───────────────────────────────────────────────────────────

/// Tracks whether a close effect originated from a reaction or the handler.
#[derive(Clone, Copy)]
enum CloseSource {
    Reaction,
    Handler,
}

/// Output of the boxed handler future: snapshot is returned alongside the
/// measured handler duration and result so it can be used for post-processing.
type HandlerResult = (Thread, Duration, Result<Vec<ThreadEffect>, ThreadBotError>);

/// A boxed, Send future that yields [`HandlerResult`].
type HandlerFuture = Pin<Box<dyn Future<Output = HandlerResult> + Send>>;

/// Create a future that never completes — used as a placeholder when no
/// handler is running.
fn pending_handler() -> HandlerFuture {
    Box::pin(std::future::pending())
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
    rx: &mut mpsc::Receiver<ThreadCommand>,
    a: &ActorCtx<H>,
) {
    let mut has_pending = false;
    let mut debounce_deadline = Instant::now() + a.debounce;
    let mut handler_fut: HandlerFuture = pending_handler();
    let mut handler_running = false;
    let mut idle_deadline = None;

    tracing::debug!(thread_id = %a.thread_id, "Thread actor started");
    a.metrics.actor_started("success");
    let mut stop_reason = "channel_closed";

    loop {
        tokio::select! {
            biased;

            cmd = rx.recv() => {
                idle_deadline = None;
                match cmd {
                    Some(ThreadCommand::NewMessage { post }) => {
                        a.metrics.actor_command("new_message", "received");
                        let should_trigger = save_incoming_message(a, &post).await;
                        if should_trigger {
                            has_pending = true;
                            debounce_deadline = Instant::now() + a.debounce;
                            // Cancel running handler — will re-run with fresh snapshot
                            if handler_running {
                                handler_fut = pending_handler();
                                handler_running = false;
                            }
                        }
                        if !has_pending && !handler_running {
                            idle_deadline = Some(next_idle_deadline(a.actor_idle_timeout));
                        }
                    }

                    Some(ThreadCommand::ControlReaction { effect, change }) => {
                        a.metrics.actor_command("control_reaction", "received");
                        // Cancel running handler
                        if handler_running {
                            handler_fut = pending_handler();
                            handler_running = false;
                        }
                        if handle_control_reaction(&effect, &change, a).await {
                            stop_reason = "control_reaction";
                            break;
                        }
                        if !has_pending && !handler_running {
                            idle_deadline = Some(next_idle_deadline(a.actor_idle_timeout));
                        }
                    }

                    Some(ThreadCommand::Shutdown) => {
                        a.metrics.actor_command("shutdown", "received");
                        tracing::info!(thread_id = %a.thread_id, "Thread actor shutting down");
                        stop_reason = "shutdown";
                        break;
                    }

                    Some(ThreadCommand::Wake) => {
                        a.metrics.actor_command("wake", "received");
                        // Don't cancel running handler — just ensure re-run after it completes
                        has_pending = true;
                        debounce_deadline = Instant::now();
                        tracing::debug!(thread_id = %a.thread_id, "Thread woken by external trigger");
                    }

                    Some(ThreadCommand::Close { reason }) => {
                        a.metrics.actor_command("close", "received");
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

                        stop_reason = thread_close_reason_label(reason);
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
                if handle_handler_result(result, a, &mut has_pending, &mut debounce_deadline).await {
                    stop_reason = "handler_closed";
                    break;
                }
                if !has_pending {
                    idle_deadline = Some(next_idle_deadline(a.actor_idle_timeout));
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
                        idle_deadline = Some(next_idle_deadline(a.actor_idle_timeout));
                        continue;
                    }
                };

                // Check for control reactions before running handler.
                // User might have added ✅/🛑 during the debounce window
                // but the WebSocket event hasn't arrived yet.
                if let Some(effect) = check_api_control_reactions(a, &snapshot).await {
                    tracing::info!(
                        thread_id = %a.thread_id,
                        effect = ?effect,
                        "Control reaction found before handler run, closing thread"
                    );
                    let (should_exit, _) =
                        execute_effects(vec![effect], &snapshot, a, CloseSource::Reaction).await;
                    if should_exit {
                        break;
                    }
                }

                // Start handler — move snapshot in, get it back with the result
                let handler = Arc::clone(&a.handler);
                let ctx = a.ctx.clone();
                handler_fut = Box::pin(async move {
                    let started = std::time::Instant::now();
                    let result = handler.handle(&snapshot, &ctx).await;
                    (snapshot, started.elapsed(), result)
                });
                handler_running = true;
            }

            _ = idle_sleep_until(idle_deadline), if !has_pending && !handler_running => {
                tracing::debug!(thread_id = %a.thread_id, "Thread actor idle timeout elapsed");
                stop_reason = "idle_timeout";
                break;
            }
        }
    }

    tracing::debug!(thread_id = %a.thread_id, "Thread actor stopped");
    a.metrics.actor_stopped(stop_reason);
}

fn next_idle_deadline(timeout: Duration) -> Instant {
    Instant::now() + timeout
}

async fn idle_sleep_until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}

// ─── Handler result processing ───────────────────────────────────────────────

/// Process the completed handler result: execute effects, update DB state.
///
/// Returns `true` if the actor should exit (thread closed).
async fn handle_handler_result<H: ThreadHandler>(
    (snapshot, handler_duration, result): HandlerResult,
    a: &ActorCtx<H>,
    has_pending: &mut bool,
    debounce_deadline: &mut Instant,
) -> bool {
    match result {
        Ok(effects) => {
            let effect_count = effects.len();
            let (should_exit, should_reschedule) =
                execute_effects(effects, &snapshot, a, CloseSource::Handler).await;
            let outcome = if should_exit {
                "closed"
            } else if should_reschedule {
                "rescheduled"
            } else {
                "success"
            };
            a.metrics.handler_run(outcome, handler_duration);
            if effect_count == 0 {
                a.metrics.effect("none", "success");
            }

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
            a.metrics.handler_run("error", handler_duration);
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

// ─── Control reaction checks ────────────────────────────────────────────────

/// Check Mattermost API for control reactions on the root post.
///
/// Fetches live reactions and checks each against
/// [`ThreadHandler::on_control_reaction`]. Returns the first closing effect
/// found (`MarkResolved` or `MarkStopped`), or `None`.
///
/// Used before handler runs to catch reactions that arrived before the
/// WebSocket event was processed.
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
// ─── Effect execution ────────────────────────────────────────────────────────

/// Execute a list of effects returned by the handler or a control reaction.
///
/// Returns `(should_exit, should_reschedule)`.
#[tracing::instrument(level = "debug", skip_all, fields(thread_id = %a.thread_id))]
async fn execute_effects<H: ThreadHandler>(
    effects: Vec<ThreadEffect>,
    thread: &Thread,
    a: &ActorCtx<H>,
    close_source: CloseSource,
) -> (bool, bool) {
    let mut should_exit = false;
    let mut should_reschedule = false;
    let total_effects = effects.len();

    for (effect_idx, effect) in effects.into_iter().enumerate() {
        let effect_label = thread_effect_label(&effect);
        tracing::info!(
            thread_id = %a.thread_id,
            root_post_id = %thread.info.root_post_id,
            effect = effect_label,
            effect_idx,
            total_effects,
            "Executing thread effect"
        );
        match effect {
            ThreadEffect::Noop => {
                a.metrics.effect(effect_label, "success");
            }

            ThreadEffect::Reply { message, metadata } => {
                let message_bytes = message.len();
                let metadata_bytes = if metadata != serde_json::Value::Null {
                    metadata.to_string().len()
                } else {
                    0
                };
                let mut req =
                    models::CreatePostRequest::new(thread.info.channel_id.clone(), message);
                req.root_id = Some(thread.info.root_post_id.clone());
                if metadata != serde_json::Value::Null {
                    req.props = Some(metadata);
                }

                match posts_api::create_post(&a.mm_config, req, None).await {
                    Ok(created) => {
                        a.metrics.effect(effect_label, "success");
                        tracing::debug!(
                            thread_id = %a.thread_id,
                            post_id = %created.id,
                            message_bytes,
                            metadata_bytes,
                            "Posted reply"
                        );
                        // Bot reply is saved to DB when the WebSocket Posted
                        // event arrives back in handle_thread_reply.
                    }
                    Err(e) => {
                        a.metrics.effect(effect_label, "error");
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
                    a.metrics.effect(effect_label, "error");
                    tracing::error!(
                        thread_id = %a.thread_id,
                        post_id = %post_id,
                        error = %e,
                        "Failed to update message"
                    );
                } else {
                    a.metrics.effect(effect_label, "success");
                }
            }

            ThreadEffect::SetThreadMetadata { metadata } => {
                tracing::info!(
                    thread_id = %a.thread_id,
                    metadata_key = "thread",
                    metadata_bytes = metadata.to_string().len(),
                    "Persisting thread metadata"
                );
                if let Err(e) = a.store.set_thread_metadata(&a.thread_id, metadata).await {
                    a.metrics.effect(effect_label, "error");
                    tracing::error!(
                        thread_id = %a.thread_id,
                        effect_type = "set_thread_metadata",
                        error = %e,
                        "Critical thread effect failed"
                    );
                    stop_after_critical_effect_failure(thread, a, "set_thread_metadata").await;
                    should_exit = true;
                    break;
                } else {
                    a.metrics.effect(effect_label, "success");
                }
            }

            ThreadEffect::SetMessageMetadata { post_id, metadata } => {
                tracing::info!(
                    thread_id = %a.thread_id,
                    post_id = %post_id,
                    metadata_key = "message",
                    metadata_bytes = metadata.to_string().len(),
                    "Persisting message metadata"
                );
                if let Err(e) = a.store.set_message_metadata(&post_id, metadata).await {
                    a.metrics.effect(effect_label, "error");
                    tracing::error!(
                        thread_id = %a.thread_id,
                        post_id = %post_id,
                        effect_type = "set_message_metadata",
                        error = %e,
                        "Critical thread effect failed"
                    );
                    stop_after_critical_effect_failure(thread, a, "set_message_metadata").await;
                    should_exit = true;
                    break;
                } else {
                    a.metrics.effect(effect_label, "success");
                }
            }

            ThreadEffect::Reschedule => {
                a.metrics.effect(effect_label, "success");
                should_reschedule = true;
            }

            ThreadEffect::MarkResolved => {
                if effect_idx < total_effects.saturating_sub(1) {
                    tracing::warn!(
                        thread_id = %a.thread_id,
                        effect_idx,
                        total_effects,
                        "MarkResolved executed before final effect in batch"
                    );
                }
                let reason = match close_source {
                    CloseSource::Reaction => ThreadCloseReason::ResolvedByReaction,
                    CloseSource::Handler => ThreadCloseReason::ResolvedByHandler,
                };

                if let Err(e) = a
                    .store
                    .update_thread_status(&a.thread_id, ThreadStatus::Resolved)
                    .await
                {
                    a.metrics.effect(effect_label, "error");
                    tracing::error!(
                        thread_id = %a.thread_id,
                        error = %e,
                        "Failed to update thread status to Resolved"
                    );
                } else {
                    a.metrics.effect(effect_label, "success");
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
                if effect_idx < total_effects.saturating_sub(1) {
                    tracing::warn!(
                        thread_id = %a.thread_id,
                        effect_idx,
                        total_effects,
                        "MarkStopped executed before final effect in batch"
                    );
                }
                let reason = match close_source {
                    CloseSource::Reaction => ThreadCloseReason::StoppedByReaction,
                    CloseSource::Handler => ThreadCloseReason::StoppedByHandler,
                };

                if let Err(e) = a
                    .store
                    .update_thread_status(&a.thread_id, ThreadStatus::Stopped)
                    .await
                {
                    a.metrics.effect(effect_label, "error");
                    tracing::error!(
                        thread_id = %a.thread_id,
                        error = %e,
                        "Failed to update thread status to Stopped"
                    );
                } else {
                    a.metrics.effect(effect_label, "success");
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

fn thread_effect_label(effect: &ThreadEffect) -> &'static str {
    match effect {
        ThreadEffect::Noop => "noop",
        ThreadEffect::Reply { .. } => "reply",
        ThreadEffect::UpdateMessage { .. } => "update_message",
        ThreadEffect::SetThreadMetadata { .. } => "set_thread_metadata",
        ThreadEffect::SetMessageMetadata { .. } => "set_message_metadata",
        ThreadEffect::Reschedule => "reschedule",
        ThreadEffect::MarkResolved => "mark_resolved",
        ThreadEffect::MarkStopped => "mark_stopped",
    }
}

fn thread_close_reason_label(reason: ThreadCloseReason) -> &'static str {
    match reason {
        ThreadCloseReason::ResolvedByReaction => "resolved_by_reaction",
        ThreadCloseReason::StoppedByReaction => "stopped_by_reaction",
        ThreadCloseReason::ResolvedByHandler => "resolved_by_handler",
        ThreadCloseReason::StoppedByHandler => "stopped_by_handler",
        ThreadCloseReason::ResolvedExternally => "resolved_externally",
        ThreadCloseReason::StoppedExternally => "stopped_externally",
    }
}

async fn stop_after_critical_effect_failure<H: ThreadHandler>(
    thread: &Thread,
    a: &ActorCtx<H>,
    effect_type: &'static str,
) {
    if let Err(e) = a
        .store
        .update_thread_status(&a.thread_id, ThreadStatus::Stopped)
        .await
    {
        tracing::error!(
            thread_id = %a.thread_id,
            effect_type,
            error = %e,
            "Failed to stop thread after critical effect failure"
        );
        return;
    }

    if let Err(e) = a
        .handler
        .on_thread_closed(thread, ThreadCloseReason::StoppedByHandler, &a.ctx)
        .await
    {
        tracing::error!(
            thread_id = %a.thread_id,
            effect_type,
            reason = ?ThreadCloseReason::StoppedByHandler,
            error = %e,
            "on_thread_closed failed after critical effect failure"
        );
    }

    tracing::info!(
        thread_id = %a.thread_id,
        effect_type,
        reason = ?ThreadCloseReason::StoppedByHandler,
        "Thread stopped after critical effect failure"
    );
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
    let message_records = store
        .list_thread_messages(thread_id)
        .await?
        .into_iter()
        .map(|message| (message.post_id.clone(), message))
        .collect::<HashMap<_, _>>();

    // Convert posts to ThreadMessages, mark is_new
    let mut messages = Vec::new();
    if let (Some(order), Some(posts)) = (&post_list.order, &post_list.posts) {
        for post_id in order {
            if let Some(post) = posts.get(post_id) {
                let mut msg = post_to_thread_message(post, thread_id);
                if let Some(record) = message_records.get(&msg.post_id) {
                    msg.metadata = record.metadata.clone();
                }
                msg.is_new = match &processed_at {
                    Some(ts) => msg.created_at > *ts,
                    None => true, // First run — everything is new
                };
                messages.push(msg);
            }
        }
    }
    messages.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.post_id.cmp(&right.post_id))
    });

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
#[tracing::instrument(
    level = "debug",
    skip_all,
    fields(thread_id = %a.thread_id, post_id = %post.id)
)]
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
        metadata: post.props.clone().unwrap_or(serde_json::Value::Null),
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
    tracing::info!(
        thread_id = %a.thread_id,
        post_id = %post.id,
        user_id = %post.user_id.clone().unwrap_or_default(),
        is_bot,
        message_bytes = post.message.as_ref().map_or(0, |m| m.len()),
        "Incoming thread message persisted"
    );

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
