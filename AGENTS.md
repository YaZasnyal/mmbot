# Repository Guidelines

## Project Structure & Module Organization

This repository is a Rust workspace for Mattermost bot development. Core crates live under `lib/`: `mattermost-api` is the generated Mattermost API client, `mattermost-bot` is the plugin and WebSocket framework, `thread-bot` adds thread actors and persistence, `bool_parser` handles API quirks, and `mattermost-test-helpers` supports integration tests. The runnable example is in `examples/hello_thread_bot`. SQL migrations are in `lib/thread-bot/migrations`. RFCs and design notes live in `rfcs/` and `notes/`.

## Build, Test, and Development Commands

Use `just` from the repository root when available:

- `just build`: run `cargo build` for the workspace.
- `just fmt` / `just fmt-check`: format or verify formatting with `rustfmt`.
- `just clippy`: run `cargo clippy -- -D warnings`.
- `just check`: run format check, clippy, and build.
- `just test-all`: run `cargo test --workspace`.
- `just test-full`: start Mattermost/Postgres, run integration tests, then stop services.

For quick local runs, use `cargo test`, `cargo test --doc`, or `cargo run --example hello_thread_bot`. The example expects `MM_BASE_PATH` and `MM_BEARER_TOKEN`; thread-bot flows also require PostgreSQL.

## Coding Style & Naming Conventions

Use Rust 2021 conventions and `cargo fmt` defaults. Keep modules and files in `snake_case`; public types and traits use `PascalCase`; functions, methods, and variables use `snake_case`. Prefer explicit error types already present in each crate, and keep generated `lib/mattermost-api/src/apis`, `models`, and `docs` changes separate from hand-written framework changes.

## Testing Guidelines

Prefer extracted module tests over inline `mod tests` blocks for new code, especially in `support-bot`: place tests under `src/tests/...` (for example `src/tests/handler.rs`) and connect them from the source file via `#[cfg(test)] #[path = "..."] mod tests;`. Unit tests may also live beside code when needed, such as `lib/thread-bot/src/actor_tests.rs`, or under crate-level `tests/`. Name tests by behavior, for example `test_reconnect_reconciles_threads`. Integration tests for `mattermost-bot` require the Compose environment from `docker-compose.yml` and should run serially with `--test-threads=1`. Add focused tests for middleware, lifecycle, persistence, and thread actor behavior when changing those areas.

## Commit & Pull Request Guidelines

Recent history uses both Conventional Commit style (`feat(thread-bot): ...`, `docs: ...`) and short imperative summaries. Prefer Conventional Commits for new work, with a scoped subject when useful. Pull requests should describe the behavior change, list test commands run, link related issues or RFCs, and call out generated API updates, migrations, or required Mattermost/Postgres setup.

Do not run git staging or commit commands unless the user explicitly asks for a git operation in the current turn. In particular, do not run `git add`, `git commit`, or similar commands just because a code slice is complete.

## Security & Configuration Tips

Do not commit bot tokens, admin passwords, or local Mattermost credentials. Use environment variables such as `MM_BASE_PATH`, `MM_BEARER_TOKEN`, `MATTERMOST_URL`, `MM_ADMIN_USER`, and `MM_ADMIN_PASS`. When changing tests, ensure `just test-env-stop` cleans up containers and volumes.
