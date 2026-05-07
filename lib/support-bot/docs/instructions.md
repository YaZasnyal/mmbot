# Instruction Repository Format

`support-bot` uses a small Markdown filesystem format for runbooks, skills, and
support instructions. The LLM navigates this tree through the `instructions`
tool instead of receiving the whole tree in context.

## Layout

```text
support-instructions/
├── index.md
├── diagnostics/
│   ├── index.md
│   ├── slow-requests.md
│   └── auth-errors.md
└── logs/
    ├── index.md
    └── search-playbook.md
```

`/index` is the default entry point. Section `index.md` files should link to
more specific documents.

## Documents

Each `.md` file must start with YAML frontmatter. Only `title` is required:

```markdown
---
title: Slow requests
---

# Slow requests

Use this runbook when a user reports elevated latency.

Related:

- [/logs/search-playbook]
```

Document ids are derived from relative paths:

- `index.md` -> `/index`
- `diagnostics/index.md` -> `/diagnostics/index`
- `diagnostics/slow-requests.md` -> `/diagnostics/slow-requests`

## Navigation Tool

The `instructions` tool loads one document at a time:

```json
{ "id": "/diagnostics/slow-requests" }
```

If `id` is omitted or `null`, the tool loads `/index`. Missing documents are
reported as tool errors in the tool result so the LLM can recover and choose a
valid linked document.

## Linting

Use `InstructionRepository::lint(root_path)` to validate authoring mistakes.
The lint pass checks:

- valid frontmatter and non-empty `title`;
- existence of `/index`;
- duplicate ids;
- broken internal links shaped exactly like `[/some/id]`.

Linting is a one-level static pass: it scans all files once, builds the set of
known ids, and checks the links written in each body. It does not recursively
follow links, so cycles between runbooks are safe.

## Migrating Existing Skills

Existing qwen-code skills should be converted into Markdown documents with
frontmatter. Put navigation in `index.md` files and link to concrete runbooks
with bracket links like `[/diagnostics/slow-requests]`. Large command outputs,
historical logs, or generated indexes should not be committed as instruction
content.
