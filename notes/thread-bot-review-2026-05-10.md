# Thread-bot review: simplification pass

Date: 2026-05-10
Scope: `lib/thread-bot`, with emphasis on `lib/thread-bot/src/runtime.rs`.

## Executive summary

`thread-bot` has already moved in the right direction after RFC 0007: actor lifecycle is no longer business lifecycle, snapshots are lazy, and reconciliation is checkpoint/post-driven. The main remaining issue is that `runtime.rs` is still doing too many jobs at once.

The best simplification path is not a big rewrite. I would split the current runtime into three small responsibilities and delete duplicated conversion/routing code as we go:

1. `router`: convert Mattermost events/posts/reactions into internal routing actions.
2. `ingest`: persist root posts, replies, bot messages, reactions, and checkpoints.
3. `actor_registry`: own `HashMap<thread_id, Sender<ThreadCommand>>` and actor spawn/teardown details.

That should make the core runtime read like orchestration instead of stateful plumbing.

## Findings

### P1: Reconciliation can run concurrently with live routing despite the comment saying it blocks

Location: `lib/thread-bot/src/runtime.rs:772`

`process_event()` spawns reconciliation on `Hello`:

```rust
tokio::spawn(async move {
    tracing::info!("Starting reconcile");
    plugin.reconcile(&config).await;
});
```

But the comment above `reconcile()` says `process_event` blocks the WebSocket read loop while reconciliation runs. That is no longer true. Live `Posted` events can be processed while reconciliation is scanning.

This matters because `reconcile()` calls `set_all_channels_not_reconciled()` and then replays missed posts through the same routing path. Live events arriving during that window will still call `handle_posted()` / `handle_thread_reply()`. Checkpoint advances should be skipped while `is_reconciled = false`, but actor commands and handler runs are not blocked. That may be acceptable, but the current code/comments suggest a stronger invariant than the implementation provides.

Recommended simplification:

- Add a small `Reconciler`/`RuntimeGate` state, probably `Arc<Mutex<()>>` or an atomic task guard, so only one reconciliation runs at a time.
- Decide explicitly whether live routing is allowed during reconcile.
- If allowed, update the doc comment and add tests for duplicate replay/live delivery.
- If not allowed, stop spawning reconcile detached and await it from `process_event()` or buffer live events behind an internal queue.

My bias: for now, keep live routing allowed, but add a single-flight guard and update the docs. Full event buffering is heavier and would bring back complexity RFC 0007 was trying to remove.

### P1: Root-post replay for an already tracked thread spawns an actor but does not send a message

Location: `lib/thread-bot/src/runtime.rs:230`

When `handle_new_thread()` sees an already tracked root post, it ensures an actor exists, advances the channel checkpoint, and returns:

```rust
Ok(Some(record)) => {
    self.ensure_or_spawn_actor(&record.thread_id, mm_config).await;
    ...
    return;
}
```

It does not send `ThreadCommand::NewMessage` and does not persist/update the root message. For an idempotent replay this is mostly harmless, but spawning an idle actor without work is surprising. With `actor_idle_timeout = Duration::ZERO`, this actor can immediately enter teardown logic for no useful reason.

Recommended simplification:

- If the root thread already exists, do not spawn an actor unless there is actual work to send.
- Extract a `record_channel_seen(channel_id, post)` helper and return after checkpoint update.
- Let replies or reactions create the actor when work exists.

This removes one branch where actor lifecycle leaks into ingestion.

### P1: Actor registry teardown relies on `Sender::strong_count()` heuristics

Location: `lib/thread-bot/src/runtime.rs:677`

After `thread_actor()` returns, the spawned task decides whether to continue or remove itself by checking:

```rust
let sender_count = current.strong_count();
if sender_count >= 2 || !rx.is_empty() {
    ... continue;
}
```

This is clever, but it is also hard to reason about. A temporary sender clone held by a routing function can keep the actor alive even if no command will actually arrive. Conversely, the code is relying on subtle timing between `send()`, clone lifetime, queue state, and registry locking.

Recommended simplification:

- Move this logic into an `ActorRegistry` type with one public method: `send_or_spawn(thread_id, command, config)`.
- Keep sender lifetime entirely inside that method where possible: get/spawn, send, remove-on-send-error.
- Let actors exit and remove their registry entry by identity/token rather than by sender count. For example, store `ActorEntry { tx, generation }`; spawned task removes only if generation still matches.

That would replace the sender-count teardown protocol with a generation-token protocol, which is easier to test and explain.

### P2: `runtime.rs` duplicates post-to-record conversion logic that already belongs near actor ingestion

Locations: `lib/thread-bot/src/runtime.rs:344`, `lib/thread-bot/src/actor.rs`

`handle_thread_reply()` manually builds `UpsertThreadMessage` for bot-authored messages. Actor ingestion has similar knowledge: post id, root id, parent id, timestamps, deleted-at zero handling, bot detection, and seen updates.

This duplication increases the chance that bot-message persistence and user-message persistence drift apart.

Recommended simplification:

- Add a small constructor/helper, for example:

```rust
impl UpsertThreadMessage {
    pub(crate) fn from_post(post: &models::Post, thread_id: &str, is_bot_message: bool) -> Self
}
```

or a free function in a new `ingest.rs`:

```rust
fn message_upsert_from_post(post: &models::Post, thread_id: &str, is_bot_message: bool) -> UpsertThreadMessage
```

- Use it from both runtime bot-message persistence and actor user-message persistence.
- Reuse the existing `nonzero_ms_to_datetime` style helper instead of hand-writing `if ts == 0` in runtime.

This is a clean deletion target.

### P2: Reaction routing has hidden policy and storage in one nested function

Location: `lib/thread-bot/src/runtime.rs:389`

Original concern: `handle_reaction()` did four things:

1. Finds the thread by post id.
2. Builds `ReactionChange`.
3. Persists root/control reactions unconditionally.
4. For non-root reactions, persisted only feedback reactions on bot messages.
5. Optionally calls handler sync policy and dispatches a control effect to actor.

That nested feedback policy made reaction storage look like a base runtime concern, even though emoji interpretation belongs to L4.

Updated simplification:

- Do not persist emoji reactions in the base runtime at all.
- Keep root-post control reactions as an L4 hook via `on_control_reaction()`.
- Ignore non-root reactions in `thread-bot`; L4 can persist any reaction-derived state into its own metadata if it needs that later.

### P2: `ThreadBotPlugin` owns too much mutable shared state directly

Location: `lib/thread-bot/src/runtime.rs:124`

Fields currently include handler, store, config, actors, bot user id, metrics. That is manageable, but most methods only need subsets. The result is that every helper has access to everything, so boundaries blur.

Recommended simplification:

Introduce two narrow structs:

```rust
struct RuntimeDeps<H> {
    handler: Arc<H>,
    store: Arc<dyn ThreadStore>,
    metrics: ThreadBotMetricsHandle,
    bot_user_id: Arc<RwLock<Option<String>>>,
}

struct ActorRegistry<H> { ... }
```

Then `ThreadBotPlugin` becomes configuration + deps + registry. This is not just aesthetics: it makes it harder for reconciliation code to accidentally manage actors, or for reaction classification to mutate checkpoints.

### P2: Reconciliation is too large and mixes scanning, skip policy, replay, and checkpoint finalization

Location: `lib/thread-bot/src/runtime.rs:493`

`reconcile()` is now conceptually good, but the function still has four responsibilities:

1. Snapshot checkpoints and mark channels unreconciled.
2. Apply max-window skip policy.
3. Stream and replay posts.
4. Finalize checkpoint/reconciled state.

Recommended simplification:

Split into tiny units:

```rust
async fn reconcile(&self, config: &Arc<Configuration>) {
    for checkpoint in self.begin_reconciliation().await {
        self.reconcile_channel(config, checkpoint).await;
    }
}

async fn reconcile_channel(&self, config: &Arc<Configuration>, cp: ChannelCheckpoint)
async fn skip_channel_backlog(...)
async fn replay_channel_posts(...) -> Option<models::Post>
async fn finish_channel_reconciliation(...)
```

This makes `max_reconcile_window` behavior locally testable without needing a full runtime.

### P2: Checkpoint writes are spread across routing paths

Locations: `lib/thread-bot/src/runtime.rs:238`, `lib/thread-bot/src/runtime.rs:276`, `lib/thread-bot/src/runtime.rs:324`, `lib/thread-bot/src/runtime.rs:573`, `lib/thread-bot/src/runtime.rs:622`

Checkpoint semantics are one of the most important correctness stories in this crate, but runtime writes checkpoints from many branches:

- existing root post replay;
- new root thread creation;
- reply routing;
- reconciliation last replayed post;
- skipped reconciliation.

Recommended simplification:

Create a small `ChannelCheckpointWriter` or helper methods:

```rust
async fn register_channel_if_needed(channel_id, post)
async fn advance_channel_if_reconciled(channel_id, post)
async fn force_checkpoint_to_post(channel_id, post)
```

The names encode policy. That would make it much easier to audit whether a path should advance only when reconciled or force upsert regardless of reconciliation state.

### P2: `handle_posted()` and `handle_replayed_post()` are duplicate routers

Locations: `lib/thread-bot/src/runtime.rs:203`, `lib/thread-bot/src/runtime.rs:590`

Both functions branch on `is_root_post()` and call the same root/reply handlers. One takes `PostedEvent`, the other takes `models::Post`.

Recommended simplification:

- Keep one `route_post(&self, post: &models::Post, source: PostSource, config: &Arc<Configuration>)`.
- `PostSource` can start as a tiny enum with `Live` and `Replay`, even if behavior is initially identical.
- If no behavior differs, skip the enum and just share `route_post()`.

The value is not huge by itself, but it is the first easy cut toward an ingestion module.

### P3: `ThreadContext` captures `bot_user_id` only when the actor is spawned

Location: `lib/thread-bot/src/runtime.rs:663`

`ActorCtx` stores both a snapshot `ThreadContext { bot_user_id: self.bot_user_id.read().await.clone() }` and the live `bot_user_id: Arc<RwLock<Option<String>>>`.

That means handler context sees the bot user id as it was at actor spawn, not necessarily the latest value. In practice `on_start()` resolves it before normal events, so this is probably fine. But carrying both forms is confusing.

Recommended simplification:

- Either make `ThreadContext` hold `Arc<RwLock<Option<String>>>` and expose an async getter, or remove `ActorCtx.bot_user_id` if it is no longer needed after lazy snapshot changes.
- Prefer one source of truth.

### P3: Error handling often logs and continues with no returned status

Locations: broad, especially `handle_new_thread()`, `handle_thread_reply()`, `handle_reaction()`, `reconcile()`.

Many runtime helpers return `()` and log errors internally. That keeps plugin processing resilient, but it makes tests and higher-level orchestration harder because failures become side effects in logs.

Recommended simplification:

- Make lower-level helpers return `Result<_, ThreadBotError>`.
- Keep the top-level plugin boundary responsible for logging and swallowing errors.
- Example split:

```rust
async fn handle_thread_reply(...) -> Result<(), ThreadBotError>
async fn process_event(...) { if let Err(e) = ... { tracing::error!(...) } }
```

This lets unit tests assert behavior without scraping logs.

## Suggested refactor slices

### Slice 1: Small deletion pass, low risk (done)

Target files:

- `lib/thread-bot/src/runtime.rs`
- `lib/thread-bot/src/actor.rs` or a new `lib/thread-bot/src/ingest.rs`

Changes:

- Extract `route_post()` and delete `handle_replayed_post()` duplication.
- Extract post timestamp/deleted-at/message-upsert helpers.
- Stop spawning an actor for already tracked root posts when no command is sent.

Expected result: less code in `runtime.rs`, no behavior change except avoiding pointless idle actor spawns.

Status: done 2026-05-11.

- Added shared post routing via `route_post()` and removed `handle_replayed_post()` duplication.
- Added `post_to_upsert_thread_message()` so runtime bot-message persistence and actor incoming-message persistence use the same conversion rules.
- Existing tracked root-post replay now advances the checkpoint without spawning an idle actor.

### Slice 2: Reconciliation guard and clearer semantics (done)

Target files:

- `lib/thread-bot/src/runtime.rs`
- maybe `lib/thread-bot/src/reconcile.rs`

Changes:

- Add single-flight guard for `Hello` reconciliation.
- Split `reconcile()` into `reconcile_channel()`, `skip_channel_backlog()`, `replay_channel_posts()`, and `finish_channel_reconciliation()`.
- Update comments to say whether live routing can happen concurrently.

Expected result: fewer hidden assumptions around checkpoint safety.

Status: done 2026-05-11.

- Added a single-flight reconciliation gate so repeated `Hello` events cannot start overlapping scans.
- Updated runtime comments to make the current semantic explicit: reconciliation is backgrounded, and live routing is allowed while checkpoints are held unreconciled.
- Split channel reconciliation into `reconcile_channel()`, `skip_channel_backlog()`, `replay_channel_posts()`, and `finish_channel_reconciliation()`.

### Slice 3: Actor registry extraction (done)

Target files:

- new `lib/thread-bot/src/actor_registry.rs`
- `lib/thread-bot/src/runtime.rs`
- `lib/thread-bot/src/handle.rs`

Changes:

- Move `actors: Arc<Mutex<HashMap<...>>>` into an `ActorRegistry` wrapper.
- Replace `ensure_or_spawn_actor()` + manual `tx.send()` call sites with `registry.send_or_spawn(thread_id, command, config)`.
- Replace `strong_count()` teardown heuristic with generation-token removal if feasible.

Expected result: runtime stops knowing how actors retire.

Status: done 2026-05-11.

- Added `ActorRegistry` in `lib/thread-bot/src/actor_registry.rs`.
- Replaced runtime actor map access, actor spawning, and actor retirement with registry-owned methods.
- Kept the `Sender::strong_count()` teardown check inside the registry because it models the important race: a sender clone may already be selected for a send before the command is visible in the queue.
- `ThreadBotHandle::wake_thread()` now routes through the registry without spawning missing actors.

### Slice 4: Reaction router cleanup (done)

Target files:

- `lib/thread-bot/src/runtime.rs`
- maybe new `lib/thread-bot/src/reactions.rs`

Changes:

- Remove base runtime reaction persistence.
- Keep only the root-post `on_control_reaction` hook and control-effect dispatch.
- Remove the feedback-reaction policy hook from `ThreadHandler`.

Expected result: reaction behavior becomes small enough that L4 owns any domain-specific emoji policy.

Status: done 2026-05-11.

- Removed runtime emoji persistence, including feedback reaction storage.
- Removed the `is_feedback_reaction()` handler hook.
- Kept root control reactions as a synchronous L4 policy hook that can dispatch effects.

## What I would not do yet

- I would not introduce a full internal event bus unless reconciliation/live races prove painful. It may be correct, but it is a bigger architecture move.
- I would not merge actor and runtime logic. The actor is already a better boundary than the runtime; the pressure should go the other direction.
- I would not move Mattermost API effect execution back into runtime. Keeping effects in the actor is consistent with the “one executor per thread while active” model.
- I would not add Layer 3 business statuses again. RFC 0007’s direction still looks right.

## Quick win checklist

- [x] Delete `handle_replayed_post()` by introducing shared `route_post()`.
- [x] Avoid actor spawn for already tracked root post replay without command.
- [x] Extract `UpsertThreadMessage` creation from `models::Post`.
- [x] Extract checkpoint helper methods with policy names.
- [x] Add reconciliation single-flight guard.
- [x] Split `reconcile()` into channel-level helpers.
- [x] Wrap actor map in `ActorRegistry`.
- [x] Move actor spawn/retirement into `ActorRegistry`; keep `strong_count()` localized there.

## Bottom line

The main simplification opportunity is to make `runtime.rs` boring. Right now it is a router, ingester, reconciler, reaction policy engine, actor registry, and plugin adapter in one file. The underlying model is mostly sound; the complexity is concentrated in boundaries that have not been named yet.

If we name those boundaries and delete the duplicated conversion/checkpoint code, the remaining hard parts should be isolated to two places only:

- `actor.rs`: debounce, cancellation, effects.
- `reconcile.rs`: checkpoint scan and replay policy.

Everything else can become thin orchestration.
