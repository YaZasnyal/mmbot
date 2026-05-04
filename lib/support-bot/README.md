# support-bot

Experimental Layer 4 skeleton for building Mattermost support bots on top of
`thread-bot`.

The crate owns support-specific concerns:

- OpenAI-compatible LLM integration through a `LlmClient` abstraction.
- Local Rust tools and remote MCP tools behind one `ToolRegistry`.
- Compact per-thread resume state stored in `thread-bot` thread metadata.
- Instruction/runbook selection from a documented filesystem manifest.
- Separate user and engineer notification sinks.

The API is intentionally experimental while the first real support bot is being
built. Prefer small integrations and keep concrete tools/prompts outside this
crate.

## Layering

```text
support-bot  -> implements thread_bot::ThreadHandler
thread-bot   -> owns Mattermost thread tracking and persistence
mattermost-bot / mattermost-api
```

`support-bot` should not add LLM behavior to `thread-bot`; it consumes
`Thread`, `ThreadEffect`, and thread/message metadata exposed by Layer 3.

## Current Scope

This crate now includes the first concrete `SupportBotHandler` skeleton:

- routes user-channel threads into the LLM/tool loop;
- routes engineer-channel threads through debug command handling before LLM;
- persists `SupportThreadState` under the `support_bot` thread metadata key;
- executes local tools through `ToolRegistry` with bounded tool rounds.
- provides default workflow tools for user replies, engineer notifications,
  and finishing a request.
- mirrors user and bot messages into a dedicated engineer channel thread when
  configured.
- supports on-demand engineer diagnostics with `!support debug-report`, which
  uploads an HTML report containing support state, source posts, and captured
  tool traces.
- can register remote MCP tools from `ToolConfig.remote_mcp_endpoints` via
  `register_remote_mcp_tools`.
- includes `SupportBotBuilder` with a default system prompt that enforces using
  the `instructions` tool before inventing diagnostics.

## Engineer Debug Commands

Engineer-channel commands are parsed before the engineer thread enters the
regular LLM flow. The default prefixes are `/support` and `!support`.

- `!support debug-report`: export the source support thread diagnostics as an
  HTML attachment in the engineer thread.
- `!support state <thread-id>`: pending command shape for state inspection.
- `!support trace <thread-id>`: pending command shape for trace inspection.
- `!support retry <thread-id>`: pending command shape for retrying a support
  run after manual review.

## Instruction Format

Instruction and runbook trees are documented in
[`docs/instructions.md`](docs/instructions.md). The manifest is the source of
truth; generated indexes such as embeddings may be added later, but should be
derived from the manifest rather than maintained by hand.
