# RFC 0006: Markdown-only instruction repository для `support-bot`

**Статус**: Черновик  
**Дата**: 2026-05-07  
**Автор**: Обсуждение в парном программировании

## Контекст

`lib/support-bot` сейчас использует `InstructionRepository`, который получает
дерево runbooks из `manifest.yaml`. Manifest хранит `id`, `path`, `title`,
`summary`, `parent` и `tags`, а LLM навигирует по базе знаний через tool action
`list` и затем загружает документы через action `load`.

Эта схема работает, но создаёт лишний слой поддержки для автора runbooks:

- каждый новый Markdown-документ нужно вручную добавить в manifest;
- навигация живёт отдельно от самих runbooks;
- index-документы для людей и machine-readable index для бота расходятся;
- broken links между runbooks не видны до runtime.

Нужна более простая модель: база знаний должна быть обычной директорией с
Markdown-файлами. Каждый файл сам описывает себя через YAML frontmatter, а
переходы между документами задаются обычными ссылками в Markdown body.

## Цели

1. Убрать обязательный `manifest.yaml` из runtime instruction repository.
2. Сделать `id` документа производным от относительного пути:
   - `index.md` -> `/index`;
   - `runbooks/403/index.md` -> `/runbooks/403/index`;
   - `runbooks/403/no-public-read-allowed.md` ->
     `/runbooks/403/no-public-read-allowed`.
3. Сделать `/index` дефолтной точкой входа, если LLM вызывает
   `instructions` без `id`.
4. Перенести навигацию в Markdown-документы через ссылки вида
   `[/runbooks/403/index]`.
5. Добавить lint API, чтобы авторы runbooks могли проверять frontmatter и
   internal links до запуска бота.
6. Сохранить tool name `instructions`, чтобы не менять общую архитектуру
   `SupportBotBuilder` и `ToolRegistry`.

## Не-цели

1. Не делать полнотекстовый поиск, embeddings или generated index.
2. Не проверять внешние URL и обычные Markdown-ссылки в первой версии.
3. Не требовать достижимости всех документов из `/index`.
4. Не поддерживать batch-loading нескольких документов в новом tool contract.
5. Не менять `workflow.rs`, lifecycle support-бота или remote MCP tools.

## Формат документов

Instruction repository является директорией с `.md` файлами:

```text
instructions/
├── index.md
├── diagnostics/
│   ├── index.md
│   └── timeout.md
└── logs/
    ├── index.md
    └── search.md
```

Каждый документ должен начинаться с YAML frontmatter:

```markdown
---
title: Timeout triage
---

# Timeout triage

Use this runbook when the user reports slow requests or request timeouts.

Related documents:

- [/logs/search]
```

Правила v1:

- frontmatter block обязателен;
- обязательное поле только `title`;
- `title` должен быть непустой строкой;
- неизвестные YAML-поля допустимы;
- body, передаваемый LLM, не включает frontmatter;
- документ без валидного frontmatter считается lint issue и не должен молча
  попадать в runtime repository.

## Document ids

`id` вычисляется из относительного пути под instruction root:

1. путь должен указывать на `.md` файл внутри root;
2. расширение `.md` удаляется;
3. path separator нормализуется в `/`;
4. вперед добавляется `/`.

Примеры:

| Path | Id |
| --- | --- |
| `index.md` | `/index` |
| `diagnostics/index.md` | `/diagnostics/index` |
| `runbooks/403/no-public-read-allowed.md` | `/runbooks/403/no-public-read-allowed` |

Path traversal через `..` и абсолютные пути запрещены.

## Tool contract

Tool name остаётся `instructions`.

Старый контракт:

```json
{
  "action": "list",
  "parent_id": null,
  "ids": ["diagnostics.timeout"],
  "limit": 20
}
```

заменяется на:

```json
{
  "id": "/diagnostics/timeout"
}
```

`id` опционален. Если поле отсутствует или равно `null`, repository грузит
`/index`.

Tool schema:

```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": ["string", "null"],
      "description": "Instruction document id. Omit or null to load /index."
    }
  },
  "additionalProperties": false
}
```

Успешный результат:

```json
{
  "document": {
    "id": "/diagnostics/timeout",
    "title": "Timeout triage"
  },
  "content": "# Timeout triage\n..."
}
```

Если документ не найден, tool возвращает `ToolResult` с `is_error = true`, но
не валит весь handler Rust-ошибкой:

```json
{
  "missing": "/diagnostics/unknown"
}
```

## Lint API

`InstructionRepository` должен предоставить публичный lint entrypoint,
например:

```rust
impl InstructionRepository {
    pub fn lint(root_path: impl AsRef<Path>) -> Result<Vec<InstructionLintIssue>>;
}
```

`InstructionLintIssue` должен быть публичным типом с достаточными данными для
CLI, тестов и future debug UI:

```rust
pub struct InstructionLintIssue {
    pub path: Option<PathBuf>,
    pub id: Option<String>,
    pub kind: InstructionLintIssueKind,
    pub message: String,
}
```

Минимальные issue kinds v1:

- invalid or missing frontmatter;
- missing or empty `title`;
- duplicate id;
- missing `/index`;
- broken internal link.

Internal link в первой версии — только bracket-ссылка exact shape
`[/some/id]`. Обычные Markdown-ссылки вроде `[text](target.md)` и external URL
не проверяются.

Линтер должен работать как одноуровневая статическая проверка, а не как
рекурсивный обход графа ссылок:

1. один раз просканировать все `.md` файлы под `root_path`;
2. построить полный set известных ids;
3. для каждого документа проверить только ссылки, которые явно встречаются в
   его body;
4. не переходить по найденным ссылкам и не запускать проверку связанных
   документов рекурсивно.

Это предотвращает бесконечные циклы на взаимных ссылках и делает результат
линтинга независимым от формы навигационного графа.

## Runtime loading

Repository должен сканировать `root_path` при создании и строить in-memory map:

```rust
HashMap<String, InstructionDocument>
```

где документ содержит как минимум:

```rust
pub struct InstructionDocument {
    pub id: String,
    pub path: PathBuf,
    pub title: String,
}
```

`summary`, `parent` и `tags` из старой manifest-модели больше не являются
обязательными runtime-полями. Если для source compatibility их нужно оставить
в публичных типах, они должны стать legacy/deprecated и не участвовать в новом
tool contract.

При чтении документа repository:

1. находит document по `id`;
2. читает файл через safe join внутри root;
3. парсит и отбрасывает frontmatter;
4. обрезает body по `max_instruction_bytes` на char boundary;
5. возвращает один документ.

## Модульная структура реализации

Новый implementation не должен расти в один большой
`lib/support-bot/src/instructions.rs`. Нужно превратить `instructions` в
модуль-директорию:

```text
lib/support-bot/src/instructions/
├── mod.rs
├── frontmatter.rs
├── lint.rs
├── repository.rs
└── tool.rs
```

Рекомендуемая ответственность:

- `mod.rs` — публичные exports и совместимость с текущим
  `pub mod instructions`;
- `frontmatter.rs` — разбор YAML frontmatter и отделение body;
- `repository.rs` — scan root, id derivation, safe path handling и loading;
- `tool.rs` — implementation `SupportTool` для `instructions`;
- `lint.rs` — `InstructionLintIssue`, issue kinds и одноуровневая статическая
  проверка ссылок.

Внешние импорты из `support_bot::instructions::{...}` должны продолжить
работать для новых публичных типов.

## Example и документация

Пример `examples/support_bot` должен перейти на новый формат:

- убрать `SUPPORT_INSTRUCTIONS_MANIFEST`;
- оставить `SUPPORT_INSTRUCTIONS_ROOT`;
- заменить `manifest.yaml` на `index.md` и nested index documents;
- добавить frontmatter во все example `.md`;
- добавить ссылки из `/index` в `/diagnostics/index` и `/logs/index`;
- добавить ссылки из section index в конкретные runbooks.

Документация, которую нужно обновить:

- `lib/support-bot/docs/instructions.md`;
- `lib/support-bot/README.md`;
- `examples/support_bot/README.md`;
- `examples/support_bot/TOOL_AUTHORING_GUIDE.md`.

System prompt в `SupportBotBuilder` должен продолжать говорить модели
использовать `instructions` tool перед диагностическими рекомендациями, но
может быть уточнён: начать с `/index`, затем переходить по Markdown-ссылкам.

## Тестирование

Минимальная проверка реализации:

- id derivation:
  - `index.md` -> `/index`;
  - nested file -> `/runbooks/403/no-public-read-allowed`;
  - path traversal rejected.
- frontmatter:
  - valid `title`;
  - missing title produces lint issue;
  - empty title produces lint issue;
  - loaded content excludes frontmatter.
- tool behavior:
  - empty args loads `/index`;
  - explicit `id` loads that document;
  - unknown `id` returns tool error result, not Rust error.
- lint:
  - valid internal links pass;
  - broken `[/missing]` link is reported;
  - duplicate/canonicalization behavior is fixed by test.

Команды проверки:

```bash
cargo test -p support-bot
cargo test -p support-bot-example
```

## Миграция

Существующий example manifest:

```yaml
documents:
  - id: diagnostics
    path: diagnostics/README.md
    title: Diagnostics
```

переезжает в:

```markdown
---
title: Diagnostics
---

# Diagnostics

Choose a concrete diagnostic article:

- [/diagnostics/timeout]
```

Авто-миграция не требуется. Достаточно обновить example и документацию.

## Открытые вопросы

1. Нужен ли отдельный CLI для lint, или достаточно public Rust API в первом
   шаге?
2. Нужно ли в будущем поддержать optional metadata вроде `summary`, `tags`,
   `owner`, `updated_at` для поиска и debug UI?
3. Нужно ли позже добавить правило достижимости всех документов из `/index`?
