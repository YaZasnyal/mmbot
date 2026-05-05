# Support Bot Review (2026-05-05)

## Scope

Reviewed the current `feature/support-bot-layer` state before merging into
`main`, with emphasis on changes after the 2026-05-03 review:

- `lib/support-bot/src/handler.rs`
- `lib/support-bot/src/user_thread_run.rs`
- `lib/support-bot/src/workflow.rs`
- `lib/support-bot/src/notifier.rs`
- `lib/support-bot/src/remote_mcp.rs`
- `lib/support-bot/src/debug_export.rs`
- `examples/support_bot/src/main.rs`
- `lib/thread-bot/src/actor.rs`

## Executive Summary

The feature is much closer to mergeable than in the previous review. The main
structural cleanup happened: conversation/history logic, workflow actions,
debug export, output sanitization, and run state are now split out of
`handler.rs`. The previous P0s around malformed LLM history, fake
`wait_for_user`, and missing finished state are addressed. `thread-bot` now
treats metadata persistence effects as critical and stops the thread when they
fail.

Remaining merge risks are concentrated around side effects that still happen
inside the handler before returned effects are persisted. Tool-call overflow,
remote MCP timeouts, and debug-report support status were addressed in the
follow-up pass.

## P0 - Accepted Risk

### 1) Engineer-channel Mattermost side effects still happen before DB state/trace effects

Follow-up: accepted as a known risk for now. A durable/idempotent side-effect
pipeline is deliberately deferred because it is larger than the current merge
needs.

Location:

- `lib/support-bot/src/handler.rs:161`
- `lib/support-bot/src/handler.rs:177`
- `lib/support-bot/src/user_thread_run.rs:63`
- `lib/support-bot/src/workflow.rs:71`
- `lib/support-bot/src/workflow.rs:112`
- `lib/support-bot/src/notifier.rs:27`

The handler still creates/reuses the engineer thread, mirrors user/bot messages,
posts `notify_engineer`, and posts finish status updates by calling Mattermost
directly before `ThreadEffect::SetMessageMetadata` and
`ThreadEffect::SetThreadMetadata` are returned and executed.

If a Mattermost post succeeds but the handler is interrupted, process exits, or
the later metadata effects fail, the source thread can lose the
`engineer_thread` ref and/or LLM trace. A later run can create duplicate engineer
roots or duplicate status/mirror posts.

Recommendation:

- Move engineer-channel posts behind idempotent `ThreadEffect` variants or a
  support-bot outbox with durable idempotency keys.
- At minimum, make `ensure_engineer_thread` search/reuse an existing engineer
  root by `support_bot.source_thread_id` before creating a new one.
- Add regression coverage for "engineer root was created but state metadata was
  not persisted; next run reuses the existing root".

## P1 - High Priority

### 2) Tool-call overflow is logged but hidden from the LLM conversation (fixed)

Follow-up: fixed by preserving the full assistant tool-call message and adding
synthetic tool error results for calls skipped by `max_tool_calls_per_round`.
Covered by `tool_call_overflow_is_visible_to_next_llm_round`.

Location:

- `lib/support-bot/src/handler.rs:255`
- `lib/support-bot/src/handler.rs:260`
- `lib/support-bot/src/user_thread_run.rs:46`

When the model requests more tool calls than
`max_tool_calls_per_round`, the handler truncates the list with `.take(...)`,
logs a warning, and records only the kept calls in the assistant tool-call
message. The omitted calls do not get tool error messages and are not visible in
the persisted trace.

This keeps the OpenAI tool-call ordering valid, but it hides the model's real
request and gives the next round no explicit signal that some requested actions
were rejected.

Recommendation:

- Preserve the full requested assistant tool-call message in trace.
- Return explicit tool error messages for omitted calls, or stop the run with a
  clear diagnostic and engineer notification.
- Add a test where the LLM requests `max_tool_calls_per_round + 1` calls and
  assert the overflow is visible in the next LLM request/report.

### 3) Remote MCP HTTP calls have no timeout (fixed)

Follow-up: fixed by adding `RemoteMcpEndpoint.timeout`, wiring
`SUPPORT_REMOTE_MCP_<NAME>_TIMEOUT_SECS` in the example, and applying the value
to the reqwest client. Covered by `remote_mcp_client_applies_timeout`.

Location:

- `lib/support-bot/src/remote_mcp.rs:77`
- `lib/support-bot/src/remote_mcp.rs:89`
- `lib/support-bot/src/remote_mcp.rs:120`

`RemoteMcpClient` builds a default reqwest client without a timeout. A slow or
hung remote MCP endpoint can block startup `tools/list` or an in-flight
`tools/call` indefinitely, which stalls the support thread actor.

Recommendation:

- Add a timeout to `RemoteMcpEndpoint` or reuse a tool timeout from
  `ToolConfig`.
- Apply it to the reqwest client and log elapsed time for `tools/list` and
  `tools/call`.
- Add a timeout test with a delayed wiremock response.

### 4) Finish status notification can fail the run after mutating in-memory state

Location:

- `lib/support-bot/src/workflow.rs:112`
- `lib/support-bot/src/workflow.rs:115`
- `lib/support-bot/src/user_thread_run.rs:116`

`finish_request` first mutates `SupportThreadState` to `Finished`, then posts a
Mattermost status update. If that post fails, `apply_action` returns an error,
the handler returns no persistence effects, and the finished state is lost. If
the Mattermost post actually succeeded but the response failed, a retry can
produce duplicate finish status updates.

Recommendation:

- Persist finished state before optional status notification, or make the status
  notification best-effort/idempotent and return a tool result that records the
  notification failure.
- Add a test where `notify_status_update` fails and assert the source thread
  still receives state/trace persistence effects.

## P2 - Medium Priority

### 5) Example treats an empty `SUPPORT_LLM_API_KEY` as a real API key

Follow-up: accepted as low priority and intentionally left unchanged.

Location:

- `examples/support_bot/src/main.rs:110`
- `examples/support_bot/README.md:37`

The README suggests `export SUPPORT_LLM_API_KEY=`, but config loading uses
`std::env::var("SUPPORT_LLM_API_KEY").ok()`. That turns an empty env var into
`Some("")`, so the LLM client sends an empty bearer token header.

Recommendation:

- Normalize empty/whitespace API key values to `None`.
- Either remove the empty export from the README or document that the variable
  should be unset for local LLM endpoints without auth.

### 6) Remote MCP registration is optional-only

Location:

- `lib/support-bot/src/remote_mcp.rs:16`
- `lib/support-bot/src/remote_mcp.rs:30`

The previous all-or-nothing startup risk is fixed in the resilient direction:
endpoint init/list failures now warn and continue. The tradeoff is that there is
no way to mark an endpoint as required, so a misconfigured critical tool backend
can silently remove tools from the bot at startup.

Recommendation:

- Add `required: bool` to `RemoteMcpEndpoint`, defaulting to `false`.
- For required endpoints, fail builder startup on init/list errors.
- Expose registered/missing endpoint status in logs or a future debug command.

## P3 - Polish

### 7) Debug report status shows thread-bot status, not support status (fixed)

Follow-up: fixed by displaying separate `thread_status` and `support_status`
summary fields in the HTML report.

Location:

- `lib/support-bot/src/debug_export.rs:151`
- `lib/support-bot/src/debug_export.rs:160`

The report summary uses `ThreadStatus` for the headline `status`, while the
support lifecycle is only present inside the state JSON. A finished support
request can therefore show `Resolved` at top level and `finished` in the state
JSON.

Recommendation:

- Display both `thread_status` and `support_status` as separate summary fields.

## Checks Run

- `cargo test -p support-bot` - passed, 50 tests after follow-up fixes.
- `cargo test -p thread-bot` - passed, 36 unit tests, 34 pg-store tests, 1 doctest.
- `cargo clippy -p support-bot -p thread-bot -- -D warnings` - passed.
- `cargo check -p support-bot-example` - passed after remote MCP timeout wiring.
- `cargo clippy -p support-bot -p support-bot-example -- -D warnings` - passed after follow-up fixes.

## Merge Readiness

The remaining blocker-level item is the accepted engineer-channel side-effect
ordering risk. With that explicitly deferred, the branch is much closer to a
practical merge candidate; the remaining open items are operational tradeoffs
rather than immediate correctness blockers for the current scope.
