# Support Bot Tool Authoring Guide

This document is meant to be copied into another coding agent as implementation
context. It describes how to add production-oriented tools for the
`support-bot` example without overloading the LLM context.

## Bot Initialization Shape

The support bot is built from a config, an OpenAI-compatible LLM client, an
instruction repository, and a tool registry managed by `SupportBotBuilder`.

Minimal initialization shape:

```rust
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use mattermost_bot::MattermostBotMetrics;
use support_bot::{
    DEFAULT_SUPPORT_SYSTEM_PROMPT, EngineerNotificationConfig, EngineerNotificationTarget,
    InstructionConfig, InstructionManifest, InstructionRepository, LlmConfig,
    OpenAiChatCompletionsClient, SupportBotBuilder, SupportBotConfig, SupportBotLimits,
    SupportBotMetrics, SupportRouteConfig, ToolConfig,
};
use thread_bot::ThreadBotMetrics;

async fn build_support_handler() -> anyhow::Result<support_bot::SupportBotHandler> {
    let config = SupportBotConfig {
        system_prompt: DEFAULT_SUPPORT_SYSTEM_PROMPT.to_string(),
        llm: LlmConfig {
            base_url: "http://localhost:11434".to_string(),
            api_key: None,
            model: "gpt-4o-mini".to_string(),
            timeout: Duration::from_secs(45),
        },
        instructions: InstructionConfig {
            root_path: PathBuf::from("examples/support_bot/instructions"),
            max_context_instructions: 5,
            max_instruction_bytes: 16 * 1024,
        },
        tools: ToolConfig::default(),
        limits: SupportBotLimits {
            max_tool_rounds: 4,
            max_tool_calls_per_round: 8,
            max_tool_result_bytes: 16 * 1024,
        },
        routes: SupportRouteConfig::default(),
        engineer_notifications: EngineerNotificationConfig {
            target: EngineerNotificationTarget::SameThread,
        },
    };

    let manifest = InstructionManifest { documents: vec![] };
    let instruction_repo = InstructionRepository::new(
        config.instructions.root_path.clone(),
        manifest,
    )
    .with_max_instruction_bytes(config.instructions.max_instruction_bytes)
    .with_max_load_documents(config.instructions.max_context_instructions);

    let llm = Arc::new(OpenAiChatCompletionsClient::new(config.llm.clone())?);

    let bot_name = "support_bot";
    let mut metrics_registry = support_bot::prometheus_client::registry::Registry::default();
    let _mattermost_metrics = MattermostBotMetrics::register(&mut metrics_registry);
    let _thread_metrics = ThreadBotMetrics::register(&mut metrics_registry);
    let support_metrics = SupportBotMetrics::register(&mut metrics_registry);

    let handler = SupportBotBuilder::new(bot_name, config, llm)
        .with_instruction_repository(instruction_repo)?
        .with_metrics(support_metrics.for_bot(bot_name))
        .with_tool(MyTool::new())?
        .build()
        .await?;

    Ok(handler)
}
```

Parts and responsibilities:

- `SupportBotConfig.system_prompt`: the top-level behavior contract for the
  LLM. Put durable rules here: tool-use policy, safety boundaries, tone,
  escalation rules, and domain assumptions. Do not put large runbooks here.
- `InstructionConfig` and `InstructionRepository`: Markdown runbooks exposed
  through the `instructions` tool. Put procedural knowledge here so the model
  can load only the relevant documents.
- `LlmConfig`: OpenAI-compatible base URL, API key, model name, and request
  timeout.
- `ToolConfig.remote_mcp_endpoints`: optional remote MCP tools. Use this for
  capabilities implemented outside this Rust process.
- `SupportBotLimits`: guardrails around LLM/tool loops and tool result size.
  Keep outputs compact enough to fit under `max_tool_result_bytes`.
- `SupportRouteConfig`: which Mattermost channels are user channels and which
  channel receives engineer/debug messages.
- `EngineerNotificationConfig`: whether engineer notifications stay in the same
  thread or go to a separate Mattermost channel.
- `SupportBotBuilder`: registers instruction, workflow, local, and remote MCP
  tools, then returns a `SupportBotHandler`.
- `SupportBotMetrics`: optional Prometheus/OpenMetrics families for support
  routes, LLM calls, tool calls, replies, and close notifications. Create the
  registry outside the bot and pass a per-bot handle with `.with_metrics(...)`.

The default builder registers workflow tools such as sending a user message,
notifying an engineer, and finishing a request. Add domain tools with
`.with_tool(...)` or through a helper that registers several tools.

The bot uses OpenAI-compatible chat completions. Tool calls are persisted in
thread/message metadata for debug reports. Tool results are truncated by
`SupportBotLimits.max_tool_result_bytes`, so tools must return compact JSON.

## Metrics Wiring

Use the re-exported `prometheus-client` crate instead of adding your own direct
dependency:

```rust
let mut registry = support_bot::prometheus_client::registry::Registry::default();
let support_metrics = support_bot::SupportBotMetrics::register(&mut registry);

let handler = SupportBotBuilder::new("support_bot", config, llm)
    .with_metrics(support_metrics.for_bot("support_bot"))
    .build()
    .await?;
```

For the runnable example, all three layers can share one registry:

```rust
let mut registry = support_bot::prometheus_client::registry::Registry::default();
let mattermost_metrics = mattermost_bot::MattermostBotMetrics::register(&mut registry);
let thread_metrics = thread_bot::ThreadBotMetrics::register(&mut registry);
let support_metrics = support_bot::SupportBotMetrics::register(&mut registry);
```

Keep `bot` labels stable and low-cardinality, for example `support_bot`,
`support_bot_eu`, or `support_bot_staging`. Do not use channel IDs, thread IDs,
post IDs, usernames, free-form errors, URLs, or tool arguments as metric labels.
Expose the registry through whatever HTTP stack the application already uses;
the library crates intentionally do not start a `/metrics` server.

## Integration Choices

Prefer one of these two paths:

1. Local Rust tools implementing `SupportTool`.
   Use this when the tool directly belongs to this bot or needs access to the
   current `thread_bot::Thread` through `ToolContext`.

2. Remote MCP tools configured with `SUPPORT_REMOTE_MCP_NAMES`.
   Use this when the operational capability lives behind a separate service.
   Remote tool names are exposed as
   `<endpoint_name>.<remote_tool_name>`.

Do not call external systems directly from the LLM prompt. Expose every
operation as a typed tool with a small schema, validation, timeouts, and bounded
output.

## Local Tool Contract

A local tool implements:

```rust
#[async_trait]
impl SupportTool for MyTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "domain.lookup".to_string(),
            description: "Look up bounded operational context for a support request.".to_string(),
            kind: ToolKind::ReadOnly,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "target": { "type": "string" },
                    "filter": { "type": "string" },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 100 }
                },
                "required": ["target"],
                "additionalProperties": false
            }),
        }
    }

    async fn call(&self, ctx: ToolContext, call: ToolCall) -> Result<ToolExecutionOutcome> {
        let args: LookupArgs = serde_json::from_value(call.arguments)
            .map_err(|error| SupportBotError::Tool(format!("invalid domain.lookup args: {error}")))?;

        // Validate and execute.
        // Return ToolExecutionOutcome::ToolResult(ToolResult { ... }).
    }
}
```

Register local tools in `SupportBotBuilder` setup, either from the example or a
small helper such as `register_support_operations_tools(&mut registry)`.

## Tool Argument Validation

Always validate arguments twice:

1. JSON Schema in `ToolSpec.input_schema`, so the model sees the shape.
2. Rust-side validation in `call`, because model output and MCP callers are not
   trusted.

Use typed structs:

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LookupArgs {
    target: String,
    #[serde(default)]
    filter: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}
```

Validation rules:

- Use `#[serde(deny_unknown_fields)]` for all local tool args.
- Reject empty required strings after `trim()`.
- Clamp or reject numeric ranges. Prefer rejecting invalid input with a clear
  tool error instead of silently changing dangerous values.
- Validate enum-like fields with Rust enums, not free strings.
- Reject unexpected modes, unknown targets, unsupported scopes, and malformed
  IDs.
- Never deserialize directly into loosely typed `serde_json::Value` unless the
  field is intentionally arbitrary JSON.
- Add unit tests for wrong type, missing required field, empty strings, extra
  fields, out-of-range numbers, and a valid happy path.

Workflow tools in `lib/support-bot/src/tools.rs` are a good reference: they use
typed args, `deny_unknown_fields`, and explicit non-empty string checks.

## Tool Result Design

Return compact structured JSON. Do not return raw megabytes of backend output.

Good shape:

```json
{
  "query": {
    "target": "domain-entity",
    "filter": "short-filter",
    "limit": 50
  },
  "summary": {
    "matches": 17,
    "time_range": "2026-05-04T19:30:00Z..2026-05-04T20:00:00Z",
    "notable_findings": 3
  },
  "groups": [
    { "label": "group-a", "count": 6 }
  ],
  "samples": [
    {
      "timestamp": "2026-05-04T19:58:12Z",
      "summary": "representative sanitized finding",
      "reference_id": "abc123"
    }
  ],
  "truncated": false
}
```

Rules:

- Include enough query echo data for auditability.
- Include summaries first, samples second.
- Include `truncated: true` when output was reduced.
- Include stable identifiers when they are safe to expose.
- Avoid secrets, tokens, full headers, cookies, authorization values, or PII.
- Redact aggressively before returning JSON to the LLM.
- Prefer a few representative samples over exhaustive raw data.
- If the backend fails, return a tool error with a short diagnostic. Do not
  fabricate an empty successful result.

## Safety Boundaries

These first tools should be read-only. Use `ToolKind::ReadOnly`.

Do not implement mutating operations such as changing infrastructure,
modifying configuration, deleting data, or acknowledging incidents until there
is a separate approval workflow.

If mutating tools are added later:

- expose them as a separate `ToolKind` or explicit workflow action;
- require a human confirmation step;
- include dry-run output;
- log exactly who requested the action and what identifiers were used.

## Prompt and Runbook Expectations

The default support bot system prompt requires the model to use the
`instructions` tool before giving operational guidance. Add runbooks under
`examples/support_bot/instructions` that teach the model when to call concrete
tools.

Recommended instruction documents:

- one document per capability or workflow;
- small routing documents that tell the model which concrete tool to call;
- procedural triage documents for common user symptoms;
- escalation documents for cases where tools do not provide enough evidence.

Keep runbooks procedural:

1. Gather the user-visible symptom.
2. Load relevant instructions.
3. Call read-only tools with bounded arguments.
4. Summarize evidence.
5. Ask for missing context or notify an engineer when blocked.

## Testing Checklist

For every new local tool:

- `spec()` contains a strict schema with `additionalProperties: false`.
- duplicate registration fails or tool names are namespaced clearly.
- valid arguments produce expected compact JSON.
- missing required fields return `SupportBotError::Tool`.
- empty required strings are rejected.
- wrong types are rejected.
- extra fields are rejected.
- min/max limits are enforced.
- backend errors become tool errors and do not panic.
- returned samples are redacted.
- large backend output is summarized and marked `truncated`.

For support-bot integration:

- the LLM receives the tool spec through `ToolRegistry::specs()`;
- tool results are persisted in the trace metadata;
- debug HTML shows the tool call and result status;
- tool-loop limits still stop safely;
- tests avoid real external services by using fakes or wiremock.

## Observability Checklist

Log structured events around every backend call:

- tool name;
- sanitized arguments;
- duration;
- result count;
- truncation flag;
- backend error class;
- support thread id when available from `ToolContext`.

Do not log raw secrets or full backend payloads containing credentials.

## Implementation Plan for a Coding Agent

1. Add a small module for operational tools, for example
   `lib/support-bot/src/ops_tools.rs`, or implement a separate MCP server.
2. Define typed arg structs with `serde(deny_unknown_fields)`.
3. Define backend traits so tests can use fake clients.
4. Implement `SupportTool` for each tool.
5. Add registration helper and call it from `SupportBotBuilder` or the example.
6. Add instruction documents and `manifest.yaml` entries.
7. Add tests for schemas, validation, errors, redaction, truncation, and happy
   paths.
8. Run `cargo fmt`, `cargo test -p support-bot`, and any crate-specific tests.

## Do Not Do These

- Do not give the LLM unbounded shell, SQL, infrastructure, or backend query
  access.
- Do not let tool args accept arbitrary targets without an allowlist.
- Do not return complete raw backend output by default.
- Do not hide backend failures as empty results.
- Do not rely only on JSON Schema for validation.
- Do not include secrets in tool output, traces, debug reports, or application
  logs.
- Do not add write/remediation tools before the read-only tools are reliable.
