# Support Bot Progress Report (2026-05-03)

## Scope

This report summarizes completed work for Layer 4 `support-bot` since the initial plan discussion:

- Remote MCP tools loading from `ToolConfig`
- Prompt/instruction bootstrap builder
- Runnable `examples/support_bot`
- Failure-path hardening for tool loop limit
- Debug HTML export flow via engineer command
- Trace persistence redesign (message metadata, not thread state)
- Mattermost API compatibility fix for `permalink` embeds
- Notes update for post-generation API patching

## Completed Work

### 1) Remote MCP tools

Implemented remote MCP registration and execution path:

- Added remote MCP provider module:
  - `lib/support-bot/src/remote_mcp.rs`
- Added registration API:
  - `register_remote_mcp_tools(&mut ToolRegistry, &ToolConfig)`
- Behavior:
  - Reads endpoints from `ToolConfig.remote_mcp_endpoints`
  - Calls `tools/list` and registers tools into `ToolRegistry`
  - Namespaces tools as `<endpoint_name>.<tool_name>`
  - Proxies calls via `tools/call`
- Exported via:
  - `lib/support-bot/src/tools.rs` and `lib/support-bot/src/lib.rs`

Testing:

- Added `wiremock` tests for `tools/list` and `tools/call`
- Validated auth header forwarding

### 2) Prompt/instruction bootstrap

Introduced builder-based assembly with default policy prompt:

- Added:
  - `lib/support-bot/src/builder.rs`
- New API:
  - `SupportBotBuilder`
  - `DEFAULT_SUPPORT_SYSTEM_PROMPT`
- Builder capabilities:
  - Registers default workflow tools
  - Registers remote MCP tools
  - Registers `InstructionRepository` as tool
  - Supports optional debug handler
- Prompt policy includes explicit instruction:
  - use `instructions` tool first
  - do not invent runbooks/instructions

### 3) Runnable support bot example

Added full runnable example under `examples/`:

- New crate:
  - `examples/support_bot/`
- Files:
  - `examples/support_bot/src/main.rs`
  - `examples/support_bot/README.md`
  - `examples/support_bot/instructions/**`
- Included:
  - env-driven config bootstrap
  - LLM client setup
  - instruction manifest loading (`yaml`)
  - postgres store wiring
  - Mattermost bot runtime wiring
  - optional remote MCP endpoint env mapping
- Added to workspace:
  - root `Cargo.toml`
- Added references in root README

Defaults were expanded so the common setup requires filling mainly:

- `MM_BEARER_TOKEN`
- LLM settings
- support channel id(s)

### 4) Failure handling hardening

Adjusted behavior when tool loop limit is reached:

- User-facing message is now generic and non-technical
- Detailed diagnostic message is sent to engineer thread
- Engineer message includes explicit command to export report:
  - `!support debug-report`
- Handler stops thread via `ThreadEffect::MarkStopped`
- Removed trace persistence via `UpdateMessage` patching of user post metadata
  - this removed a likely source of Mattermost `400 Bad Request` during `patch_post`

### 5) Debug HTML export flow

Implemented on-demand export in engineer thread:

- Command:
  - `!support debug-report`
- Behavior:
  - Resolves source support thread from engineer thread root props
  - Fetches source posts from Mattermost
  - Uploads self-contained HTML file to engineer thread
  - Includes per-message tool traces in the report

Added detailed `tracing::info`/`warn` diagnostics for:

- route resolution (`user/engineer/ignored`)
- debug command parsing path
- source thread lookup
- report generation/upload/posting

### 6) Trace persistence redesign

Refactored trace model to avoid fragile coupling and keep history aligned with user messages:

- Removed trace storage from `SupportThreadState`
- Added DB-only message metadata update path:
  - `ThreadEffect::SetMessageMetadata { post_id, metadata }`
  - handled by thread actor through `store.set_message_metadata(...)`
- LLM traces (`support_bot.llm_trace`) are now written to the **trigger user message** metadata
  - not to a newly created bot reply
- `build_llm_messages(...)` now restores:
  - assistant `tool_calls` from stored trace metadata
  - tool result messages from stored trace metadata
- `debug-report` now reads traces from `thread_messages.metadata` in DB

Also fixed metadata persistence gap in `thread-bot`:

- incoming post metadata now stores `post.props` instead of `Null` in `save_incoming_message(...)`
- this ensures trace props survive and are available for report generation

### 7) Mattermost API compatibility fix

Observed runtime deserialization failure due to embed type `permalink`.

Fix applied in generated API model:

- Updated:
  - `lib/mattermost-api/src/models/post_metadata_embeds_inner.rs`
- Added enum variants:
  - `Permalink` with `#[serde(rename = "permalink")]`
  - `Unknown` with `#[serde(other)]` for forward compatibility

### 8) Notes update for regeneration workflow

Updated generation notes with explicit manual diff to re-apply post-generation:

- Updated:
  - `notes/generate_mm_api.md`
- Added patch snippet for the `permalink` embed compatibility change

## Validation

Executed during implementation:

- `cargo check -p support-bot-example`
- `cargo test -p support-bot`
- `cargo test -p support-bot` after trace redesign and `SetMessageMetadata` integration
- `cargo test -p thread-bot` compile path validated; runtime test suite may fail in restricted sandbox due to mock server bind permissions

Note: `wiremock` tests require local port binding; in sandboxed mode this may require elevated execution.

## Commits

- `de8545c` feat(support-bot): add remote MCP loader and wiremock tests
- `8c696f5` feat(support-bot): add builder with instruction-first prompt
- `5204270` feat(support-bot): add runnable example and harden failure handling
- `7fa93e2` refactor(support-bot): replace debug toggle with debug-report
- `1da1ca7` refactor(support-bot): persist traces in message metadata

## Known Current Behavior

- Debug command parser expects `!support ...` or `/support ...`
- `!debug` alone is not recognized
- Supported built-in engineer command:
  - `!support debug-report`
- `state/trace/retry` command set is still pending implementation

## Suggested Next Task

Implement engineer control commands:

1. `!support state`
2. `!support trace`
3. `!support retry`

And optionally add thread export:
- add `!support retry` flow to restart stopped threads safely
