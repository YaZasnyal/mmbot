# RFC 0008 implementation plan

## Constraints

- Use the official `agent-client-protocol` crate and Qwen Code `--acp`.
- The feature branch contains only the ACP runtime; `master` is the comparison build.
- Preserve admission, thread serialization, linked engineer threads, HTML export,
  audit metadata, control reactions, and Markdown runbooks.
- Persist integration state only. Never publish or store reasoning/tool events.

## Tasks

1. Launch Qwen, initialize ACP v1, create/load sessions, deny permissions, and
   collect only final agent-message text.
2. Persist the Mattermost thread to ACP session mapping and duplicate-input cursor.
3. Replace the Layer 4 Chat Completions/tool loop with one ACP handler path.
4. Delete legacy LLM, prompt, tool-registry, MCP-client, trace, config, docs, and tests.
5. Keep detailed bounded ACP reports in the linked engineer thread and safe user errors.
6. Wire the local `support` skill and Qwen executable/model/cwd/timeout in the example.
7. Run repository gates and a live local Qwen continuation/restart smoke.
8. Hand the branch to the user for an independent `master` versus branch comparison.
