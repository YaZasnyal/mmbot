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

As of this note, the tree is already dirty. Do not assume every modified file
belongs to the current slice.

Known modified files:

- `rfcs/0007-passive-thread-effects.md`
- `lib/thread-bot/src/runtime.rs`
- `lib/thread-bot/src/actor.rs`
- `lib/thread-bot/src/handler.rs`
- `lib/support-bot/src/handler.rs`

Treat unrelated existing changes as user/previous-session work. Do not revert
them.

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
- `ThreadCommand::Reconcile` is temporarily left in actor code with
  `#[allow(dead_code)]` because actor-level legacy tests still cover it.

Verification run:

```bash
cargo fmt -p thread-bot --check
cargo test -p thread-bot
cargo clippy -p thread-bot -- -D warnings
```

All passed after slice 1.

## Next tasks

Prefer one small reviewable slice at a time.

1. Add focused runtime tests for checkpoint-driven reconciliation.
   - Reconnect does not call thread-list APIs.
   - Missed root post runs tracking decision.
   - Missed reply in a known thread spawns only that thread actor.
   - Bot-authored missed post is persisted but does not trigger handler.
   - `max_reconcile_window` advances checkpoint without thread status updates.

2. Remove legacy actor-level reconciliation path.
   - Remove `ThreadCommand::Reconcile`.
   - Remove `ReconcileOutcome` and `handle_reconcile`.
   - Remove actor tests that only validate old reconnect reconciliation.
   - Keep normal `NewMessage` reconciliation coverage in runtime tests.

3. Introduce `actor_idle_timeout`.
   - Add config field with `Duration::ZERO` default.
   - Make actor exit after quiescence when timeout expires.
   - Ensure no exit while pending work, handler running, or reschedule pending.
   - Remove actor from `actors` map when it exits, or make sender cleanup
     robust enough that next event respawns.

4. Start removing L3 business lifecycle.
   - Plan before editing because this touches store schema, handle API,
     actor effects, tests, and support-bot.
   - Replace routing checks that filter out `Resolved`/`Stopped`.
   - Move support-bot lifecycle into its metadata/state.
   - Eventually remove `ThreadStatus`, `MarkResolved`, `MarkStopped`, and
     `on_thread_closed`.

5. Add `thread_kind`.
   - Update existing initial migration, not a forward migration.
   - Add `thread_kind` to record/input types and store queries.
   - Replace RFC-era `kind` wording in implementation with `thread_kind`.

6. Add lazy invocation API.
   - Introduce `ThreadInvocation` and `ThreadTrigger`.
   - Handler receives lightweight record plus trigger.
   - Full snapshot is built only via `ctx.build_thread_snapshot(thread_id)`.
   - Update support-bot user flow to explicitly build snapshot only where
     LLM context needs transcript.

7. Add thread links and linked effects.
   - Add `thread_links` table to the initial migration.
   - Add store APIs for forward and reverse lookups.
   - Add `EnsureLinkedThread`.
   - Add `ThreadTarget::LinkedThread`.
   - Persist created bot replies immediately after Mattermost `create_post`.
   - Move support engineer thread creation out of `MattermostSupportNotifier`
     raw API calls and into effects.

8. Keep raw-post path removed.
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
