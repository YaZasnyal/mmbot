# support bot

This package builds the deployable RFC 0008 ACP-only support bot. Container and
Kubernetes configuration are intentionally left to the deployment environment.

## Configuration

```bash
export MM_BASE_PATH=http://localhost:8065
export MM_BEARER_TOKEN=...
export SUPPORT_USER_CHANNEL_IDS=user-channel-id
export SUPPORT_ENGINEER_CHANNEL_ID=engineer-channel-id
export THREAD_BOT_DATABASE_URL=postgres://test:test@localhost:5433/thread_bot_test
export THREAD_BOT_DB_MAX_CONNECTIONS=5
export THREAD_BOT_DB_ACQUIRE_TIMEOUT_SECS=5

export SUPPORT_QWEN_EXECUTABLE=qwen
export SUPPORT_QWEN_CWD=.
export SUPPORT_QWEN_MODEL=qwen/qwen3.6-35b-a3b
export SUPPORT_QWEN_TIMEOUT_SECS=120
# Optional for local debugging; JSON is the default.
export SUPPORT_LOG_FORMAT=text

cargo run -p support-bot-app
```

The executable is named `support-bot`. Application logs, including fatal
startup errors, are JSON objects written to stdout by default. Set
`SUPPORT_LOG_FORMAT=text` for human-readable local output.

### PostgreSQL failover

Set multiple direct PostgreSQL hosts when the database endpoint does not handle
primary failover:

```bash
export THREAD_BOT_DATABASE_URL=postgres://bot:password@postgres-1:5432/thread_bot
export THREAD_BOT_DATABASE_HOSTS=postgres-1,postgres-2
```

`THREAD_BOT_DATABASE_URL` supplies credentials, database, port, and connection
parameters. Each entry in `THREAD_BOT_DATABASE_HOSTS` replaces its host. The
store checks `transaction_read_only` when acquiring a connection and remembers
the last writable pool. Without `THREAD_BOT_DATABASE_HOSTS`, it uses the URL as
a single target.

`THREAD_BOT_DB_MAX_CONNECTIONS` applies to each target. A failed target may
delay switching by up to `THREAD_BOT_DB_ACQUIRE_TIMEOUT_SECS` before the next
one is tried.

`SUPPORT_QWEN_MODEL` is optional. `SUPPORT_ADMISSION_REQUIRED_TEXTS` may contain
comma-separated phrases; on a thread's first message at least one phrase must
match before a Qwen session is started. `SUPPORT_QWEN_EXECUTABLE` accepts any
executable name or path available in the bot process.

The repository skill at [`.qwen/skills/support/SKILL.md`](../../.qwen/skills/support/SKILL.md)
is the diagnostic entry point. It links to the existing Markdown runbooks in
[`instructions/`](instructions/); runbooks stay load-on-demand and are not
copied into Rust prompts.

Each accepted support thread creates a linked engineer thread. It receives
mirrored user/bot messages and an ACP report with session ID, duration, stop
reason, recovery flag, or a bounded failure detail. Users receive only final
agent text or a generic availability error. `!support debug-report` in the
engineer thread uploads an HTML snapshot of messages and persisted runtime state.
