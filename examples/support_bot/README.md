# support_bot example

Runnable `support-bot` example on top of `thread-bot`.

## What this example wires

- `SupportBotBuilder` with default workflow tools.
- `InstructionRepository` loaded from Markdown files under `instructions/`.
- Optional remote MCP tool registration from env-driven `ToolConfig`.
- Optional Prometheus/OpenMetrics metric families for Mattermost WS,
  thread actors, and support workflow counters/durations.
- User-channel routing and engineer-channel routing.
- Engineer-thread mirroring, explicit engineer notifications, finish status
  updates, close notifications, and debug report export.
- Mattermost runtime via `mattermost-bot` + `ThreadBotPlugin`.
- PostgreSQL persistence via `PgThreadStore`.

## Quick start

1. Start local infra (from repo root):

```bash
just test-env-start
```

2. Configure env (minimum manual input: Mattermost bot token, LLM, and channel IDs):

```bash
export MM_BASE_PATH=http://localhost:8065
export MM_BEARER_TOKEN=YOUR_BOT_TOKEN

export THREAD_BOT_DATABASE_URL=postgres://test:test@localhost:5433/thread_bot_test
export THREAD_BOT_DB_MAX_CONNECTIONS=5

export SUPPORT_LLM_BASE_URL=http://localhost:11434
export SUPPORT_LLM_MODEL=gpt-4o-mini
export SUPPORT_LLM_API_KEY=
export SUPPORT_LLM_TIMEOUT_SECS=45
# Optional: inline prompt or a file path. File wins when both are set.
# export SUPPORT_SYSTEM_PROMPT="You are a support assistant..."
# export SUPPORT_SYSTEM_PROMPT_FILE=examples/support_bot/system-prompt.md

export SUPPORT_USER_CHANNEL_IDS=YOUR_SUPPORT_CHANNEL_ID
export SUPPORT_ENGINEER_CHANNEL_ID=YOUR_ENGINEER_CHANNEL_ID
# Optional: only handle user threads whose first message contains these texts.
# export SUPPORT_ADMISSION_REQUIRED_TEXTS=@xxxduty

export SUPPORT_INSTRUCTIONS_ROOT=examples/support_bot/instructions

# Optional remote MCP tools
# export SUPPORT_REMOTE_MCP_NAMES=logs,metrics
# export SUPPORT_REMOTE_MCP_LOGS_URL=http://localhost:9001/mcp
# export SUPPORT_REMOTE_MCP_LOGS_AUTH_HEADER="Bearer dev-token"
# export SUPPORT_REMOTE_MCP_LOGS_TIMEOUT_SECS=30
# export SUPPORT_REMOTE_MCP_METRICS_URL=http://localhost:9002/mcp
```

3. Run:

```bash
cargo run -p support-bot-example
```

## Notes

- Defaults are already baked into the example for everything except:
  - `MM_BEARER_TOKEN` (required),
  - LLM settings (`SUPPORT_LLM_BASE_URL`, `SUPPORT_LLM_MODEL`, and optional `SUPPORT_LLM_API_KEY`),
  - channel routing (`SUPPORT_USER_CHANNEL_IDS`, `SUPPORT_ENGINEER_CHANNEL_ID`).
- `THREAD_BOT_DATABASE_URL` defaults to `postgres://test:test@localhost:5433/thread_bot_test`.
- `MM_BASE_PATH` defaults to `http://localhost:8065`.
- `SUPPORT_SYSTEM_PROMPT_FILE` or `SUPPORT_SYSTEM_PROMPT` overrides the default support-bot system prompt.
- `SUPPORT_ADMISSION_REQUIRED_TEXTS` is an optional comma-separated list of
  case-sensitive substrings that must all appear in the first user-thread
  message. For example, `@xxxduty` ignores unrelated channel traffic before
  engineer-thread creation or LLM processing.
- `SUPPORT_REMOTE_MCP_NAMES` is a comma-separated list. For each name `x`, provide `SUPPORT_REMOTE_MCP_X_URL` and optionally `SUPPORT_REMOTE_MCP_X_AUTH_HEADER` / `SUPPORT_REMOTE_MCP_X_TIMEOUT_SECS` (name is uppercased, `-` becomes `_`).
- The included instruction files are placeholders; replace them with your runbooks.
- Instruction repository lint issues are logged at `error` during startup, but
  they do not stop the example process. Invalid documents are skipped by the
  repository until fixed.
- Support thread state is stored in thread metadata under `support_bot`. The
  request status can be `active`, `ignored`, `finished`, or `stopped`.
  Admission-rejected threads are persisted as `ignored`; active threads become
  `finished` when the model calls `finish_request`.
- The first handled user thread creates an engineer thread containing a source
  link and quoted root request in `SUPPORT_ENGINEER_CHANNEL_ID`.
  Later user messages, bot replies, `notify_engineer` calls, and
  `finish_request` status updates are posted as replies in that engineer thread.
- Engineer thread commands:
  - `!support debug-report` to export full source support thread as a self-contained HTML attachment (includes tool-trace section).
- On tool-loop limit failure, the bot tells the user it stopped the thread,
  notifies the engineer thread with trace summary, and asks engineers to run
  `!support debug-report` if they want the full HTML snapshot.
- Metrics are registered into an in-process `prometheus_client::Registry`.
  Use the crate re-export (`support_bot::prometheus_client`) so applications do
  not need to match the exact dependency version. This example wires the
  registry and labels every series with `bot="support_bot"`; add your own HTTP
  `/metrics` endpoint around that registry if you want scraping.
- [`TOOL_AUTHORING_GUIDE.md`](TOOL_AUTHORING_GUIDE.md) is a copy-paste handoff
  for coding agents that need to add new support tools.
