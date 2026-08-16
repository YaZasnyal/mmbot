# RFC 0008 implementation status

- Branch: `feat/rfc-0008-qwen-acp-runtime`
- ACP spike: complete
- ACP-only Layer 4 replacement: complete
- Legacy Layer 4 runtime deletion: complete
- Repository validation: code gates complete; Mattermost integration credentials blocked
- Workplace comparison (`master` versus this branch): pending human deployment

Evidence from 2026-08-15:

- Qwen Code `0.21.12`; local model `qwen/qwen3.6-35b-a3b` answered `QWEN_LOCAL_OK`.
- Official `agent-client-protocol` `2.0.0`, stable ACP v1 API.
- Live Rust/Qwen smoke passed three prompts: same-session continuation and
  `session/load` after process restart (158.21 seconds).
- The support skill lives at `.qwen/skills/support/SKILL.md` and links to the
  existing runbooks instead of duplicating them.
- ACP permissions are denied by default; Mattermost receives agent-message text only.
- Linked engineer reports contain source/session IDs, duration, stop reason,
  recovery flag, final response, or a bounded failure detail.
- `just check` passed (format, clippy with warnings denied, workspace build).
- `cargo test --workspace --lib --bins` passed: 31 support-bot tests plus all
  other workspace unit/bin tests; one explicit live smoke is ignored by default.
- The explicit live smoke then passed all three Qwen turns after a 9.62-second run.

Known gate condition:

- **Claim**: the full Mattermost integration gate needs credentials for the
  already-running local server.
- **Evidence**: `just test-all` reached `http://localhost:8065` on 2026-08-15,
  but both existing `mattermost_bot` integration tests received `403 Forbidden`
  because open signup is disabled and `MM_ADMIN_USER` / `MM_ADMIN_PASS` were not
  supplied.
- **Unblock**: provide credentials for that server or run `just test-full` with
  the repository-managed test environment.
- **Owner**: local/test infrastructure.

Deliberately deferred:

- Atomic outbound reply plus metadata persistence across a process crash.
- Active-request cancellation and automatic bounded Qwen process restart.
