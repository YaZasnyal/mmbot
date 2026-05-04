# support_bot example

Runnable `support-bot` example on top of `thread-bot`.

## What this example wires

- `SupportBotBuilder` with default workflow tools.
- `InstructionRepository` loaded from `instructions/manifest.yaml`.
- Optional remote MCP tool registration from env-driven `ToolConfig`.
- Optional Prometheus/OpenMetrics metric families for Mattermost WS,
  thread actors, and support workflow counters/durations.
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

export SUPPORT_INSTRUCTIONS_ROOT=examples/support_bot/instructions
export SUPPORT_INSTRUCTIONS_MANIFEST=examples/support_bot/instructions/manifest.yaml

# Optional remote MCP tools
# export SUPPORT_REMOTE_MCP_NAMES=logs,metrics
# export SUPPORT_REMOTE_MCP_LOGS_URL=http://localhost:9001/mcp
# export SUPPORT_REMOTE_MCP_LOGS_AUTH_HEADER="Bearer dev-token"
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
  - channel routing (`SUPPORT_USER_CHANNEL_IDS`, optional `SUPPORT_ENGINEER_CHANNEL_ID`).
- `THREAD_BOT_DATABASE_URL` defaults to `postgres://test:test@localhost:5433/thread_bot_test`.
- `MM_BASE_PATH` defaults to `http://localhost:8065`.
- `SUPPORT_SYSTEM_PROMPT_FILE` or `SUPPORT_SYSTEM_PROMPT` overrides the default support-bot system prompt.
- If `SUPPORT_ENGINEER_CHANNEL_ID` is unset, engineer notifications stay in the same thread.
- `SUPPORT_REMOTE_MCP_NAMES` is a comma-separated list. For each name `x`, provide `SUPPORT_REMOTE_MCP_X_URL` and optionally `SUPPORT_REMOTE_MCP_X_AUTH_HEADER` (name is uppercased, `-` becomes `_`).
- The included instruction files are placeholders; replace them with your runbooks.
- Engineer thread commands:
  - `!support debug-report` to export full source support thread as a self-contained HTML attachment (includes tool-trace section).
- On tool-loop fatal stop, the bot also uploads an HTML thread snapshot to the engineer thread automatically.
- Metrics are registered into an in-process `prometheus_client::Registry`.
  Use the crate re-export (`support_bot::prometheus_client`) so applications do
  not need to match the exact dependency version. This example wires the
  registry and labels every series with `bot="support_bot"`; add your own HTTP
  `/metrics` endpoint around that registry if you want scraping.
- [`TOOL_AUTHORING_GUIDE.md`](TOOL_AUTHORING_GUIDE.md) is a copy-paste handoff
  for coding agents that need to add new support tools.
