# Knowledge Base

The support bot knowledge base is a Markdown instruction repository loaded
through the `instructions` tool. It exists so the LLM can fetch only the
runbooks it needs instead of receiving a huge prompt.

## Repository Format

Instruction files live under:

```text
examples/support_bot/instructions/
```

The repository maps Markdown paths to ids:

- `index.md` -> `/index`
- `diagnostics/index.md` -> `/diagnostics/index`
- `diagnostics/timeout.md` -> `/diagnostics/timeout`
- `logs/search.md` -> `/logs/search`

Each document must start with YAML frontmatter:

```markdown
---
title: Timeout diagnostics
---

# Timeout diagnostics

Use this runbook when a user reports timeout errors.

Related:

- [/logs/search]
```

Only `title` is required today. Keep it short and human-readable.

## How The LLM Uses Instructions

The default support-bot system prompt requires the model to call
`instructions` before giving diagnostic or operational guidance:

1. Load `/index`.
2. Follow Markdown links like `[/diagnostics/timeout]`.
3. Use linked runbooks to choose tools and ask follow-up questions.
4. Report evidence or notify an engineer when blocked.

Do not rely on the model remembering runbooks from the system prompt. Put
procedural knowledge in instruction files.

## Recommended Structure

Use small routing documents plus focused runbooks:

```text
instructions/
├── index.md
├── diagnostics/
│   ├── index.md
│   ├── timeout.md
│   ├── elevated-errors.md
│   └── auth-failures.md
├── logs/
│   ├── index.md
│   └── search.md
├── metrics/
│   ├── index.md
│   └── latency.md
└── escalation/
    ├── index.md
    └── engineer-handoff.md
```

`/index` should be a router, not a giant manual. It should describe symptom
categories and link to the right section documents.

## Runbook Template

Use this shape for each operational runbook:

```markdown
---
title: Short title
---

# Short title

Use when:

- Concrete user symptom.
- Concrete product area.

Do not use when:

- Similar but different symptom that should route elsewhere.

Steps:

1. Ask for any missing required context.
2. Call `tool.name` with bounded arguments.
3. Compare the result against the expected healthy shape.
4. If evidence is enough, explain the finding to the user.
5. If evidence is insufficient, call `notify_engineer` with a compact handoff.

Tools:

- `tool.name`: what it does, required args, safe bounds, and what output means.

Escalate when:

- The tool returns an error.
- Required identifiers are missing.
- The result indicates a customer-impacting outage.
- The runbook confidence is low.

Related:

- [/other/document]
```

## Tool Guidance In Runbooks

When a runbook mentions a tool, include:

- when to call it;
- required arguments;
- allowed scopes or examples;
- max lookback or limit to use;
- what fields in the result matter;
- what to tell the user;
- when to notify an engineer.

Example:

```markdown
Call `logs.search` only after you know the service name and symptom keyword.
Use `since_minutes` between 15 and 60 unless the user gave a narrower incident
window. Start with `limit: 20`. If `truncated` is true, summarize the visible
sample and ask an engineer to run a deeper query.
```

## Writing Style

Good instruction documents are procedural and specific:

- Use numbered steps for workflows.
- Prefer concrete examples over abstract advice.
- Say exactly which tool to call.
- Say exactly which result fields matter.
- Keep one runbook focused on one symptom or workflow.
- Link sideways instead of duplicating large sections.

Avoid:

- broad product essays;
- long command reference dumps;
- copied backend output;
- credentials or private endpoints;
- instructions that tell the LLM to invent commands;
- vague lines like "investigate the logs" without a tool name and bounds.

## Linting

Use `InstructionRepository::load(root_path)` during startup when you want the
repository to log lint issues and then load valid documents. Use
`InstructionRepository::lint(root_path)` directly when readiness/startup policy
needs to fail on lint findings. The lint pass checks:

- valid frontmatter;
- non-empty title;
- existence of `/index`;
- duplicate ids;
- broken internal links shaped like `[/some/id]`.

For production, failing lint should make readiness fail and should usually make
startup fail. The current example logs lint errors and continues; a production
deployment should be stricter.

## Size Guidance

`InstructionRepository` returns full document bodies. The bot administrator owns
document size policy through repository authoring, review, and any custom tool
wrapper they choose to add.

Practical guidance:

- Keep routing documents under 2 KB.
- Keep runbooks under 8 KB when possible.
- Split large runbooks by symptom, service, or workflow stage.
- Put examples in the same document only when they directly affect tool use.

## Done Criteria

- `/index` links to every major support area.
- Every tool has at least one runbook that explains when to call it.
- Every runbook has frontmatter and a single H1 matching the title.
- `InstructionRepository::lint` returns no issues.
- The LLM can solve the happy path by loading `/index`, one section document,
  and one concrete runbook.
