# RFC 0007 implementation handoff

Updated: 2026-05-09

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

## Next tasks

Prefer one small reviewable slice at a time.

1. Add `thread_kind`.
   - Update existing initial migration, not a forward migration.
   - Add `thread_kind` to record/input types and store queries.
   - Replace RFC-era `kind` wording in implementation with `thread_kind`.

2. Add lazy invocation API.
   - Introduce `ThreadInvocation` and `ThreadTrigger`.
   - Handler receives lightweight record plus trigger.
   - Full snapshot is built only via `ctx.build_thread_snapshot(thread_id)`.
   - Update support-bot user flow to explicitly build snapshot only where
     LLM context needs transcript.

3. Add thread links and linked effects.
   - Add `thread_links` table to the initial migration.
   - Add store APIs for forward and reverse lookups.
   - Add `EnsureLinkedThread`.
   - Add `ThreadTarget::LinkedThread`.
   - Persist created bot replies immediately after Mattermost `create_post`.
   - Move support engineer thread creation out of `MattermostSupportNotifier`
     raw API calls and into effects.

4. Keep raw-post path removed.
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
