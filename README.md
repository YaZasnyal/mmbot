# mmbot

A Rust framework for building Mattermost bots. Layered architecture — from a low-level API client to a ready-made thread-aware bot runtime.

## Architecture

```
┌──────────────────────────────────────────┐
│  Layer 3: thread-bot                     │  Thread-aware bots with debounce,
│  ThreadHandler, per-thread actors        │  persistence and reconciliation
└────────────────┬─────────────────────────┘
                 │
┌────────────────┴─────────────────────────┐
│  Layer 2: mattermost-bot                 │  Core framework: WebSocket,
│  Bot, Plugin, middleware, cron           │  plugins, graceful shutdown
└────────────────┬─────────────────────────┘
                 │
┌────────────────┴─────────────────────────┐
│  Layer 1: mattermost-api                 │  Typed HTTP/WS client,
│  Auto-generated from OpenAPI             │  generated from OpenAPI spec
└──────────────────────────────────────────┘
```

**Layer 1 — `mattermost-api`** — auto-generated Mattermost API v4 client. Typed models, HTTP requests via `reqwest`. Serves as the foundation for all layers above.

**Layer 2 — `mattermost-bot`** — plugin-based framework. Connects to Mattermost over WebSocket, auto-reconnects, dispatches events to plugins in parallel. Supports middleware (e.g. `IgnoreSelf`), cron jobs, and lifecycle hooks (`on_start`/`on_shutdown`).

**Layer 3 — `thread-bot`** — higher-level runtime for thread-based bots. Each thread gets its own actor (tokio task) with guaranteed sequential message processing. Built-in debounce, PostgreSQL persistence, post-reconnect reconciliation, and control via reactions.

## Quick start

### Layer 2: simple plugin

```rust
use std::sync::Arc;
use mattermost_bot::{async_trait, Bot, Plugin, Event, Configuration, tokio_graceful};
use mattermost_bot::types::EventType;

struct EchoPlugin;

#[async_trait]
impl Plugin for EchoPlugin {
    fn id(&self) -> &'static str { "echo" }

    fn filter(&self, event: &Arc<Event>) -> bool {
        matches!(event.data, EventType::Posted(_))
    }

    async fn process_event(&self, event: &Arc<Event>, config: &Arc<Configuration>) {
        if let EventType::Posted(posted) = &event.data {
            let post = &posted.post.inner;
            println!("{}: {}", posted.sender_name, post.message.as_deref().unwrap_or(""));
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Configuration {
        base_path: "http://localhost:8065".to_string(),
        bearer_access_token: Some("your_token".to_string()),
        ..Default::default()
    };

    let mut bot = Bot::with_config(config)?
        .with_plugin(EchoPlugin);

    let shutdown = tokio_graceful::Shutdown::builder()
        .with_signal(tokio::signal::ctrl_c())
        .build();

    bot.run(shutdown.guard()).await;
    Ok(())
}
```

### Layer 3: thread bot

```rust
use std::sync::Arc;
use async_trait::async_trait;
use thread_bot::{ThreadHandler, ThreadEffect, ThreadBotError, Thread, ThreadContext, ThreadBotPlugin, PgThreadStore, ThreadStore};
use mattermost_bot::Bot;

struct MyHandler;

#[async_trait]
impl ThreadHandler for MyHandler {
    fn id(&self) -> &'static str { "my_handler" }

    async fn should_track(&self, _thread: &Thread, _ctx: &ThreadContext) -> Result<bool, ThreadBotError> {
        Ok(true) // track all threads
    }

    async fn handle(&self, thread: &Thread, _ctx: &ThreadContext) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        Ok(vec![ThreadEffect::Reply {
            message: format!("Received {} messages", thread.messages.len()),
            metadata: serde_json::Value::Null,
        }])
    }
}
```

Full working example: [`examples/hello_thread_bot`](examples/hello_thread_bot/src/main.rs).

## Middleware

Middlewares wrap plugins to add cross-cutting behavior:

```rust
use mattermost_bot::middlewares::IgnoreSelf;

// Bot will ignore its own messages
let plugin = IgnoreSelf::new(EchoPlugin);
bot.with_plugin(plugin);
```

## Running

```bash
# Environment variables
export MM_BASE_PATH=http://localhost:8065
export MM_BEARER_TOKEN=your_bot_token

# Run the example (PostgreSQL required for thread-bot)
cargo run --example hello_thread_bot
```

### Getting a bot token

1. Mattermost → Integrations → Bot Accounts → create a bot
2. Copy the Access Token
3. Add the bot to the desired channels/teams

## Development

```bash
cargo fmt                          # formatting
cargo clippy -- -D warnings        # linting
cargo test                         # all tests
cargo test --doc                   # doctests only
```

## Project structure

```
mmbot/
├── lib/
│   ├── mattermost-api/         # Layer 1: API client (auto-generated)
│   ├── mattermost-bot/         # Layer 2: framework (Bot, Plugin, middleware)
│   ├── thread-bot/             # Layer 3: thread-based bots
│   ├── bool_parser/            # Utility for Mattermost API quirks
│   └── mattermost-test-helpers/# Integration test helpers
└── examples/
    └── hello_thread_bot/       # Thread bot example
```

## Tech stack

Rust 2021 · Tokio · reqwest · serde · SQLx (PostgreSQL) · tracing
