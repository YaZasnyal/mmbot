# Initialization and Runtime

This document describes how a coding LLM should wire the runnable
`support-bot` example into a production-shaped process.

## Runtime Responsibilities

The final process must initialize these layers in this order:

1. Tracing/logging.
2. Environment-backed application config.
3. Shared Prometheus registry and all metric families.
4. Instruction repository with startup linting.
5. OpenAI-compatible LLM client.
6. Local tool clients and tool registrations.
7. Support handler.
8. PostgreSQL thread store.
9. Thread-bot plugin with metrics.
10. Mattermost bot with metrics.
11. Axum HTTP server for probes and metrics.
12. Graceful shutdown shared by Mattermost runtime and HTTP runtime.

Do not start a separate metric registry per layer. There must be one registry
per process.

## Required Dependencies

If axum is not already present in `examples/support_bot/Cargo.toml`, add:

```toml
axum = "0.7"
http = "1"
```

If the implementation needs shared mutable readiness state, use the standard
library where possible (`Arc`, atomics, locks). Prefer existing dependencies
already present in the workspace before adding new ones.

## Configuration

Keep config loading in small functions. Avoid reading environment variables
inside business logic after startup.

Required Mattermost config:

- `MM_BASE_PATH`, default `http://localhost:8065`.
- `MM_BEARER_TOKEN`, required.

Required thread store config:

- `THREAD_BOT_DATABASE_URL`, default
  `postgres://test:test@localhost:5433/thread_bot_test`.
- `THREAD_BOT_DB_MAX_CONNECTIONS`, default `5`.

Required support config:

- `SUPPORT_USER_CHANNEL_IDS`, comma-separated. Empty means no explicit user
  channel allowlist.
- `SUPPORT_ENGINEER_CHANNEL_ID`, required for engineer mirroring and
  notifications.
- `SUPPORT_INSTRUCTIONS_ROOT`, default
  `examples/support_bot/instructions`.
- `SUPPORT_LLM_BASE_URL`, default `http://localhost:11434`.
- `SUPPORT_LLM_MODEL`, default `gpt-4o-mini`.
- `SUPPORT_LLM_API_KEY`, optional.
- `SUPPORT_LLM_TIMEOUT_SECS`, default `45`.
- `SUPPORT_MAX_TOOL_ROUNDS`, default `4`.
- `SUPPORT_MAX_TOOL_CALLS_PER_ROUND`, default `8`.
- `SUPPORT_MAX_TOOL_RESULT_BYTES`, default `16384`.
- `SUPPORT_MAX_CONTEXT_INSTRUCTIONS`, default `5`.
- `SUPPORT_MAX_INSTRUCTION_BYTES`, default `16384`.
- `SUPPORT_SYSTEM_PROMPT_FILE`, optional; wins over inline prompt.
- `SUPPORT_SYSTEM_PROMPT`, optional; falls back to
  `DEFAULT_SUPPORT_SYSTEM_PROMPT`.

Remote MCP config:

- `SUPPORT_REMOTE_MCP_NAMES`, comma-separated endpoint names.
- For each name `x`, read:
  - `SUPPORT_REMOTE_MCP_X_URL`, required.
  - `SUPPORT_REMOTE_MCP_X_AUTH_HEADER`, optional.
  - `SUPPORT_REMOTE_MCP_X_TIMEOUT_SECS`, default `30`.

HTTP config:

- `SUPPORT_HTTP_ADDR`, default `0.0.0.0:8080`.

Use uppercase env keys for endpoint names by uppercasing and replacing `-` with
`_`.

## Metrics Wiring

Use the `support_bot::prometheus_client` re-export for the registry and
encoding so the example matches the library version.

The initialization must look like this shape:

```rust
use mattermost_bot::MattermostBotMetrics;
use support_bot::SupportBotMetrics;
use thread_bot::ThreadBotMetrics;

let bot_name = "support_bot";
let registry = Arc::new(std::sync::Mutex::new(
    support_bot::prometheus_client::registry::Registry::default(),
));

let (mattermost_metrics, thread_metrics, support_metrics) = {
    let mut registry = registry.lock().expect("metrics registry mutex poisoned");
    (
        MattermostBotMetrics::register(&mut registry),
        ThreadBotMetrics::register(&mut registry),
        SupportBotMetrics::register(&mut registry),
    )
};
```

Then pass metric handles to every layer:

```rust
let handler = SupportBotBuilder::new(bot_name, config.clone(), llm)
    .with_instruction_repository(instruction_repo)?
    .with_metrics(support_metrics.for_bot(bot_name))
    .build()
    .await?;

let plugin = ThreadBotPlugin::new(handler, store)
    .with_metrics(thread_metrics.for_bot(bot_name, bot_name));

let bot = Bot::with_config(mm_config)?
    .with_metrics(mattermost_metrics.for_bot(bot_name))
    .with_plugin(plugin);
```

Metric label rules:

- Keep `bot` stable and low-cardinality: `support_bot`,
  `support_bot_staging`, or a small deployment name.
- Tool names are allowed because tool registrations are bounded.
- Do not use channel IDs, thread IDs, post IDs, user names, raw errors, URLs,
  query strings, or tool arguments as labels.

## Axum HTTP Server

The example must run an axum server next to the Mattermost bot runtime.

Routes:

- `GET /livez`: returns `204 No Content` when the process is alive.
- `GET /readyz`: returns `204 No Content` when dependencies are usable,
  otherwise `503 Service Unavailable` with a compact JSON body.
- `GET /metrics`: returns OpenMetrics/Prometheus text from the shared registry.

Use this shape:

```rust
use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use support_bot::prometheus_client::{
    encoding::text::encode,
    registry::Registry,
};

#[derive(Clone)]
struct HttpState {
    registry: Arc<Mutex<Registry>>,
    readiness: Arc<Readiness>,
}

fn observability_router(state: HttpState) -> Router {
    Router::new()
        .route("/livez", get(livez))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        .with_state(state)
}

async fn livez() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn metrics(State(state): State<HttpState>) -> Response {
    let mut body = String::new();
    let encoded = {
        let registry = state.registry.lock().expect("metrics registry mutex poisoned");
        encode(&mut body, &registry)
    };

    match encoded {
        Ok(()) => (
            [(header::CONTENT_TYPE, HeaderValue::from_static("text/plain; version=0.0.4"))],
            body,
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to encode metrics: {error}"),
        )
            .into_response(),
    }
}
```

Do not hold the registry lock while doing network or database work. Encoding
metrics while holding the lock is acceptable because it is short and local.

## Readiness

Create a small `Readiness` object that owns cheap dependency handles:

- PostgreSQL pool or `PgThreadStore` backing pool if exposed.
- Mattermost API `Configuration`.
- Instruction root path.
- Remote MCP endpoint configs.

`/readyz` should perform bounded checks:

- PostgreSQL: run `SELECT 1` against the pool with a short timeout.
- Mattermost: call a cheap authenticated endpoint such as current user or a
  server/info endpoint with a short timeout. If the generated client does not
  expose a convenient method, implement the check with the same configured base
  URL and bearer token using an existing HTTP client already used by the
  project.
- Instructions: call `InstructionRepository::lint(root)` and fail readiness
  if lint issues are present. This catches bad runbook deploys.
- Remote MCP: at minimum validate that configured URLs parse and required
  env-derived fields are present. If the MCP client exposes a cheap list-tools
  or health call, use it with each endpoint timeout.

Readiness response on failure:

```json
{
  "status": "not_ready",
  "checks": [
    { "name": "postgres", "ok": false, "error": "connection refused" },
    { "name": "instructions", "ok": true }
  ]
}
```

Keep errors compact and sanitized. Do not include bearer tokens, auth headers,
database passwords, or full URLs with credentials.

## Graceful Shutdown

Run Mattermost and axum under the same shutdown signal.

Expected shape:

```rust
let shutdown = mattermost_bot::tokio_graceful::Shutdown::builder()
    .with_signal(tokio::signal::ctrl_c())
    .build();

let http_server = async {
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    axum::serve(listener, observability_router(http_state))
        .with_graceful_shutdown(shutdown.guard().cancelled())
        .await?;
    anyhow::Ok(())
};

let bot_runner = async {
    bot.run(shutdown.guard()).await;
    anyhow::Ok(())
};

tokio::try_join!(http_server, bot_runner)?;
```

Adjust the exact shutdown API to the version available in this workspace. The
important requirement is that both runtimes stop on the same signal and neither
task is left detached.

## Startup Validation

At startup:

- Parse config once.
- Validate required env values.
- Lint instructions and log every issue with path, id, kind, and message.
- Decide whether lint issues are fatal. For production, prefer fatal startup
  failure. For the example, continuing is acceptable only if explicitly
  documented.
- Register all local tools before `build().await`.
- Start the HTTP server only after config and metrics are initialized.

## Done Criteria

- `cargo fmt` passes.
- `cargo check -p support-bot-example` passes.
- Hitting `/livez` returns 204.
- Hitting `/readyz` returns 204 with dependencies up and 503 with a sanitized
  JSON body when a required dependency is unavailable.
- Hitting `/metrics` includes `mattermost_bot_`, `thread_bot_`, and
  `support_bot_` metric families after registration.
