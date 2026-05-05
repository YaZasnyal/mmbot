# Instruction Repository Format

`support-bot` uses a small filesystem format for runbooks, skills, and support
instructions. The LLM should navigate this tree through instruction tools
instead of receiving the whole tree in context.

## Layout

```text
support-instructions/
├── README.md
├── manifest.yaml
├── diagnostics/
│   ├── README.md
│   ├── slow-requests.md
│   └── auth-errors.md
└── logs/
    ├── README.md
    └── search-playbook.md
```

`README.md` files explain the purpose of a directory for humans. The bot uses
`manifest.yaml` as the source of truth.

## Manifest

```yaml
documents:
  - id: diagnostics.slow_requests
    path: diagnostics/slow-requests.md
    title: Slow requests
    summary: How to diagnose elevated latency in service requests.
    parent: diagnostics
    tags: [latency, timeout, performance]
```

Fields:

- `id`: stable identifier stored in thread/message metadata.
- `path`: relative Markdown path under the instruction root.
- `title`: short human-readable name.
- `summary`: one-sentence routing hint.
- `parent`: optional parent document id for hierarchical navigation.
- `tags`: normalized concepts shown to the LLM while browsing the tree.

## Navigation Tools

The repository is meant to back tools such as:

- `list_runbooks(parent_id?)`: list children with title and summary.
- `load_runbooks(ids)`: load full Markdown for one or more runbooks.

The LLM chooses which tree levels to inspect and which runbooks to load. The
handler enforces `max_load_documents` and `max_instruction_bytes` when
fulfilling tool calls.

## Migrating Existing Skills

Existing qwen-code skills should be converted into this format instead of read
directly. Keep procedural details in Markdown files, then add concise tags,
hierarchy, and summaries to the manifest. Large command outputs, historical
logs, or generated indexes should not be committed as instruction content.
