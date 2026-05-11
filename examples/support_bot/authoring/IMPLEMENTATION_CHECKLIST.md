# Implementation Checklist

Use this as the final acceptance checklist for a coding LLM.

## Runtime

- `examples/support_bot/Cargo.toml` includes any needed HTTP dependencies.
- Startup initializes tracing before other work.
- Config is loaded once from environment variables.
- Required env vars produce clear startup errors.
- `InstructionRepository::lint` runs at startup.
- `OpenAiChatCompletionsClient` is built from `LlmConfig`.
- `SupportBotBuilder` receives the instruction repository, local tools,
  remote MCP config, and support metrics handle.
- `PgThreadStore` is built from the configured PostgreSQL pool.
- `ThreadBotPlugin` receives `thread_metrics.for_bot(bot_name, bot_name)`.
- `Bot` receives `mattermost_metrics.for_bot(bot_name)`.
- Mattermost runtime and axum runtime share graceful shutdown.

## Metrics And HTTP

- There is exactly one shared `prometheus_client::registry::Registry`.
- `MattermostBotMetrics::register`, `ThreadBotMetrics::register`, and
  `SupportBotMetrics::register` all use that registry.
- `/metrics` encodes the shared registry.
- `/livez` returns 204 without dependency checks.
- `/readyz` returns 204 when PostgreSQL, Mattermost, instructions, and remote
  MCP configuration are ready.
- `/readyz` returns 503 with sanitized JSON when any required check fails.
- Metric labels stay low-cardinality.
- No auth headers, tokens, database passwords, or PII appear in readiness,
  metrics, or logs.

## Tools

- Tool names are stable and namespaced.
- Tool specs use strict JSON Schema with `additionalProperties: false`.
- Tool args use typed serde structs with `deny_unknown_fields`.
- Rust validation rejects empty strings, unsupported enum values, bad IDs, and
  out-of-range numeric fields.
- Tools are read-only unless a human approval workflow exists.
- Tool outputs are compact JSON with summary-first shape.
- Tool outputs include `truncated` when reduced.
- Tool outputs redact secrets and PII.
- Backend errors become `SupportBotError::Tool` with sanitized messages.
- Tool tests cover schema, validation, happy path, backend error, redaction,
  and truncation.

## Knowledge Base

- `/index` exists and routes to major support areas.
- Every Markdown file has YAML frontmatter with `title`.
- Every internal link uses `[/document/id]` format.
- Tool runbooks explain when to call tools, required args, safe bounds, and how
  to interpret output.
- Escalation runbooks explain when to use `notify_engineer`.
- Lint returns no issues.

## Verification Commands

Run from the repository root:

```bash
cargo fmt
cargo check -p support-bot-example
cargo test -p support-bot
```

If HTTP routes are implemented, also run the example with dependencies up and
check:

```bash
curl -i http://localhost:8080/livez
curl -i http://localhost:8080/readyz
curl -s http://localhost:8080/metrics | rg 'mattermost_bot_|thread_bot_|support_bot_'
```

## Do Not Do These

- Do not run git staging or commit commands unless the user asks.
- Do not give the LLM unbounded shell, SQL, or infrastructure access.
- Do not put large runbooks in the system prompt.
- Do not create separate metrics registries per layer.
- Do not make readiness leak secrets.
- Do not silently hide backend failures as empty successful tool results.
- Do not add write/remediation tools before read-only diagnostics are reliable.
