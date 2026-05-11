# Tool Authoring

This document describes how to add production-oriented tools to
`support-bot`.

## Integration Choices

Prefer one of two paths:

1. Local Rust tools implementing `support_bot::SupportTool`.
2. Remote MCP tools configured through `SUPPORT_REMOTE_MCP_NAMES`.

Use local Rust tools when the capability belongs to this process, needs typed
Rust clients, or needs `ToolContext.thread`. Use remote MCP when the capability
already exists behind another service.

Every operational capability must be represented as a typed tool. Do not put
raw commands, SQL, URLs, or backend credentials in the prompt and ask the LLM to
use them directly.

## Naming

Use namespaced, stable names:

- `logs.search`
- `metrics.query`
- `deployments.lookup`
- `incidents.find_related`

Workflow tools already exist and are registered by default:

- `send_user_message`
- `notify_engineer`
- `finish_request`

Do not shadow workflow tool names. Duplicate registration must fail.

## Local Tool Contract

A local tool implements `SupportTool`:

```rust
use support_bot::{
    async_trait, SupportBotError, SupportTool, ToolCall, ToolContext,
    ToolExecutionOutcome, ToolKind, ToolResult, ToolSpec,
};

struct LogsSearchTool<C> {
    client: C,
}

#[async_trait]
impl<C> SupportTool for LogsSearchTool<C>
where
    C: LogsClient + Send + Sync,
{
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "logs.search".to_string(),
            description: "Search sanitized service logs for a bounded time range.".to_string(),
            kind: ToolKind::ReadOnly,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "service": { "type": "string" },
                    "query": { "type": "string" },
                    "since_minutes": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 240
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 50
                    }
                },
                "required": ["service", "query"],
                "additionalProperties": false
            }),
        }
    }

    async fn call(&self, ctx: ToolContext, call: ToolCall) -> support_bot::Result<ToolExecutionOutcome> {
        let args: LogsSearchArgs = serde_json::from_value(call.arguments)
            .map_err(|error| SupportBotError::Tool(format!("invalid logs.search args: {error}")))?;
        let args = args.validate()?;

        let thread_id = ctx
            .thread
            .as_ref()
            .map(|thread| thread.info.thread_id.as_str())
            .unwrap_or("-");

        tracing::info!(
            tool = "logs.search",
            service = %args.service,
            since_minutes = args.since_minutes,
            limit = args.limit,
            thread_id = %thread_id,
            "support-bot: calling logs tool"
        );

        let result = self.client.search(args).await
            .map_err(|error| SupportBotError::Tool(format!("logs backend failed: {}", error.public_message())))?;

        Ok(ToolExecutionOutcome::ToolResult(ToolResult {
            call_id: call.id,
            content: serde_json::to_value(result)
                .map_err(|error| SupportBotError::Tool(format!("failed to encode logs.search result: {error}")))?,
            is_error: false,
        }))
    }
}
```

Use `ToolExecutionOutcome::ToolResult` for normal domain tools. Only workflow
tools should return `ToolExecutionOutcome::Action`.

## Argument Validation

Always validate arguments twice:

1. JSON Schema in `ToolSpec.input_schema`, so the model sees the shape.
2. Rust-side validation in `call`, because model output and MCP callers are not
   trusted.

Use typed structs:

```rust
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LogsSearchArgs {
    service: String,
    query: String,
    #[serde(default = "default_since_minutes")]
    since_minutes: u32,
    #[serde(default = "default_limit")]
    limit: u32,
}

impl LogsSearchArgs {
    fn validate(mut self) -> support_bot::Result<Self> {
        self.service = require_non_empty(self.service, "service")?;
        self.query = require_non_empty(self.query, "query")?;

        if !allowed_services().contains(&self.service.as_str()) {
            return Err(SupportBotError::Tool(format!(
                "unsupported service: {}",
                self.service
            )));
        }
        if !(1..=240).contains(&self.since_minutes) {
            return Err(SupportBotError::Tool(
                "since_minutes must be between 1 and 240".to_string(),
            ));
        }
        if !(1..=50).contains(&self.limit) {
            return Err(SupportBotError::Tool(
                "limit must be between 1 and 50".to_string(),
            ));
        }

        Ok(self)
    }
}
```

Validation rules:

- Use `#[serde(deny_unknown_fields)]` for all local tool args.
- Reject empty required strings after `trim()`.
- Prefer explicit allowlists for services, environments, clusters, indexes, and
  namespaces.
- Reject unsupported enum values using Rust enums, not free strings.
- Reject or error on out-of-range numbers. Do not silently expand scope.
- Never deserialize the whole args object into `serde_json::Value` unless the
  field is intentionally arbitrary JSON.
- Treat all tool args as untrusted input.

## Output Design

Return compact structured JSON. Do not return raw backend output.

Recommended result shape:

```json
{
  "query": {
    "service": "api",
    "since_minutes": 30,
    "limit": 20
  },
  "summary": {
    "matches": 17,
    "time_range": "2026-05-04T19:30:00Z..2026-05-04T20:00:00Z",
    "notable_findings": 3
  },
  "groups": [
    { "label": "timeout", "count": 6 }
  ],
  "samples": [
    {
      "timestamp": "2026-05-04T19:58:12Z",
      "summary": "sanitized representative finding",
      "reference_id": "abc123"
    }
  ],
  "truncated": false
}
```

Rules:

- Echo sanitized query inputs for auditability.
- Put summary fields before samples.
- Include `truncated: true` when output was reduced.
- Include stable reference IDs only when they are safe to show to the LLM.
- Redact secrets, tokens, cookies, authorization headers, passwords, and PII.
- Prefer representative samples over exhaustive rows.
- Keep output below `SupportBotLimits.max_tool_result_bytes`.
- If the backend fails, return a tool error. Do not fabricate empty success.

## Backend Clients

Do not couple tools directly to concrete network clients if tests need fakes.
Define small traits per backend:

```rust
#[async_trait]
trait LogsClient {
    async fn search(&self, args: LogsSearchArgs) -> Result<LogsSearchResult, LogsBackendError>;
}
```

Production clients should enforce:

- request timeout;
- bounded result limit;
- sanitized error messages;
- no secret-bearing debug formatting;
- structured tracing around calls.

## Observability Inside Tools

The support handler already records generic tool call metrics by tool name and
outcome. Tool implementations should add structured tracing for backend calls:

- tool name;
- sanitized arguments;
- duration;
- result count;
- truncation flag;
- backend error class;
- support thread id when present in `ToolContext`.

Do not add high-cardinality custom Prometheus labels from tool arguments unless
there is a fixed allowlist.

## Safety Boundaries

Initial operational tools must be read-only:

```rust
kind: ToolKind::ReadOnly
```

Do not implement mutating operations such as changing infrastructure, deleting
data, restarting services, changing feature flags, acknowledging incidents, or
editing customer records until the bot has a separate human approval workflow.

If mutating tools are added later, they must include:

- explicit human confirmation;
- dry-run output;
- audit logging of requester and identifiers;
- narrow allowlists;
- clear rollback or remediation notes;
- separate tests for denied and approved paths.

## Remote MCP Tools

Remote MCP endpoints are configured through `ToolConfig.remote_mcp_endpoints`.
The builder registers them when `.with_remote_mcp_tools(true)` is left enabled.

Remote tools are exposed as:

```text
<endpoint_name>.<remote_tool_name>
```

For example, endpoint `logs` with remote tool `search` becomes `logs.search`.

Remote MCP tools must follow the same principles as local tools:

- strict schema;
- bounded output;
- redaction;
- timeouts;
- sanitized errors;
- read-only by default.

## Testing Checklist

For every new local tool, add tests under `lib/support-bot/src/tests/...` or
the owning crate's existing test layout.

Required tests:

- `spec()` returns the expected name, kind, schema, and
  `additionalProperties: false`.
- duplicate registration fails when names collide.
- valid args produce expected compact JSON.
- missing required fields return `SupportBotError::Tool`.
- empty required strings are rejected.
- wrong types are rejected.
- extra fields are rejected.
- enum and allowlist validation works.
- numeric min/max limits are enforced.
- backend errors become tool errors and do not panic.
- output samples are redacted.
- large backend output is summarized and marked `truncated`.
- `ToolContext::without_thread()` is handled if the tool does not require a
  thread.

Also add integration coverage where useful:

- tool specs are included in `ToolRegistry::specs()`;
- tool results appear in the persisted debug trace;
- tool-loop limits still stop safely;
- tests use fake clients or wiremock-style servers, not real production
  services.

## Implementation Steps

1. Create a module for the capability, for example
   `lib/support-bot/src/ops_tools/logs.rs` or a separate crate if ownership is
   broader.
2. Define typed args and result structs.
3. Define a small backend trait and fake implementation for tests.
4. Implement `SupportTool`.
5. Add a registration helper, for example `register_ops_tools(builder, clients)`.
6. Register tools before `SupportBotBuilder::build().await`.
7. Add knowledge-base runbooks that teach the LLM when and how to call the new
   tools.
8. Run `cargo fmt` and focused tests.
