# support-bot

Layer 4 support workflow for Mattermost. It maps each support thread to a Qwen
Code ACP session and keeps operational diagnostics in a linked engineer thread.

The crate intentionally does not implement an LLM loop, tool registry, MCP
client, or prompt heuristics. Qwen owns agent behavior and loads the repository
support skill; Rust owns routing, session metadata, final replies, admission,
control reactions, and engineer reports.

```rust,no_run
use std::sync::Arc;
use support_bot::{QwenAcpConfig, QwenAcpRuntime, SupportBotBuilder, SupportBotConfig};

# async fn example(config: SupportBotConfig) -> anyhow::Result<()> {
let runtime = QwenAcpRuntime::start(QwenAcpConfig {
    executable: "qwen".into(),
    cwd: ".".into(),
    model: None,
    timeout: std::time::Duration::from_secs(120),
}).await?;
let handler = SupportBotBuilder::new("support", config, Arc::new(runtime)).build();
# Ok(())
# }
```

Qwen receives `/support`, the Mattermost thread/post IDs, and the new user
message. It returns one structured object:

```json
{"message":"User-facing Markdown","action":"none|ignore|finish","reason":null}
```

Only `message` is posted to the user. `ignore` marks a non-support thread as
ignored; `finish` marks a completed conversation as finished. Unknown actions
are treated as `none`.

The support skill must not use Qwen's `ask_user_question` popup: Mattermost
cannot display or answer that ACP permission UI. To request clarification, Qwen
returns the question in `message` with `action: "none"`; the user's next
Mattermost reply continues the same ACP session.

Session ID, last input post, recovery status, duration, stop reason, and failures are available
in thread metadata and the linked engineer thread. Engineers can run
`!support debug-report` to export the tracked thread and runtime state as HTML.
