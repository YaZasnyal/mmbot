# support-bot example

This example runs the RFC 0008 ACP-only support bot. To compare behavior at
work, run the legacy build from `master` separately and this branch as the Qwen
build; there is no runtime switch inside this binary.

## Configuration

```bash
export MM_BASE_PATH=http://localhost:8065
export MM_BEARER_TOKEN=...
export SUPPORT_USER_CHANNEL_IDS=user-channel-id
export SUPPORT_ENGINEER_CHANNEL_ID=engineer-channel-id
export THREAD_BOT_DATABASE_URL=postgres://test:test@localhost:5433/thread_bot_test

export SUPPORT_QWEN_EXECUTABLE=qwen
export SUPPORT_QWEN_CWD=.
export SUPPORT_QWEN_MODEL=qwen/qwen3.6-35b-a3b
export SUPPORT_QWEN_TIMEOUT_SECS=120

cargo run -p support-bot-example
```

`SUPPORT_QWEN_MODEL` is optional. `SUPPORT_ADMISSION_REQUIRED_TEXTS` may contain
comma-separated phrases; on a thread's first message at least one phrase must
match before a Qwen session is started.

The repository skill at `.qwen/skills/support/SKILL.md` is the diagnostic entry
point. It links to the existing Markdown runbooks in `instructions/`; runbooks
stay load-on-demand and are not copied into Rust prompts.

Each accepted support thread creates a linked engineer thread. It receives
mirrored user/bot messages and an ACP report with session ID, duration, stop
reason, recovery flag, or a bounded failure detail. Users receive only final
agent text or a generic availability error. `!support debug-report` in the
engineer thread uploads an HTML snapshot of messages and persisted runtime state.
