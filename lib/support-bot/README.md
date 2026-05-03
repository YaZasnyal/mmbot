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

This initial crate provides contracts and lightweight utilities. The bounded
LLM/tool execution loop and concrete `SupportBotHandler` are added in later
steps so their API can be reviewed independently.

## Instruction Format

Instruction and runbook trees are documented in
[`docs/instructions.md`](docs/instructions.md). The manifest is the source of
truth; generated indexes such as embeddings may be added later, but should be
derived from the manifest rather than maintained by hand.
