# RFC 0007 implementation handoff

Updated: 2026-05-10

## Goal

Implement RFC 0007 incrementally and keep each review slice small.

Primary RFC to read first:

- `rfcs/0007-passive-thread-effects.md`

High-level target:

- make `thread-bot` actors short-lived executors/synchronizers;
- make handler invocation lazy instead of always building a full `Thread`;
- move business lifecycle state out of Layer 3 and into L4 metadata/store;
- make reconciliation checkpoint/post-driven, not tracked-thread-driven;
- add typed thread links and linked-thread effects for support engineer/debug
  threads;
- keep generated Mattermost API and unrelated refactors out of these slices.

## Current working tree note

As of this note, implementation slice 1 was committed and the working tree was
clean. Still verify with `git status --short` before starting the next slice.

## Done

### RFC decisions recorded

- `thread_kind` is the L4-owned free-string classifier.
- `ThreadStatus`, `MarkResolved`, `MarkStopped`, and `on_thread_closed` should
  be removed from the target Layer 3 model.
- L4 owns lifecycle states such as `finished`, `stale`, `waiting_for_user`, and
  `escalated`.
- `actor_idle_timeout` default should be `Duration::ZERO`; nonzero values are a
  performance optimization only.
- Reconciliation should be proportional to missed posts, not tracked threads.
- `ThreadTrigger::Reconcile` should not exist in the target API; missed
  external posts should arrive as `NewMessage`.
- Thread links should live in normalized storage, not in `threads.metadata`.
- `EnsureLinkedThread` should create/register linked engineer/debug threads in
  L3 effect execution.
- `ThreadMetadataTarget` defaults to `CurrentThread`.

### Implementation slice 1: checkpoint-driven reconciliation

Files touched:

- `lib/thread-bot/migrations/20260408_001_initial.sql`
- `lib/thread-bot/src/channel_messages.rs`
- `lib/thread-bot/src/runtime.rs`
- `lib/thread-bot/src/actor.rs`
- `lib/thread-bot/src/store.rs`
- `lib/thread-bot/src/pg_store.rs`
- `lib/thread-bot/src/testutil.rs`
- `lib/thread-bot/src/types.rs`
- `lib/thread-bot/src/lib.rs`

Changes:

- `ThreadBotPlugin::reconcile()` no longer calls
  `list_threads_by_status(...)`.
- Runtime no longer ensures actors for all active/tracked threads on reconnect.
- Runtime no longer sends `ThreadCommand::Reconcile` from reconnect.
- Channel checkpoints now store `last_seen_post_id` as the reconciliation
  cursor.
- Added `channel_messages_after(...)`, a paged `Stream` that uses Mattermost's
  `after` cursor instead of `since`.
- Reconciliation scans channel checkpoints, iterates missed posts via
  `while let Some(result) = channel_messages.next().await`, and replays them
  through the normal post
  routing path.
- Missed replies only create actors for the specific tracked thread they belong
  to.
- Missed root posts go through normal tracking decision.
- `max_reconcile_window` now skips a channel backlog by advancing the channel
  checkpoint to the latest channel post; it does not force-close any threads.
- Removed raw-post execution support from runtime and `ThreadHandler`.

Verification run:

```bash
cargo fmt -p thread-bot --check
cargo test -p thread-bot
cargo clippy -p thread-bot -- -D warnings
```

All passed after slice 1.

### Checkpoint-driven reconciliation follow-up

The checkpoint/stream reconciliation task is considered solved for now. A
separate focused runtime-test slice was discussed, but we are intentionally not
blocking the refactor on it before removing the legacy actor-level
reconciliation path.

### Implementation slice 2: remove actor-level reconciliation

Files touched:

- `lib/thread-bot/src/actor.rs`
- `lib/thread-bot/src/actor_tests.rs`

Changes:

- Removed `ThreadCommand::Reconcile`.
- Removed `ReconcileOutcome` and `handle_reconcile`.
- Removed actor tests that only validated the old reconnect reconciliation
  command.
- Kept live API control-reaction checks in the normal debounce path.
- Runtime-level reconciliation remains channel checkpoint/post driven.

Verification run:

```bash
cargo fmt -p thread-bot --check
cargo test -p thread-bot
cargo clippy -p thread-bot -- -D warnings
```

All passed after slice 2.

### Implementation slice 3: actor idle timeout

Files touched:

- `lib/thread-bot/src/actor.rs`
- `lib/thread-bot/src/actor_tests.rs`
- `lib/thread-bot/src/runtime.rs`
- `lib/thread-bot/src/testutil.rs`

Changes:

- Added `ThreadBotConfig::actor_idle_timeout` with `Duration::ZERO` default.
- Added `ThreadBotPlugin::with_actor_idle_timeout(...)`.
- Actors schedule idle teardown only after at least one command/run, avoiding a
  spawn-before-first-command race.
- Actor exits after quiescence: no pending debounce, no running handler, no
  reschedule.
- Idle retirement is coordinated in `runtime.rs`: the spawned task keeps the
  receiver alive, locks the actor map, and exits only when the actor is still
  the registered sender, its receiver queue is empty, and no runtime event is
  holding an extra sender clone.
- If that idle retirement check fails, the same spawned task continues the
  actor loop with the same receiver instead of spawning a replacement actor.
- Exited actors remove only their own sender from the actor map, so the next
  event respawns the actor normally.
- Added actor tests for zero timeout, nonzero timeout, pending debounce, and
  running handler.

Verification run:

```bash
cargo fmt -p thread-bot --check
cargo test -p thread-bot
cargo clippy -p thread-bot -- -D warnings
```

All passed after slice 3.

### Implementation slice 4: move lifecycle checks to L4 metadata

Files touched:

- `lib/thread-bot/src/actor.rs`
- `lib/thread-bot/src/actor_tests.rs`
- `lib/thread-bot/src/runtime.rs`
- `lib/support-bot/src/handler.rs`
- `lib/support-bot/src/state.rs`
- `lib/support-bot/src/notifier.rs`
- `lib/support-bot/src/debug_export.rs`

Changes:

- Removed the actor's pre-handler Mattermost API scan for root control
  reactions. L3 no longer polls reactions during debounce to discover
  `MarkResolved`/`MarkStopped` before invoking L4.
- Changed actor control-reaction handling to execute any L4-returned
  `ThreadEffect`, not only close effects.
- Removed runtime routing gates that ignored persisted threads with
  `ThreadStatus::Resolved` or `ThreadStatus::Stopped`.
- Existing root posts, replies, and reactions now route based on whether the
  thread is tracked in storage, not whether Layer 3 considers it active.
- Added L4 metadata lifecycle checks in support-bot: non-`Active`
  `SupportThreadState.status` now skips the user workflow and returns `Noop`.
- Added `SupportThreadStatus::Stopped`.
- Moved support-bot control-reaction lifecycle behavior into L4 metadata:
  ✅ sets `support_bot.status = "finished"` and 🛑 sets
  `support_bot.status = "stopped"` via `SetThreadMetadata`.
- Added an L4 pre-workflow live root reaction check in support-bot, replacing
  the removed L3 debounce-time Mattermost API scan without making Layer 3 own
  the lifecycle decision.
- Added regression tests proving finished/stopped metadata prevents LLM
  invocation even though Layer 3 still routes the thread, and proving
  control-reaction metadata effects are executed.
- Kept `ThreadStatus`, `MarkResolved`, `MarkStopped`, `on_thread_closed`, and
  status store APIs in place for a later compatibility-breaking slice. The
  WebSocket control-reaction path still calls `on_control_reaction`.

Verification run:

```bash
cargo fmt -p thread-bot --check
cargo test -p thread-bot
cargo clippy -p thread-bot -- -D warnings
cargo fmt -p support-bot --check
cargo test -p support-bot
cargo clippy -p support-bot -- -D warnings
```

All passed after slice 4.

### Implementation slice 5: remove support-bot close effects

Files touched:

- `lib/support-bot/src/user_thread_run.rs`
- `lib/support-bot/src/handler.rs`

Changes:

- Removed support-bot `MarkResolved` emission from the `finish_request` path.
- Removed support-bot `MarkStopped` emission from the tool-loop-limit path.
- Tool-loop-limit now sets `support_bot.status = "stopped"` in thread
  metadata before persisting final state.
- Finish remains metadata-only: `finish_request` sets
  `support_bot.status = "finished"` and the handler persists that state.
- Updated support-bot tests to assert metadata lifecycle state instead of L3
  close effects.

Verification run:

```bash
cargo fmt -p support-bot --check
cargo test -p support-bot
cargo clippy -p support-bot -- -D warnings
```

All passed after slice 5.

### Implementation slice 6: remove L3 business lifecycle

Files touched:

- `lib/thread-bot/migrations/20260408_001_initial.sql`
- `lib/thread-bot/src/actor.rs`
- `lib/thread-bot/src/actor_tests.rs`
- `lib/thread-bot/src/handle.rs`
- `lib/thread-bot/src/handler.rs`
- `lib/thread-bot/src/lib.rs`
- `lib/thread-bot/src/pg_store.rs`
- `lib/thread-bot/src/runtime.rs`
- `lib/thread-bot/src/store.rs`
- `lib/thread-bot/src/testutil.rs`
- `lib/thread-bot/src/types.rs`
- `lib/thread-bot/tests/common/mod.rs`
- `lib/thread-bot/tests/pg_store_tests.rs`
- `lib/support-bot/src/debug_export.rs`
- `lib/support-bot/src/handler.rs`
- `lib/support-bot/src/notifier.rs`
- `lib/support-bot/src/testutil.rs`
- `examples/hello_thread_bot/src/main.rs`
- `lib/support-bot/README.md`

Changes:

- Removed `ThreadStatus` from Layer 3 records, snapshots, upsert inputs,
  store rows, and the initial migration.
- Removed status-oriented store/handle APIs:
  `list_threads_by_status`, `update_thread_status`, `resolve_thread`, and
  `stop_thread`.
- Added status-free `ThreadStore::list_threads(...)` and
  `ThreadBotHandle::list_threads(...)`.
- Removed `ThreadEffect::MarkResolved`, `ThreadEffect::MarkStopped`,
  `ThreadCloseReason`, and `ThreadHandler::on_thread_closed`.
- Removed actor lifecycle transitions and close-effect execution; actors now
  stop only for channel shutdown, critical effect failure, or idle teardown.
- Updated support-bot debug/export/test helpers for status-free Layer 3
  records.
- Updated `hello_thread_bot` to store example lifecycle under metadata instead
  of calling L3 close APIs.
- Updated pg-store and actor tests to assert persisted tracking/progress and
  metadata instead of L3 lifecycle state.

Verification run:

```bash
cargo fmt -p thread-bot --check
cargo fmt -p support-bot --check
cargo fmt -p hello-thread-bot --check
cargo test -p thread-bot
cargo test -p support-bot
cargo check -p hello-thread-bot
cargo clippy -p thread-bot -- -D warnings
cargo clippy -p support-bot -- -D warnings
cargo clippy -p hello-thread-bot -- -D warnings
```

All passed after slice 6. `fmt --check` initially requested mechanical wrapping
in `thread-bot` and `support-bot`; `cargo fmt -p thread-bot -p support-bot -p
hello-thread-bot` was applied and checks were rerun.

### Implementation slice 7: add `thread_kind`

Files touched:

- `lib/thread-bot/migrations/20260408_001_initial.sql`
- `lib/thread-bot/src/types.rs`
- `lib/thread-bot/src/handler.rs`
- `lib/thread-bot/src/runtime.rs`
- `lib/thread-bot/src/pg_store.rs`
- `lib/thread-bot/src/testutil.rs`
- `lib/thread-bot/src/actor_tests.rs`
- `lib/thread-bot/tests/common/mod.rs`
- `lib/thread-bot/tests/pg_store_tests.rs`
- `lib/support-bot/src/handler.rs`
- `lib/support-bot/src/notifier.rs`
- `examples/hello_thread_bot/src/main.rs`

Changes:

- Added nullable `threads.thread_kind` to the initial migration and indexed it.
- Added `thread_kind: Option<String>` to `ThreadRecord`, `ThreadInfo`, and
  `UpsertThread`.
- Updated PostgreSQL and mock store upsert/select paths to persist and return
  `thread_kind`.
- Added `ThreadHandler::thread_kind(...)` as a temporary compatibility hook
  while `should_track() -> bool` still exists; Layer 3 stores the value but
  does not interpret it.
- `support-bot` now classifies tracked user threads as `support_user`;
  engineer/debug thread linkage still waits for the linked-thread effects
  slice.
- `hello_thread_bot` now classifies tracked threads as `hello_world`.
- Updated pg-store tests/helpers and support-bot test fixtures for the new
  field.

Verification run:

```bash
cargo fmt -p thread-bot -p support-bot -p hello-thread-bot
cargo fmt -p thread-bot -p support-bot -p hello-thread-bot --check
cargo test -p thread-bot
cargo test -p support-bot
cargo check -p hello-thread-bot
cargo clippy -p thread-bot -- -D warnings
cargo clippy -p support-bot -- -D warnings
cargo clippy -p hello-thread-bot -- -D warnings
```

All passed after slice 7.

### Implementation slice 8: lazy handler invocation

Files touched:

- `lib/thread-bot/src/types.rs`
- `lib/thread-bot/src/handler.rs`
- `lib/thread-bot/src/actor.rs`
- `lib/thread-bot/src/lib.rs`
- `lib/thread-bot/src/runtime.rs`
- `lib/thread-bot/src/testutil.rs`
- `lib/support-bot/src/handler.rs`
- `examples/hello_thread_bot/src/main.rs`

Changes:

- Added `ThreadInvocation` and `ThreadTrigger` to the public Layer 3 API.
- Changed `ThreadHandler::handle(...)` to receive a lightweight
  `ThreadInvocation` instead of an eagerly built full `Thread`.
- Actor debounce execution now loads only the persisted `ThreadRecord` before
  invoking L4; full Mattermost thread snapshots are no longer built by default.
- Actor effect execution and control-reaction execution now use `ThreadRecord`
  for channel/root metadata instead of requiring a full snapshot.
- Successful handler runs advance `last_processed_*` from the invocation's
  persisted `last_seen_*` cursor.
- Added trigger variants for `NewMessage`, `Reschedule`, and `Wake`.
- Updated support-bot routing to check metadata/live control reactions from the
  lightweight record first, then explicitly call
  `ctx.build_thread_snapshot(thread_id)` only when it needs message text or LLM
  transcript context.
- Updated engineer debug handling and the hello example to build snapshots
  explicitly where transcript/message count is needed.
- Updated support-bot handler unit tests to exercise the new invocation path
  with a small snapshot store and Mattermost API mock.

Verification run:

```bash
cargo fmt -p thread-bot -p support-bot -p hello-thread-bot
cargo test -p thread-bot
cargo test -p support-bot
cargo clippy -p thread-bot -- -D warnings
cargo clippy -p support-bot -- -D warnings
cargo clippy -p hello-thread-bot -- -D warnings
```

All passed after slice 8.

### Implementation slice 9: thread link storage

Files touched:

- `lib/thread-bot/migrations/20260408_001_initial.sql`
- `lib/thread-bot/src/types.rs`
- `lib/thread-bot/src/store.rs`
- `lib/thread-bot/src/pg_store.rs`
- `lib/thread-bot/src/testutil.rs`
- `lib/thread-bot/src/lib.rs`
- `lib/thread-bot/tests/pg_store_tests.rs`
- `lib/support-bot/src/testutil.rs`
- `lib/support-bot/src/handler.rs`

Changes:

- Added normalized `thread_links` table to the initial migration.
- Added `ThreadLink` and `UpsertThreadLink` public types.
- Added store APIs for typed forward links:
  `upsert_thread_link`, `get_thread_link`, and `list_thread_links`.
- Added `list_reverse_thread_links` for reverse lookup by target thread.
- Implemented link APIs in PostgreSQL store and in-memory test store.
- Added pg-store tests for create/read, update-by-target-thread, forward listing,
  and reverse listing.
- Updated support-bot test stores for the expanded `ThreadStore` trait.
- This slice intentionally does not add `EnsureLinkedThread`,
  linked `ThreadTarget` variants, or support-bot notifier migration yet.

Verification run:

```bash
cargo fmt -p thread-bot -p support-bot
cargo test -p thread-bot
cargo test -p support-bot
cargo clippy -p thread-bot -- -D warnings
cargo clippy -p support-bot -- -D warnings
```

All passed after slice 9.

### Implementation slice 10: `EnsureLinkedThread` effect

Files touched:

- `lib/thread-bot/src/handler.rs`
- `lib/thread-bot/src/actor.rs`
- `lib/thread-bot/src/actor_tests.rs`
- `lib/support-bot/src/handler.rs`

Changes:

- Added `ThreadEffect::EnsureLinkedThread`.
- Effect execution creates a Mattermost root post in the requested channel,
  tracks the created root as a thread, persists the created root message
  immediately, and stores the typed thread link.
- Link/thread/message metadata are explicit effect inputs.
- Duplicate-avoidance policy is left to the plugin. `support-bot` now checks
  for an existing engineer link before queuing the effect.
- Added actor regression coverage for creating the linked tracked thread,
  root message, and link.
- This slice intentionally does not migrate support-bot engineer notification
  code to the effect yet.

Verification run:

```bash
cargo fmt -p thread-bot
cargo test -p thread-bot
cargo test -p support-bot
cargo clippy -p thread-bot -- -D warnings
cargo clippy -p support-bot -- -D warnings
cargo check -p hello-thread-bot
```

All passed after slice 10.

### Implementation slice 11: linked reply targets

Files touched:

- `lib/thread-bot/src/handler.rs`
- `lib/thread-bot/src/lib.rs`
- `lib/thread-bot/src/actor.rs`
- `lib/thread-bot/src/actor_tests.rs`

Changes:

- Added public `ThreadTarget` with `CurrentThread`,
  `Thread { thread_id }`, and `LinkedThreads { link_kind }`.
- Changed `ThreadEffect::Reply` to include `target`, so current-thread and
  linked-thread replies use one effect shape.
- Actor resolves explicit thread targets by `thread_id`, and resolves
  `LinkedThreads` through `thread_links` to post to every matching linked
  tracked thread.
- Actor now persists created bot replies immediately after Mattermost
  `create_post`.
- Added actor coverage for immediate current-thread reply persistence,
  explicit-thread reply persistence, and linked-thread reply persistence.

Verification run:

```bash
cargo fmt -p thread-bot
cargo test -p thread-bot
cargo test -p support-bot
cargo clippy -p thread-bot -- -D warnings
```

All passed after slice 11. `cargo check -p support-bot` also passed during
implementation.

### Implementation slice 12: support-bot linked engineer effects

Files touched:

- `lib/support-bot/src/handler.rs`
- `lib/support-bot/src/user_thread_run.rs`
- `lib/support-bot/src/workflow.rs`
- `lib/support-bot/src/notifier.rs`

Changes:

- Added `support_engineer` as the support-bot engineer link kind.
- User workflow now checks for an engineer link before LLM work when the
  configured engineer target is a Mattermost channel.
- If the link is absent, support-bot returns
  `EnsureLinkedThread { link_kind: "support_engineer", ... }` plus
  `Reschedule`, so creation is executed by Layer 3 and the workflow resumes
  after the link exists.
- Bot/user mirrors, `notify_engineer`, tool-loop-limit engineer notifications,
  and finish status updates now queue `Reply { target:
  LinkedThreads { link_kind: "support_engineer" }, ... }` effects instead of
  calling Mattermost directly.
- `MattermostSupportNotifier` still exists for debug report attachment upload
  and legacy notifier tests, but the normal user workflow no longer uses it to
  create or write engineer threads.

Verification run:

```bash
cargo fmt -p thread-bot -p support-bot
cargo test -p thread-bot
cargo test -p support-bot
cargo clippy -p thread-bot -- -D warnings
cargo clippy -p support-bot -- -D warnings
cargo check -p hello-thread-bot
```

All passed after slice 12.

### Implementation slice 13: explicit and broadcast thread targets

Files touched:

- `lib/thread-bot/src/handler.rs`
- `lib/thread-bot/src/store.rs`
- `lib/thread-bot/src/pg_store.rs`
- `lib/thread-bot/src/testutil.rs`
- `lib/thread-bot/src/actor.rs`
- `lib/thread-bot/src/actor_tests.rs`
- `lib/thread-bot/tests/pg_store_tests.rs`
- `lib/support-bot/src/handler.rs`
- `lib/support-bot/src/user_thread_run.rs`
- `lib/support-bot/src/workflow.rs`
- `lib/support-bot/src/testutil.rs`

Changes:

- Changed `thread_links` primary key from `(source_thread_id, link_kind)` to
  `(source_thread_id, target_thread_id)`, so `link_kind` is classification,
  not uniqueness policy.
- Changed `get_thread_link` to read by source and target thread id.
- Kept `list_thread_links` as the plugin-level API for choosing any grouping
  policy by metadata or `link_kind`.
- Replaced single `LinkedThread { link_kind }` reply targeting with:
  `Thread { thread_id }` for one explicit tracked thread and
  `LinkedThreads { link_kind }` for broadcasting to every linked target of
  that kind.
- Updated support-bot engineer mirrors/notifications to broadcast via
  `LinkedThreads { link_kind: "support_engineer" }`.
- Added actor coverage for explicit thread-id replies.

Verification run:

```bash
cargo fmt -p thread-bot -p support-bot -p hello-thread-bot
cargo test -p thread-bot
cargo test -p support-bot
cargo clippy -p thread-bot -- -D warnings
cargo clippy -p support-bot -- -D warnings
cargo check -p hello-thread-bot
```

All passed after slice 13.

### Implementation slice 14: remove legacy engineer-thread state

Files touched:

- `lib/support-bot/src/handler.rs`
- `lib/support-bot/src/state.rs`

Changes:

- Removed `engineer_thread` from persisted `SupportThreadState`.
- User-thread flow no longer caches engineer-thread coordinates in thread
  metadata; it uses normalized `thread_links` only.
- `ensure_engineer_thread_effects` remains the explicit plugin-side gate for
  deciding whether a linked engineer thread must be created.
- Kept backward compatibility when reading old metadata by ignoring the legacy
  `engineer_thread` field during deserialization.
- Updated tests to model existing engineer routing through `thread_links`
  instead of metadata.

Verification run:

```bash
cargo fmt -p support-bot
cargo test -p support-bot
```

All passed after slice 14.

### Implementation slice 15: metadata effect targets

Files touched:

- `lib/thread-bot/src/handler.rs`
- `lib/thread-bot/src/lib.rs`
- `lib/thread-bot/src/actor.rs`
- `lib/thread-bot/src/actor_tests.rs`
- `lib/thread-bot/src/testutil.rs`
- `lib/support-bot/src/handler.rs`
- `lib/support-bot/src/user_thread_run.rs`
- `lib/support-bot/src/tests/handler.rs`
- `examples/hello_thread_bot/src/main.rs`

Changes:

- Added public `ThreadMetadataTarget` with `CurrentThread`, `ThreadId`, and
  `RootPostId`.
- Changed `ThreadEffect::SetThreadMetadata` to carry an explicit metadata
  target.
- Actor effect execution now resolves metadata targets and writes metadata to
  the selected tracked thread.
- Existing support-bot and hello-thread-bot call sites now explicitly use
  `ThreadMetadataTarget::CurrentThread`.
- Fixed the in-memory test store's `get_thread_by_post(...)` root-post lookup
  to match PostgreSQL behavior when `thread_id != root_post_id`.
- Added actor coverage for setting metadata by explicit thread id and by root
  post id.

Verification run:

```bash
cargo fmt -p thread-bot -p support-bot -p hello-thread-bot
cargo test -p thread-bot actor_set_thread_metadata_effect -- --nocapture
cargo test -p support-bot --no-run
cargo check -p hello-thread-bot
cargo test -p thread-bot
cargo test -p support-bot
cargo clippy -p thread-bot -- -D warnings
cargo clippy -p support-bot -- -D warnings
cargo clippy -p hello-thread-bot -- -D warnings
```

All passed after slice 15.

## Next tasks

Prefer one small reviewable slice at a time.

1. Replace the temporary `should_track(...) -> bool` plus `thread_kind(...)`
   compatibility hook with a single tracking decision API that can return
   `Ignore` or `Track { thread_kind, metadata }`.
2. Decide whether to move `EnsureLinkedThread` duplicate avoidance into Layer 3
   by making `(source_thread_id, link_kind)` lookup/no-op semantics part of the
   effect executor, or keep the current plugin-owned policy.
3. Keep raw-post path removed.
   - `execute_raw_post_effects` has been removed from runtime.
   - `ThreadHandler::handle_raw_post` has been removed.
   - `build_raw_engineer_thread_snapshot` is not present.
   - RFC target says no separate raw executor long-term.

## Review posture

User wants to review gradually. Do not batch the whole RFC into one massive
patch. After each slice:

- summarize exactly what changed;
- list commands run;
- call out legacy paths intentionally left behind;
- update this note.
