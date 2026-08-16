# mmbot RFC 0008 — Orchestration contract

## Mission & product

Implement RFC 0008 as a thin ACP bridge from Mattermost support threads to the
installed Qwen Code runtime, retaining the linked engineer thread and detailed
diagnostic reports. The feature branch contains only the new runtime. Comparison
uses an independent build from `master`; no legacy runtime or rollout switch
remains in this branch.

## Phase map

| # | Phase | Gate | Bounce | Mode |
| --- | --- | --- | --- | --- |
| 0 | Local ACP spike | `cargo test -p support-bot acp` and local Qwen smoke | 0 | AUTO |
| 1 | ACP-only Layer 4 replacement | `cargo test -p support-bot` | 1 | AUTO |
| 2 | Repository validation | `just check && just test-all` | owning phase | AUTO |
| 3 | Workplace comparison | Human compares `master` and this branch in the test environment | 1 | HUMAN |
| 4 | Merge | Human merges after comparison | 2 | HUMAN |

## HITL points

- Phase 3: deploy separate `master` and feature-branch builds in the workplace test environment.
- Phase 4: merge after the comparison is accepted.

## Runnable commands

- `cargo test -p support-bot <filter>` — focused library gate.
- `cargo run -p support-bot-app` — local Mattermost run.
- `just fmt-check`, `just clippy`, `just build`, `just test-all` — repository gates.

## Hard rules

Read RFC 0008 and AGENTS.md before edits. Preserve Layers 1–3 and linked engineer
threads. Use the official `agent-client-protocol` crate; do not implement ACP,
Qwen roles, or a new tool loop. Publish only final ACP text to users. Never expose
reasoning, raw tool events, credentials, or logs. Do not stage or commit. Minimal
code: reuse existing boundaries and dependencies; add no speculative abstractions.

## Zone map & merge policy

One working tree owns this mission. Branch: `feat/rfc-0008-qwen-acp-runtime`.
No staging, commits, pushes, merges, or edits to generated Mattermost API code.

## Status & halt file

`notes/0008-implementation-status.md`

## Glossary

- ACP bridge: Rust client that owns Qwen process/session lifecycle only.
- Engineer thread: existing linked Mattermost thread for diagnostics and reports.

## Doc-sync map

| Change | Update |
| --- | --- |
| Runtime/configuration | `examples/support_bot/README.md` |
| Layer 4 architecture | `rfcs/0008-qwen-code-support-runtime.md` |
| Mission progress/blockers | `notes/0008-implementation-status.md` |
