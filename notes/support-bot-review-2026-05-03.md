# Support Bot Review (2026-05-03)

## Scope

Reviewed the new `support-bot` implementation and its integration points with `thread-bot`:

- `lib/support-bot/src/handler.rs`
- `lib/support-bot/src/notifier.rs`
- `lib/support-bot/src/remote_mcp.rs`
- `lib/support-bot/src/llm.rs`
- `lib/support-bot/src/instructions.rs`
- `lib/support-bot/src/builder.rs`
- relevant `thread-bot` effect execution paths

This review focuses on code quality, architecture, immediate refactoring, and operational logs.

## Executive Summary

The current implementation is a good Layer 4 skeleton: `support-bot` stays above `thread-bot`, has a clear `LlmClient` abstraction, uses `ThreadEffect`s for Mattermost-facing work, separates instruction lookup from the prompt, and now stores LLM tool traces in DB message metadata instead of patching Mattermost posts.

The main risk is not the basic structure; it is the behavior at boundaries:

- LLM history reconstruction can produce malformed OpenAI-style tool history.
- Some external side effects happen before the handler returns effects, so failures can leave Mattermost and DB state out of sync.
- `wait_for_user` does not actually pause the thread.
- remote MCP registration is a startup hard dependency.
- several important paths lack structured logs around correlation IDs, round numbers, tool names, and state transitions.

## P0 - Must Fix Before Production

### 1) LLM history reconstruction duplicates assistant tool calls (done)

Location:

- `lib/support-bot/src/handler.rs:340`
- `lib/support-bot/src/handler.rs:364`
- `lib/support-bot/src/handler.rs:372`
- `lib/support-bot/src/handler.rs:375`

`build_llm_messages` pushes each stored Mattermost message, assigns `tool_calls` from `load_tool_calls(&message.metadata)`, then appends the full stored trace with `messages.extend(load_trace(...))`.

That means a user message can receive assistant `tool_calls` even though it has role `user`, and the same assistant tool-call message is appended again from the trace. For OpenAI-compatible chat completions, tool-call ordering is strict: assistant tool calls must be followed by matching tool messages. Duplicating or attaching tool calls to the wrong role can make later LLM requests invalid or confuse the model.

Recommendation:

- Do not attach `tool_calls` to the Mattermost-visible user/bot message.
- Treat saved trace as synthetic chat messages inserted after the trigger message.
- Add a test that runs a two-turn conversation and asserts the second LLM request has this shape:
  - system messages
  - user trigger message
  - assistant message with `tool_calls`
  - tool result messages
  - visible bot/user messages as plain assistant/user messages

### 2) External engineer mirroring happens inside handler before DB state effect is persisted

Location:

- `lib/support-bot/src/handler.rs:153`
- `lib/support-bot/src/handler.rs:154`
- `lib/support-bot/src/handler.rs:161`
- `lib/support-bot/src/handler.rs:190`
- `lib/support-bot/src/handler.rs:511`
- `lib/support-bot/src/notifier.rs:20`
- `lib/support-bot/src/notifier.rs:37`

`handle_user_thread` creates or mirrors to the engineer thread by directly calling Mattermost before returning `ThreadEffect::SetThreadMetadata`. If the handler is interrupted after the Mattermost post is created but before returned effects are executed, or if effect execution later fails, the source thread metadata may not store `engineer_thread`. On the next run, `ensure_engineer_thread` can create another engineer root thread.

Recommendation:

- Prefer moving engineer notification/mirroring into explicit `ThreadEffect` variants owned by `thread-bot`, or into a support-bot side-effect queue with idempotency keys.
- At minimum, add an idempotency guard in engineer thread props and search/reuse existing engineer roots by `source_thread_id` before creating a new one.
- Persist `engineer_thread` as early and explicitly as possible once created.

### 3) `wait_for_user` does not pause processing (done)

Location:

- `lib/support-bot/src/tools.rs:82`
- `lib/support-bot/src/handler.rs:472`

The workflow tool says "Pause active handling until the user sends more information", but `SupportAction::WaitForUser` only returns a tool result. The LLM loop continues immediately and may produce an assistant message or call more tools. There is no state transition such as `waiting_for_user`, no stop/reschedule semantics, and no metadata that explains why the bot is waiting.

Recommendation:

- Add state fields such as `status: active | waiting_for_user | finished | escalated` and `waiting_reason`.
- When `wait_for_user` is called, return a user-visible reply when needed, persist state, and exit the current LLM loop.
- Add tests that a `wait_for_user` tool call causes no extra LLM round unless the design explicitly requires one.

## P1 - High Priority

### 4) Remote MCP registration is an all-or-nothing startup dependency

Location:

- `lib/support-bot/src/builder.rs:88`
- `lib/support-bot/src/remote_mcp.rs:15`
- `lib/support-bot/src/remote_mcp.rs:17`

`SupportBotBuilder::build` fails if any configured MCP endpoint fails `tools/list`. This is useful in strict deployments, but brittle for support operations: one optional logs/search backend can prevent the whole bot from starting.

Recommendation:

- Add config for endpoint criticality: `required: bool`.
- For optional endpoints, log registration failure and continue.
- Register a diagnostic pseudo-tool or expose endpoint health in `!support state`.

### 5) Tool-call truncation silently drops calls over the per-round limit

Location:

- `lib/support-bot/src/handler.rs:218`
- `lib/support-bot/src/handler.rs:221`

The handler uses `.take(max_tool_calls_per_round)` and ignores remaining model-requested tool calls. The assistant trace then contains only the truncated list, not the fact that truncation occurred. The model never receives an explicit error for omitted calls.

Recommendation:

- If the model requests too many tools, return tool error messages for the omitted calls or stop with a clear diagnostic.
- Log requested vs executed counts.
- Add a trace entry that records truncation.

### 6) ThreadEffect execution swallows failures

Location:

- `lib/thread-bot/src/actor.rs:544`
- `lib/thread-bot/src/actor.rs:585`
- `lib/thread-bot/src/actor.rs:595`

`execute_effects` logs failures for posting replies and setting metadata but keeps going. For support-bot, this matters because trace persistence, state persistence, replies, and MarkStopped/MarkResolved can diverge.

Recommendation:

- Consider making critical effects fail the handler run or trigger a retry/dead-letter state.
- Add per-effect criticality, or order support-bot effects so critical persistence happens before non-critical posts.
- Add metrics/logs for effect failure counts by effect type.

### 7) Debug command parser can false-match prefixes (done)

Location:

- `lib/support-bot/src/debug.rs:33`
- `lib/support-bot/src/debug.rs:37`

`starts_with(prefix)` means `!supportive state` matches the `!support` prefix and parses `ive` as a command. This is minor but easy to fix.

Recommendation:

- Require exact prefix followed by whitespace or end of string.
- Add parser tests for `!supportive state`.

### 8) `finish_request` resolves immediately but does not store support-specific final state (done)

Location:

- `lib/support-bot/src/handler.rs:480`

`finish_request` emits `MarkResolved`, but does not persist a support-specific summary/status before closure. If later reporting relies on `SupportThreadState`, the final summary exists only inside the tool result trace.

Recommendation:

- Store `finished_summary` and `status = finished` in `SupportThreadState`.
- Persist it before `MarkResolved`.

## P2 - Medium Priority

### 9) `SupportThreadState` has unused fields and no explicit lifecycle

Location:

- `lib/support-bot/src/state.rs`

`selected_instruction_ids` and `compact_history` exist, but the handler does not update them. The operational state is mostly implicit in message traces and Mattermost thread status.

Recommendation:

- Decide which state is canonical:
  - thread lifecycle and engineer thread ref in `SupportThreadState`
  - tool/LLM audit in message metadata
  - human-readable source posts in Mattermost
- Add explicit lifecycle enum and update it from workflow actions.
- Remove or postpone fields until they are populated.

### 10) Handler is doing too many jobs

Location:

- `lib/support-bot/src/handler.rs`

`SupportBotHandler` currently owns routing, engineer command handling, LLM message building, tool-loop orchestration, workflow action execution, debug export, trace persistence, and mirroring. This is still manageable, but it is becoming the central knot.

Recommended immediate refactor:

- Extract `conversation.rs`:
  - `build_llm_messages`
  - trace load/store helpers
  - `result_to_message`
- Extract `workflow.rs`:
  - `apply_action`
  - workflow state transitions
- Extract `debug_export.rs`:
  - `handle_debug_export_html`
  - report collection from Mattermost/store

Keep `handler.rs` as the route/orchestration layer.

### 11) README is stale

Location:

- `lib/support-bot/README.md:41`

README still says the bot "supports a per-thread debug flag for streaming tool calls and tool results to the engineer thread", but the current behavior is on-demand `!support debug-report`.

Recommendation:

- Update README to reflect current debug-report flow.
- Document pending commands: `state`, `trace`, `retry`.

### 12) Remote MCP client has no per-endpoint timeout or retry policy

Location:

- `lib/support-bot/src/remote_mcp.rs:64`

The client uses default reqwest timeout behavior. LLM tool loops should not hang on slow remote MCP calls.

Recommendation:

- Add `timeout` to `RemoteMcpEndpoint`.
- Consider bounded retry only for idempotent `tools/list`; be careful retrying `tools/call`.
- Log elapsed time for `tools/list` and `tools/call`.

### 13) HTML report is useful but lacks high-level trace summary

Location:

- `lib/support-bot/src/notifier.rs:198`

The report includes raw per-message trace entries. For real incidents, engineers also need a compact summary: model, number of LLM rounds, tool names, error count, final state, and whether data was truncated.

Recommendation:

- Add a summary panel before messages.
- Include source thread state JSON.
- Include tool-call table: round, tool, duration if available, success/error.

## P3 - Lower Priority / Polish

### 14) Tool argument validation is basic

Location:

- `lib/support-bot/src/tools.rs`

Workflow tools manually read strings from `serde_json::Value`; instruction tools deserialize typed args. Prefer typed args for all local tools.

Recommendation:

- Convert workflow tool args to typed structs.
- Add validation tests for empty strings, wrong types, and extra fields.

### 15) Mattermost quote rendering does not escape all Markdown edge cases

Location:

- `lib/support-bot/src/notifier.rs:275`

`quote_for_mattermost` prefixes lines with `>`, which is mostly fine, but large or structured messages can still render awkwardly.

Recommendation:

- For engineer mirrors, consider fenced blocks or a compact post template with source metadata.
- Add max length and truncation markers to mirrored messages.

## Logs To Add

### P0/P1 logs

- LLM round start/end:
  - `thread_id`
  - `post_id`
  - `round`
  - `message_count`
  - `tool_count_available`
  - `elapsed_ms`
- LLM response:
  - `round`
  - `assistant_content_bytes`
  - `requested_tool_calls`
  - `executed_tool_calls`
  - `truncated_tool_calls`
- Tool execution:
  - `tool_call_id`
  - `tool_name`
  - `tool_kind`
  - `is_error`
  - `result_bytes`
  - `elapsed_ms`
- Workflow actions:
  - `action`
  - `tool_call_id`
  - `state_before`
  - `state_after`
  - `effects_added`
- Engineer thread creation/reuse:
  - `source_thread_id`
  - `source_root_post_id`
  - `engineer_channel_id`
  - `engineer_root_post_id`
  - `created_or_reused`
- Metadata persistence:
  - `thread_id`
  - `post_id`
  - `metadata_key`
  - `trace_entries`
  - `metadata_bytes`

### Useful warning logs

- MCP endpoint registration failure for optional endpoints.
- Tool result truncation when `max_tool_result_bytes` is reached.
- Empty assistant response with no tool calls.
- Debug command rejected because source thread props are missing.
- `SetMessageMetadata` failure for trace persistence.
- `MarkResolved` or `MarkStopped` emitted before state metadata is persisted.

## Recommended Immediate Refactoring Plan

1. Fix LLM history reconstruction and add regression tests.
2. Implement real `wait_for_user` lifecycle semantics.
3. Add explicit support lifecycle fields to `SupportThreadState`.
4. Make engineer-thread creation idempotent or move it behind effects.
5. Split `handler.rs` into conversation, workflow, and debug export helpers. (done)
6. Add structured logs around LLM rounds, tool calls, workflow actions, and metadata persistence.
7. Make remote MCP endpoints optionally non-fatal during startup.
8. Update README and progress notes to match the current debug-report design.

## Tests To Add

- Reconstructed LLM messages preserve valid assistant/tool ordering on the second user turn.
- `wait_for_user` exits the loop and persists waiting state.
- `finish_request` persists final support state before `MarkResolved`.
- Tool-call overflow reports truncation instead of silently dropping calls.
- Optional MCP endpoint failure does not fail builder.
- Debug parser does not match `!supportive`.
- Engineer thread creation is idempotent after interrupted/partial processing.

