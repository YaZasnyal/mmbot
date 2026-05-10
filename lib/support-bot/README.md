# support-bot

Experimental Layer 4 crate for building Mattermost support bots on top of
`thread-bot`.

The crate owns support-specific concerns:

- OpenAI-compatible LLM integration through a `LlmClient` abstraction.
- Local Rust tools and remote MCP tools behind one `ToolRegistry`.
- Compact per-thread resume state stored in `thread-bot` thread metadata.
- Markdown instruction/runbook loading from a documented filesystem tree.
- Separate user and engineer notification sinks, including engineer-channel
  mirroring, status updates, and debug exports.

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

This crate now includes a concrete `SupportBotHandler`:

- routes user-channel threads into the LLM/tool loop;
- routes engineer-channel threads through debug command handling before LLM;
- persists compact `SupportThreadState` under the `support_bot` thread metadata
  key, including `active`/`finished`/`stopped` request state and optional finish
  summary;
- executes local tools through `ToolRegistry` with bounded tool rounds.
- provides default workflow tools for user replies, engineer notifications,
  and finishing a request.
- mirrors user and bot messages into a dedicated engineer channel thread when
  configured.
- posts support status updates to the engineer thread when `finish_request`
  marks a request as `finished`.
- supports on-demand engineer diagnostics with `!support debug-report`, which
  uploads an HTML report containing support state, source posts, and captured
  tool traces.
- notifies the engineer thread on tool-loop limit failures and tells engineers
  to run `!support debug-report` for the full HTML snapshot.
- can register remote MCP tools from `ToolConfig.remote_mcp_endpoints` via
  `register_remote_mcp_tools`.
- includes `SupportBotBuilder` with a default system prompt that enforces using
  the `instructions` tool before inventing diagnostics.

## Runtime Flow

For a user-channel thread, the handler loads `SupportThreadState`, ensures the
engineer thread exists when a separate engineer channel is configured, mirrors
new user messages, builds LLM context from the source thread plus loaded
instruction state, and executes bounded tool rounds.

Default workflow tools return structured actions:

- `send_user_message`: replies to the user through `ThreadEffect::Reply` and
  mirrors the bot response to the engineer thread when configured.
- `notify_engineer`: sends diagnostic context or escalation notes to the
  engineer sink.
- `finish_request`: stores `status = "finished"` and `finished_summary`,
  posts a status update to the engineer thread, and resolves the underlying
  `thread-bot` thread.

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
[`docs/instructions.md`](docs/instructions.md). Documents are ordinary `.md`
files with YAML frontmatter. The default entry point is `/index`, and generated
indexes such as embeddings may be added later as derived artifacts rather than
maintained by hand.
