# Support Bot Refactor Review (2026-05-10)

## Scope

Reviewed `lib/support-bot` after RFC 0007 slice 16, with focus on next
refactors that make code simpler and smaller.

Main files reviewed:

- `lib/support-bot/src/handler.rs`
- `lib/support-bot/src/user_thread_run.rs`
- `lib/support-bot/src/workflow.rs`
- `lib/support-bot/src/notifier.rs`
- `lib/support-bot/src/debug_export.rs`
- `lib/support-bot/src/tools.rs`
- `lib/support-bot/src/conversation.rs`
- `lib/support-bot/src/state.rs`
- `lib/support-bot/src/tests/handler.rs`

## Summary

RFC 0007 did the important boundary cleanup: normal support workflow no longer
creates or writes engineer threads through direct Mattermost calls. It now uses
`EnsureLinkedThread`, `Reply { target: LinkedThreads { .. } }`, metadata effects,
and normalized `thread_links`.

The next simplification pass should remove compatibility leftovers and collapse
workflow-only ceremony. Best payoff:

1. Delete or split legacy `MattermostSupportNotifier`.
2. Make `UserThreadRun` and `workflow::apply_action` synchronous effect builders.
3. Move user workflow loop out of `handler.rs`.
4. Split the giant handler test module by behavior.

## Refactor Candidates

### 1. Delete the legacy engineer notifier path (done)

Original state:

- Normal workflow no longer calls `MattermostSupportNotifier::ensure_engineer_thread`,
  `notify_engineer`, `mirror_user_message`, `mirror_bot_message`, or
  `notify_status_update`.
- These methods still live in `notifier.rs`, use legacy `EngineerThreadRef`, and
  preserve the old direct-side-effect model.
- `MattermostSupportNotifier` is still publicly exported from `lib.rs`.
- `EngineerThreadRef` is still public in `state.rs`, but no production path uses
  it except the legacy notifier.

Relevant locations:

- `lib/support-bot/src/notifier.rs:12`
- `lib/support-bot/src/notifier.rs:30`
- `lib/support-bot/src/notifier.rs:75`
- `lib/support-bot/src/notifier.rs:111`
- `lib/support-bot/src/notifier.rs:135`
- `lib/support-bot/src/notifier.rs:155`
- `lib/support-bot/src/state.rs:13`
- `lib/support-bot/src/lib.rs:45`
- `lib/support-bot/src/lib.rs:47`

Applied:

- Kept only what debug export still needs: HTML file upload/posting and shared
  formatting helpers.
- Renamed the remaining type to `DebugReportPoster`.
- Removed legacy engineer-thread methods, `EngineerThreadRef`, and the public
  `MattermostSupportNotifier` export.

Expected result:

- `notifier.rs` shrinks sharply.
- One old state type disappears.
- Normal support flow has one mental model: ThreadEffects, not direct posts.

### 2. Make `UserThreadRun` a synchronous effect accumulator (done)

Original state:

- `UserThreadRun::reply(...)` and `mirror_user_message(...)` are `async`, but do
  not await anything. They only push `ThreadEffect`s.
- `workflow::apply_action(...)` is async mostly because it calls
  `run.reply(...).await`.
- `ctx` is passed into `reply(...)` but unused. `ctx` is passed into
  `mirror_user_message(...)` only to build a source link.

Relevant locations:

- `lib/support-bot/src/user_thread_run.rs:73`
- `lib/support-bot/src/user_thread_run.rs:95`
- `lib/support-bot/src/workflow.rs:16`
- `lib/support-bot/src/workflow.rs:32`
- `lib/support-bot/src/handler.rs:222`
- `lib/support-bot/src/handler.rs:315`

Applied:

- Made `reply(...)`, `mirror_user_message(...)`, and `workflow::apply_action(...)`
  synchronous.
- Pass `source_link: String` into `mirror_user_message(...)` instead of all
  `ThreadContext`.
- Keep `process_tool_calls(...)` async because tool execution is async.

Expected result:

- Fewer fake awaits.
- Easier tests.
- Clearer separation: async boundary is LLM/tool/Mattermost, not effect assembly.

### 3. Extract user workflow loop from `handler.rs` (done)

Current state:

- `handler.rs` is 758 lines and owns routing, engineer debug handling, user
  snapshot loading, live control reaction checks, LLM loop, tool execution,
  engineer-link bootstrap, tool-loop-limit reporting, and final effect assembly.
- `user_thread_run.rs` and `workflow.rs` helped, but the main loop still makes
  `SupportBotHandler` the god object.

Relevant locations:

- `lib/support-bot/src/handler.rs:168`
- `lib/support-bot/src/handler.rs:225`
- `lib/support-bot/src/handler.rs:381`
- `lib/support-bot/src/handler.rs:501`
- `lib/support-bot/src/handler.rs:537`

Applied:

- Moved the user-thread loop and tool-call batch processing from
  `lib/support-bot/src/handler.rs` into
  `lib/support-bot/src/handler/user_workflow.rs`.
- Kept `SupportBotHandler` as the coordinator while preserving behavior and
  metrics/reporting semantics.

Expected result:

- `handler.rs` becomes a smaller coordinator.
- Workflow internals are local to a dedicated module.

### 4. Split `tests/handler.rs` (done)

Original state:

- `lib/support-bot/src/tests/handler.rs` is 1329 lines with 22 tests plus large
  fixtures/mocks.
- It now covers several separate concepts: route behavior, LLM loop, workflow
  tools, metadata lifecycle, linked engineer effects, control reactions, and
  debug report handling.

Applied:

- Replaced monolithic `lib/support-bot/src/tests/handler.rs` with
  `lib/support-bot/src/tests/handler/mod.rs` for shared fixtures/mocks.
- Split behavior tests into:
  `lib/support-bot/src/tests/handler/routing.rs`,
  `lib/support-bot/src/tests/handler/user_workflow.rs`,
  `lib/support-bot/src/tests/handler/engineer_thread.rs`,
  `lib/support-bot/src/tests/handler/control_reactions.rs`,
  `lib/support-bot/src/tests/handler/debug_report.rs`.
- Updated `lib/support-bot/src/handler.rs` test module path to
  `tests/handler/mod.rs`.

Expected result:

- Less review friction.
- Smaller compile-error blast radius when changing one behavior.
- Easier to evolve user workflow tests after extraction.

### 5. Collapse workflow tool boilerplate

Current state:

- `WorkflowTool` has three constructors, a private `WorkflowToolKind`, repeated
  schema branches, repeated argument parsing, then maps to `SupportAction`.
- This is fine for three tools, but it is more structure than behavior.

Relevant locations:

- `lib/support-bot/src/tools.rs:120`
- `lib/support-bot/src/tools.rs:128`
- `lib/support-bot/src/tools.rs:160`
- `lib/support-bot/src/tools.rs:196`

Recommendation:

- Replace constructor trio with static specs plus one `match call.name.as_str()`.
- Or define a small `WorkflowToolSpec { name, description, schema, parse }` table.
- Keep `SupportAction` as the public internal boundary; it is useful.

Expected result:

- Fewer private types.
- Easier to scan supported workflow actions.

### 6. Normalize support metadata types

Current state:

- `SupportPostMetadata` is also used as `thread_metadata` for engineer linked
  thread creation.
- `SupportPostKind::EngineerThread` sounds like thread metadata, while the type
  name says post metadata.
- `conversation.rs` wraps metadata helpers with thin aliases
  (`load_state`, `store_state`, `load_trace`, `with_trace_metadata`) that mostly
  forward to `metadata.rs`.

Relevant locations:

- `lib/support-bot/src/metadata.rs:43`
- `lib/support-bot/src/metadata.rs:90`
- `lib/support-bot/src/conversation.rs:96`
- `lib/support-bot/src/conversation.rs:107`
- `lib/support-bot/src/handler.rs:528`

Recommendation:

- Introduce a small `SupportThreadMetadata` only if thread metadata needs a
  distinct shape; otherwise rename `SupportPostMetadata` to neutral
  `SupportMetadata`.
- Consider deleting the thin `conversation.rs` metadata wrappers and importing
  `metadata::{load_thread_state, store_thread_state, ...}` directly.

Expected result:

- Less semantic mismatch.
- Fewer wrapper functions.

### 7. Make debug export the only explicit direct-side-effect exception

Current state:

- Normal workflow is effect-driven after RFC 0007.
- Debug HTML export still uploads a file and creates a Mattermost post directly
  inside `handle_debug_export_html(...)`, then returns a reply effect.
- This is acceptable as an engineer-only debug operation, but it should be named
  and isolated so the old notifier model does not leak back into workflow code.

Relevant locations:

- `lib/support-bot/src/debug_export.rs:13`
- `lib/support-bot/src/debug_export.rs:97`
- `lib/support-bot/src/notifier.rs:195`
- `lib/support-bot/src/notifier.rs:276`

Recommendation:

- After deleting legacy notifier methods, keep debug upload in a narrow module.
- Add a short comment/doc note that debug export is an intentional direct
  Mattermost side effect because `ThreadEffect` has no file attachment variant.
- Do not add a new `ThreadEffect` yet unless more attachment use cases appear.

Expected result:

- Clear boundary.
- No new generic effect machinery for one debug feature.

## Suggested Order

1. Done: synchronous `UserThreadRun`/`workflow` cleanup.
2. Done: split/delete legacy notifier and `EngineerThreadRef`.
3. Extract `user_workflow.rs` from `handler.rs`. Bigger move, easier after step 1.
4. Split handler tests after extraction. Best done once module boundaries settle.
5. Metadata/tool boilerplate cleanup as small opportunistic follow-ups.

## Checks

Applied follow-up refactor for items 1 and 2.

Commands run:

```bash
cargo fmt -p support-bot
cargo test -p support-bot
cargo clippy -p support-bot -- -D warnings
```

All passed.
