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
  waiting for user input, and finishing a request.

Mattermost-specific engineer notification delivery and remote MCP tool loading
are intentionally left for later reviewable steps.

## Instruction Format

Instruction and runbook trees are documented in
[`docs/instructions.md`](docs/instructions.md). The manifest is the source of
truth; generated indexes such as embeddings may be added later, but should be
derived from the manifest rather than maintained by hand.
