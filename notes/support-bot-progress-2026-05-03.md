# Support Bot Progress Report (2026-05-03)

## Scope

This report summarizes completed work for Layer 4 `support-bot` since the initial plan discussion:

- Remote MCP tools loading from `ToolConfig`
- Prompt/instruction bootstrap builder
- Runnable `examples/support_bot`
- Failure-path hardening for tool loop limit
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
- Handler stops thread via `ThreadEffect::MarkStopped`
- Removed trace persistence via `UpdateMessage` patching of user post metadata
  - this removed a likely source of Mattermost `400 Bad Request` during `patch_post`

### 5) Mattermost API compatibility fix

Observed runtime deserialization failure due to embed type `permalink`.

Fix applied in generated API model:

- Updated:
  - `lib/mattermost-api/src/models/post_metadata_embeds_inner.rs`
- Added enum variants:
  - `Permalink` with `#[serde(rename = "permalink")]`
  - `Unknown` with `#[serde(other)]` for forward compatibility

### 6) Notes update for regeneration workflow

Updated generation notes with explicit manual diff to re-apply post-generation:

- Updated:
  - `notes/generate_mm_api.md`
- Added patch snippet for the `permalink` embed compatibility change

## Validation

Executed during implementation:

- `cargo check -p support-bot-example`
- `cargo test -p support-bot`

Note: `wiremock` tests require local port binding; in sandboxed mode this may require elevated execution.

## Commits

- `de8545c` feat(support-bot): add remote MCP loader and wiremock tests
- `8c696f5` feat(support-bot): add builder with instruction-first prompt
- `5204270` feat(support-bot): add runnable example and harden failure handling

## Known Current Behavior

- Debug command parser expects `!support ...` or `/support ...`
- `!debug` alone is not recognized
- Currently implemented built-in command path is for `debug` toggle/status
- `state/trace/retry` command set is still pending implementation

## Suggested Next Task

Implement engineer control commands:

1. `!support state`
2. `!support trace`
3. `!support retry`

And optionally add thread export:

- HTML transcript export to engineer channel
- auto-send on fatal failure
- on-demand send via debug/control command
