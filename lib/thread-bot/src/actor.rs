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
use mattermost_api::apis::posts_api;
use mattermost_api::models;

use crate::error::ThreadBotError;
use crate::handler::{ThreadContext, ThreadEffect, ThreadHandler, ThreadTarget};
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

/// Output of the boxed handler future: invocation is returned alongside the
/// measured handler duration and result so it can be used for post-processing.
type HandlerResult = (
    ThreadInvocation,
    Duration,
    Result<Vec<ThreadEffect>, ThreadBotError>,
);

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
    let mut pending_run: Option<ThreadTrigger> = None;
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
                            pending_run = Some(ThreadTrigger::NewMessage {
                                post_id: post.id.clone(),
                            });
                            debounce_deadline = Instant::now() + a.debounce;
                            // Cancel running handler — will re-run with fresh snapshot
                            if handler_running {
                                handler_fut = pending_handler();
                                handler_running = false;
                            }
                        }
                        if pending_run.is_none() && !handler_running {
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
                        if pending_run.is_none() && !handler_running {
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
                        pending_run.get_or_insert(ThreadTrigger::Wake);
                        debounce_deadline = Instant::now();
                        tracing::debug!(thread_id = %a.thread_id, "Thread woken by external trigger");
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
                if handle_handler_result(result, a, &mut pending_run, &mut debounce_deadline).await {
                    stop_reason = "handler_closed";
                    break;
                }
                if pending_run.is_none() {
                    idle_deadline = Some(next_idle_deadline(a.actor_idle_timeout));
                }
            }

            _ = tokio::time::sleep_until(debounce_deadline), if pending_run.is_some() => {
                let trigger = pending_run
                    .take()
                    .expect("pending_run is Some when debounce branch is enabled");

                let thread_record = match a.store.get_thread(&a.thread_id).await {
                    Ok(Some(record)) => record,
                    Ok(None) => {
                        tracing::error!(
                            thread_id = %a.thread_id,
                            "Cannot invoke handler for missing thread record"
                        );
                        idle_deadline = Some(next_idle_deadline(a.actor_idle_timeout));
                        continue;
                    }
                    Err(e) => {
                        tracing::error!(
                            thread_id = %a.thread_id,
                            error = %e,
                            "Failed to load thread record for handler invocation"
                        );
                        idle_deadline = Some(next_idle_deadline(a.actor_idle_timeout));
                        continue;
                    }
                };
                let invocation = ThreadInvocation {
                    thread: thread_record,
                    trigger,
                };

                // Start handler — move invocation in, get it back with the result
                let handler = Arc::clone(&a.handler);
                let ctx = a.ctx.clone();
                handler_fut = Box::pin(async move {
                    let started = std::time::Instant::now();
                    let result = handler.handle(&invocation, &ctx).await;
                    (invocation, started.elapsed(), result)
                });
                handler_running = true;
            }

            _ = idle_sleep_until(idle_deadline), if pending_run.is_none() && !handler_running => {
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
    (invocation, handler_duration, result): HandlerResult,
    a: &ActorCtx<H>,
    pending_run: &mut Option<ThreadTrigger>,
    debounce_deadline: &mut Instant,
) -> bool {
    match result {
        Ok(effects) => {
            let effect_count = effects.len();
            let (should_exit, should_reschedule) =
                execute_effects(effects, &invocation.thread, a).await;
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

            // Mark the invocation's seen cursor as processed. Messages that
            // arrive during a running handler cancel it or schedule a follow-up
            // run, so they are handled by a later invocation.
            if let (Some(last_seen_post_id), Some(last_seen_post_at)) = (
                invocation.thread.last_seen_post_id.as_deref(),
                invocation.thread.last_seen_post_at,
            ) {
                if let Err(e) = a
                    .store
                    .update_thread_processed(&a.thread_id, last_seen_post_id, last_seen_post_at)
                    .await
                {
                    tracing::error!(
                        thread_id = %a.thread_id,
                        error = %e,
                        "Failed to update thread processed position"
                    );
                }
            }

            if should_reschedule && !should_exit {
                *pending_run = Some(ThreadTrigger::Reschedule);
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

    let thread_record = match a.store.get_thread(&a.thread_id).await {
        Ok(Some(record)) => record,
        Ok(None) => {
            tracing::error!(
                thread_id = %a.thread_id,
                "Cannot execute control reaction for missing thread record"
            );
            return false;
        }
        Err(e) => {
            tracing::error!(
                thread_id = %a.thread_id,
                error = %e,
                "Failed to load thread record for control reaction"
            );
            return false;
        }
    };

    let (should_exit, _) = execute_effects(vec![effect.clone()], &thread_record, a).await;

    should_exit
}

// ─── Effect execution ────────────────────────────────────────────────────────

/// Execute a list of effects returned by the handler or a control reaction.
///
/// Returns `(should_exit, should_reschedule)`.
#[tracing::instrument(level = "debug", skip_all, fields(thread_id = %a.thread_id))]
async fn execute_effects<H: ThreadHandler>(
    effects: Vec<ThreadEffect>,
    thread: &ThreadRecord,
    a: &ActorCtx<H>,
) -> (bool, bool) {
    let mut should_exit = false;
    let mut should_reschedule = false;
    let total_effects = effects.len();

    for (effect_idx, effect) in effects.into_iter().enumerate() {
        let effect_label = thread_effect_label(&effect);
        tracing::info!(
            thread_id = %a.thread_id,
            root_post_id = %thread.root_post_id,
            effect = effect_label,
            effect_idx,
            total_effects,
            "Executing thread effect"
        );
        match effect {
            ThreadEffect::Noop => {
                a.metrics.effect(effect_label, "success");
            }

            ThreadEffect::Reply {
                target,
                message,
                metadata,
            } => {
                let target_threads = match resolve_thread_target(thread, a, target).await {
                    Ok(target_threads) => target_threads,
                    Err(error) => {
                        a.metrics.effect(effect_label, "error");
                        tracing::error!(
                            thread_id = %a.thread_id,
                            error = %error,
                            "Failed to resolve reply target"
                        );
                        stop_after_critical_effect_failure(thread, a, "reply");
                        should_exit = true;
                        break;
                    }
                };

                let mut posted_all = true;
                for target_thread in target_threads {
                    match post_reply_to_thread(&target_thread, a, message.clone(), metadata.clone())
                        .await
                    {
                        Ok(created_post_id) => {
                            tracing::info!(
                                thread_id = %a.thread_id,
                                target_thread_id = %target_thread.thread_id,
                                post_id = %created_post_id,
                                "Posted targeted reply"
                            );
                        }
                        Err(error) => {
                            posted_all = false;
                            tracing::error!(
                                thread_id = %a.thread_id,
                                target_thread_id = %target_thread.thread_id,
                                error = %error,
                                "Failed to post targeted reply"
                            );
                        }
                    }
                }
                a.metrics
                    .effect(effect_label, if posted_all { "success" } else { "error" });
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
                    stop_after_critical_effect_failure(thread, a, "set_thread_metadata");
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
                    stop_after_critical_effect_failure(thread, a, "set_message_metadata");
                    should_exit = true;
                    break;
                } else {
                    a.metrics.effect(effect_label, "success");
                }
            }

            ThreadEffect::EnsureLinkedThread {
                link_kind,
                channel_id,
                thread_kind,
                message,
                metadata,
                thread_metadata,
                link_metadata,
            } => {
                let result = ensure_linked_thread(
                    thread,
                    a,
                    EnsureLinkedThreadRequest {
                        link_kind,
                        channel_id,
                        thread_kind,
                        message,
                        metadata,
                        thread_metadata,
                        link_metadata,
                    },
                )
                .await;
                match result {
                    Ok(()) => {
                        a.metrics.effect(effect_label, "success");
                    }
                    Err(error) => {
                        a.metrics.effect(effect_label, "error");
                        tracing::error!(
                            thread_id = %a.thread_id,
                            error = %error,
                            "Critical linked thread effect failed"
                        );
                        stop_after_critical_effect_failure(thread, a, "ensure_linked_thread");
                        should_exit = true;
                        break;
                    }
                }
            }

            ThreadEffect::Reschedule => {
                a.metrics.effect(effect_label, "success");
                should_reschedule = true;
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
        ThreadEffect::EnsureLinkedThread { .. } => "ensure_linked_thread",
        ThreadEffect::Reschedule => "reschedule",
    }
}

struct EnsureLinkedThreadRequest {
    link_kind: String,
    channel_id: String,
    thread_kind: Option<String>,
    message: String,
    metadata: serde_json::Value,
    thread_metadata: serde_json::Value,
    link_metadata: serde_json::Value,
}

async fn resolve_thread_target<H: ThreadHandler>(
    current_thread: &ThreadRecord,
    a: &ActorCtx<H>,
    target: ThreadTarget,
) -> Result<Vec<ThreadRecord>, ThreadBotError> {
    match target {
        ThreadTarget::CurrentThread => Ok(vec![current_thread.clone()]),
        ThreadTarget::Thread { thread_id } => {
            let thread = a
                .store
                .get_thread(&thread_id)
                .await?
                .ok_or_else(|| ThreadBotError::ThreadNotFound(thread_id))?;
            Ok(vec![thread])
        }
        ThreadTarget::LinkedThreads { link_kind } => {
            let links = a.store.list_thread_links(&current_thread.thread_id).await?;
            let linked_thread_ids = links
                .into_iter()
                .filter(|link| link.link_kind == link_kind)
                .map(|link| link.target_thread_id)
                .collect::<Vec<_>>();
            if linked_thread_ids.is_empty() {
                return Err(ThreadBotError::InvalidState(format!(
                    "linked threads not found: source_thread_id={}, link_kind={}",
                    current_thread.thread_id, link_kind
                )));
            }

            let mut threads = Vec::with_capacity(linked_thread_ids.len());
            for thread_id in linked_thread_ids {
                let thread = a
                    .store
                    .get_thread(&thread_id)
                    .await?
                    .ok_or_else(|| ThreadBotError::ThreadNotFound(thread_id.clone()))?;
                threads.push(thread);
            }
            Ok(threads)
        }
    }
}

async fn post_reply_to_thread<H: ThreadHandler>(
    target_thread: &ThreadRecord,
    a: &ActorCtx<H>,
    message: String,
    metadata: serde_json::Value,
) -> Result<String, ThreadBotError> {
    let mut req = models::CreatePostRequest::new(target_thread.channel_id.clone(), message);
    req.root_id = Some(target_thread.root_post_id.clone());
    if metadata != serde_json::Value::Null {
        req.props = Some(metadata.clone());
    }

    let created = posts_api::create_post(&a.mm_config, req, None)
        .await
        .map_err(ThreadBotError::mattermost_api)?;
    let created_at = created
        .create_at
        .map(ms_to_datetime)
        .unwrap_or_else(Utc::now);
    let user_id = created
        .user_id
        .clone()
        .or_else(|| a.ctx.bot_user_id.clone())
        .unwrap_or_default();
    let post_id = created.id;
    a.store
        .upsert_message(UpsertThreadMessage {
            post_id: post_id.clone(),
            thread_id: target_thread.thread_id.clone(),
            user_id,
            is_bot_message: true,
            root_id: created
                .root_id
                .clone()
                .or_else(|| Some(target_thread.root_post_id.clone())),
            parent_post_id: created
                .root_id
                .clone()
                .or_else(|| Some(target_thread.root_post_id.clone())),
            metadata,
            post_created_at: created_at,
            post_updated_at: created.update_at.map(ms_to_datetime),
            post_deleted_at: created.delete_at.and_then(nonzero_ms_to_datetime),
        })
        .await?;

    Ok(post_id)
}

async fn ensure_linked_thread<H: ThreadHandler>(
    source_thread: &ThreadRecord,
    a: &ActorCtx<H>,
    request: EnsureLinkedThreadRequest,
) -> Result<(), ThreadBotError> {
    let mut create_request =
        models::CreatePostRequest::new(request.channel_id.clone(), request.message);
    if request.metadata != serde_json::Value::Null {
        create_request.props = Some(request.metadata.clone());
    }
    let created = posts_api::create_post(&a.mm_config, create_request, None)
        .await
        .map_err(ThreadBotError::mattermost_api)?;
    let created_at = created
        .create_at
        .map(ms_to_datetime)
        .unwrap_or_else(Utc::now);
    let created_user_id = created
        .user_id
        .clone()
        .or_else(|| a.ctx.bot_user_id.clone())
        .unwrap_or_default();

    a.store
        .upsert_thread(UpsertThread {
            thread_id: created.id.clone(),
            root_post_id: created.id.clone(),
            channel_id: request.channel_id,
            creator_user_id: created_user_id.clone(),
            thread_kind: request.thread_kind,
            metadata: request.thread_metadata,
        })
        .await?;
    a.store
        .upsert_message(UpsertThreadMessage {
            post_id: created.id.clone(),
            thread_id: created.id.clone(),
            user_id: created_user_id,
            is_bot_message: true,
            root_id: None,
            parent_post_id: None,
            metadata: request.metadata,
            post_created_at: created_at,
            post_updated_at: created.update_at.map(ms_to_datetime),
            post_deleted_at: created.delete_at.and_then(nonzero_ms_to_datetime),
        })
        .await?;
    a.store
        .upsert_thread_link(UpsertThreadLink {
            source_thread_id: source_thread.thread_id.clone(),
            link_kind: request.link_kind,
            target_thread_id: created.id,
            metadata: request.link_metadata,
        })
        .await?;

    Ok(())
}

fn nonzero_ms_to_datetime(ms: i64) -> Option<DateTime<Utc>> {
    if ms == 0 {
        None
    } else {
        Some(ms_to_datetime(ms))
    }
}

fn stop_after_critical_effect_failure<H: ThreadHandler>(
    thread: &ThreadRecord,
    a: &ActorCtx<H>,
    effect_type: &'static str,
) {
    tracing::info!(
        thread_id = %a.thread_id,
        effect_type,
        root_post_id = %thread.root_post_id,
        "Thread actor stopped after critical effect failure"
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
