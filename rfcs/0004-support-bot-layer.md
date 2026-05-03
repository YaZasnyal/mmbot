# RFC 0004: Layer 4 `support-bot` — LLM support bot поверх `thread-bot`

**Статус**: Черновик  
**Дата**: 2026-05-03  
**Автор**: Обсуждение в парном программировании

## Контекст

Layer 3 (`lib/thread-bot`) уже предоставляет базу для thread-based ботов:

- `ThreadHandler` trait;
- per-thread actor model с debounce и повторным запуском;
- persistence через `ThreadStore` / `PgThreadStore`;
- thread/message metadata;
- `ThreadEffect` для ответов, обновления metadata, reschedule и закрытия треда;
- `ThreadBotHandle` для cron и внешних действий.

Следующий слой нужен для конкретного сценария: бот технической поддержки, который ведёт тред, работает с LLM, вызывает инструменты, опирается на runbooks/skills и может продолжить выполнение после новых сообщений.

Важно сохранить границу: `thread-bot` остаётся generic Layer 3. LLM, tool calling, инструкции и бизнес-логика поддержки живут в Layer 4.

## Цели

1. Добавить экспериментальный crate `lib/support-bot` как каркас для support-ботов.
2. Реализовать `SupportBotHandler`, который использует `thread-bot::ThreadHandler`.
3. Поддержать OpenAI-compatible LLM через `/v1/chat/completions` и tool calling.
4. Дать единый tool registry для локальных Rust tools и внешних remote MCP tools.
5. Хранить состояние выполнения в metadata треда, чтобы обработка продолжалась после новых сообщений или рестарта.
6. Поддержать большой набор иерархических инструкций без загрузки всего дерева в контекст.
7. Описать документированный формат runbooks/skills и README для пользователей слоя.

## Не-цели

1. Не менять `thread-bot` ради LLM-специфики.
2. Не делать универсальный agent framework.
3. Не поддерживать Responses API в v1.
4. Не запускать локальные tools как отдельные MCP приложения.
5. Не проектировать stdio MCP transport; внешние tools должны быть доступны как remote MCP services.
6. Не вводить approval / permission layer для опасных действий.
7. Не выполнять разрушительные действия за инженера: бот может рекомендовать, сообщать и диагностировать.

## Архитектурное положение

```text
Layer 4: lib/support-bot
         LLM client, tool registry, instruction repository,
         support state machine, notifier
        ↓ implements
Layer 3: lib/thread-bot
         ThreadHandler, actors, persistence, Mattermost thread effects
        ↓ implements Plugin for
Layer 2: lib/mattermost-bot
        ↓ uses
Layer 1: lib/mattermost-api
```

`lib/support-bot` должен быть reusable skeleton: пользователь проекта подключает свои prompts, tools, runbooks и настройки, но не переписывает agent loop.

## Основная идея

`SupportBotHandler` получает свежий `Thread` snapshot от Layer 3. Он:

1. читает `SupportThreadState` из `thread.info.metadata`;
2. извлекает новые сообщения пользователя;
3. выбирает релевантные инструкции через `InstructionRepository`;
4. собирает LLM input: system prompt, выбранные инструкции, compact state, thread context;
5. вызывает OpenAI-compatible Chat Completions;
6. выполняет requested tool calls через `ToolRegistry`;
7. повторяет цикл до final response, wait, finish или лимита итераций;
8. возвращает `ThreadEffect`:
   - `Reply` / `UpdateMessage` для ответа пользователю;
   - `SetThreadMetadata` для сохранения state;
   - `Reschedule` если нужно продолжить выполнение;
   - `MarkResolved` / `MarkStopped` при завершении.

## LLM API

Для v1 фиксируем Chat Completions wire protocol:

- endpoint: `{base_url}/v1/chat/completions`;
- auth: OpenAI-style bearer token, optional для локальных endpoint;
- request: `model`, `messages`, `tools`, `tool_choice`, temperature/limits;
- response: assistant message, optional `tool_calls`;
- tool results возвращаются как `role = "tool"` messages.

Причина выбора: большинство OpenAI-compatible endpoint совместимы именно с Chat Completions, включая локальные и self-hosted варианты. Responses API можно добавить позже отдельным `LlmClient` implementation.

Публичная абстракция:

```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, SupportBotError>;
}
```

Default implementation: `OpenAiChatCompletionsClient` на `reqwest`.

## Tools

### Единая модель tools

Внутри `support-bot` все tools приводятся к общей модели:

```rust
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

pub struct ToolResult {
    pub call_id: String,
    pub content: serde_json::Value,
    pub is_error: bool,
}

pub enum ToolExecutionOutcome {
    ToolResult(ToolResult),
    Action(SupportAction),
}

pub enum ToolKind {
    ReadOnly,
    Notify,
    Workflow,
}
```

Эта модель адаптируется в OpenAI Chat Completions `tools` schema. `ToolKind` не является permission system; это metadata для prompt, документации и будущих UI/approval сценариев.

### Локальные tools

Локальные tools реализуются как Rust trait:

```rust
#[async_trait]
pub trait SupportTool: Send + Sync {
    fn spec(&self) -> ToolSpec;

    async fn call(
        &self,
        ctx: ToolContext,
        arguments: serde_json::Value,
    ) -> Result<ToolExecutionOutcome, SupportBotError>;
}
```

Локальный tool работает в том же процессе, имеет доступ к переданному `ToolContext` и не требует отдельного MCP server.

### Remote MCP tools

Для внешних инструментов v1 поддерживает remote MCP endpoints. `McpToolProvider` получает список remote endpoints из config, читает доступные tools и регистрирует их в `ToolRegistry`.

Stdio child-process MCP не поддерживается и не закладывается как целевой transport. Предполагается, что внешние MCP tools будут доступны по сети, например как отдельные сервисы в том же контейнерном окружении.

### Built-in workflow tools

Каркас должен предоставить минимальные встроенные workflow tools:

- `send_user_message` — подготовить ответ пользователю в текущий Mattermost thread;
- `notify_engineer` — отправить информацию или рекомендацию инженеру;
- `wait_for_user` — перевести тред в ожидание ответа пользователя;
- `finish_request` — завершить обработку обращения;
- пример read-only admin tool для просмотра данных сервиса.

Workflow tools являются pseudo-tools: LLM вызывает их через обычный tool calling, но `ToolRegistry` не выполняет внешний side effect напрямую. Вместо этого он возвращает `ToolExecutionOutcome::Action`, а `SupportBotHandler` в конце итерации превращает actions в один согласованный набор `ThreadEffect`.

```rust
pub enum SupportAction {
    SendUserMessage { message: String },
    NotifyEngineer { message: String },
    WaitForUser { reason: Option<String> },
    FinishRequest { summary: Option<String> },
}
```

Обычные local/MCP tools возвращают `ToolExecutionOutcome::ToolResult`, который добавляется в LLM context как tool result. Workflow actions не возвращаются модели как уже выполненный side effect; они завершают текущий loop и применяются handler-ом. Это предотвращает двойные сообщения и сохраняет всю запись state/effects в одном apply-step.

Бот не получает dangerous tools в v1. Если сценарий требует ручного действия, он пишет рекомендацию инженеру.

## State

Resume-состояние support-бота хранится в `Thread.info.metadata` как JSON:

```rust
pub struct SupportThreadState {
    pub version: u32,
    pub phase: SupportPhase,
    pub selected_instruction_ids: Vec<String>,
    pub compact_history: Vec<SupportHistoryItem>,
    pub last_action: Option<String>,
    pub engineer_thread: Option<EngineerThreadRef>,
}

pub struct EngineerThreadRef {
    pub channel_id: String,
    pub root_post_id: String,
}

pub enum SupportPhase {
    New,
    Diagnosing,
    WaitingForUser,
    WaitingForEngineer,
    Resolved,
}
```

Layer 4 обновляет state через `ThreadEffect::SetThreadMetadata`.

Thread state не должен становиться полным transcript store. Полные сообщения остаются в Mattermost/thread snapshot, а state хранит только то, что нужно для продолжения выполнения: фазу, выбранные инструкции, компактную историю reasoning-relevant действий и последний action marker.

## Message metadata и audit trail

Tool calls, LLM steps и диагностические артефакты лучше привязывать к конкретным сообщениям через message metadata, потому что Layer 3 уже возвращает эти metadata в следующих `Thread` snapshot.

Правило v1:

- thread metadata — только compact resume-state;
- message metadata — audit trail для конкретного bot/user message: tool calls, selected runbooks, LLM model, короткие summaries результатов;
- большие payloads tool results не хранятся целиком, только summary, ids или ссылки на внешнее хранилище/лог.

Для записи audit metadata Layer 4 может использовать `ThreadEffect::Reply { metadata }`, `ThreadEffect::UpdateMessage { metadata }` и store-level message metadata helpers, если они уже доступны через `ThreadContext`.

При новом сообщении пользователя `thread-bot` заново вызывает handler. `SupportBotHandler` читает state, добавляет новые сообщения в working context и продолжает с текущей фазы.

## Instruction repository

Инструкций много, поэтому нельзя грузить всё дерево в prompt.

Для v1 используем deterministic shortlist по manifest/index, без vector DB:

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

`manifest.yaml` описывает каждый документ:

```yaml
documents:
  - id: diagnostics.slow_requests
    path: diagnostics/slow-requests.md
    title: Slow requests
    summary: How to diagnose elevated latency in service requests.
    tags: [latency, timeout, performance]
    triggers:
      - "slow"
      - "timeout"
      - "долго отвечает"
```

`InstructionRepository`:

1. загружает manifest;
2. выбирает candidates по tags/triggers/summaries и текущему thread text;
3. применяет лимиты `max_context_instructions` и `max_instruction_bytes`;
4. загружает полный Markdown только для выбранных документов.

Manifest является source of truth. Любые будущие embeddings/vector indexes должны быть generated artifacts поверх manifest, а не отдельной ручной базой знаний.

Существующие qwen-code skills/runbooks не читаются как есть. Их нужно конвертировать в этот формат или написать adapter позже. Формат должен быть подробно описан в README, чтобы набор инструкций можно было поддерживать руками.

## Execution loop

Один `ThreadHandler::handle()` выполняет bounded loop:

```text
build context
↓
LLM call
↓
if tool_calls:
    execute tools
    append tool results
    repeat until max_tool_rounds
else:
    apply assistant action
```

Обязательные лимиты:

- `max_tool_rounds`;
- `max_tool_calls_per_round`;
- `max_tool_result_bytes`;
- `max_context_instructions`;
- `max_instruction_bytes`;
- LLM timeout.

Если лимит достигнут, бот должен сохранить state и уведомить инженера или пользователя, что требуется ручная обработка.

## Communication model

Сообщения разделяются на два sink-а:

1. user sink — текущий Mattermost thread;
2. engineer sink — отдельный канал, endpoint или custom notifier.

Engineer sink для Mattermost может быть настроен на другой канал. В этом режиме первое уведомление создаёт отдельный engineer thread в engineer channel, а последующие уведомления по тому же user thread отправляются reply-сообщениями в этот engineer thread. Связь `user_thread_id -> engineer_thread_root_post_id` хранится в `SupportThreadState` или thread metadata.

Публичная абстракция:

```rust
#[async_trait]
pub trait SupportNotifier: Send + Sync {
    async fn send_user_message(&self, thread: &Thread, message: String)
        -> Result<(), SupportBotError>;

    async fn notify_engineer(&self, thread: &Thread, message: String)
        -> Result<(), SupportBotError>;
}
```

Mattermost implementation может использовать `ThreadEffect::Reply` для user sink и Mattermost API для engineer sink. Если engineer sink пишет в отдельный канал, notifier должен идемпотентно создать engineer thread при первом уведомлении и переиспользовать его root post для следующих уведомлений. Встроенные tools должны идти через notifier, чтобы пользователь мог заменить транспорт.

### Future human approval

Human approval не входит в v1 execution loop, но должен быть совместим с communication model. Если позже появятся tools, требующие подтверждения, approval flow должен идти через engineer sink:

- бот отправляет инженеру карточку/сообщение с описанием действия;
- Mattermost implementation может добавить interactive buttons через Mattermost API;
- config содержит allowlist пользователей или групп, которые могут подтверждать действия;
- подтверждение возвращается в support-bot как отдельный внешний signal или сообщение, а не как прямой LLM side effect.

## Configuration

Примерная структура:

```rust
pub struct SupportBotConfig {
    pub llm: LlmConfig,
    pub instructions: InstructionConfig,
    pub tools: ToolConfig,
    pub limits: SupportBotLimits,
    pub engineer_notifications: EngineerNotificationConfig,
    pub approval: Option<ApprovalConfig>,
}

pub enum EngineerNotificationTarget {
    SameThread,
    MattermostChannel { channel_id: String },
    Custom,
}
```

Конфиг должен быть программно собираемым из Rust. Загрузка из TOML/YAML может быть примером или helper-ом, но не обязательной частью core API.

`ApprovalConfig` резервируется для будущего approval flow. В v1 он может содержать только allowlist инженеров и не включать enforcement.

## Документация

Для слоя нужны хорошие README:

1. `lib/support-bot/README.md` — что делает crate, как подключить к `ThreadBotPlugin`.
2. `lib/support-bot/docs/instructions.md` или `examples/.../README.md` — формат instruction tree и manifest.
3. README примера — как зарегистрировать local tools, remote MCP tools и prompts.

Документация должна явно сказать:

- локальные qwen-code skills нужно конвертировать в support-bot instruction format;
- LLM должен следовать выбранным инструкциям, а не придумывать диагностику;
- опасные действия не выполняются автоматически, вместо этого бот уведомляет инженера.

## План реализации

### Этап 1: каркас crate

- добавить `lib/support-bot` в workspace;
- определить config, error types, `LlmClient`, `SupportTool`, `ToolRegistry`, `InstructionRepository`;
- добавить README с экспериментальным статусом API.

### Этап 2: LLM + tool loop

- реализовать `OpenAiChatCompletionsClient`;
- реализовать преобразование `ToolSpec` в OpenAI tool schema;
- реализовать bounded execution loop с fake tests.

### Этап 3: state + ThreadHandler

- реализовать `SupportThreadState` serialization;
- реализовать `SupportBotHandler: ThreadHandler`;
- сохранять state через `SetThreadMetadata`;
- поддержать фазы `Diagnosing`, `WaitingForUser`, `WaitingForEngineer`, `Resolved`.

### Этап 4: instructions

- реализовать manifest parser;
- реализовать deterministic selection по triggers/tags/summaries;
- добавить пример instruction tree;
- описать конвертацию существующих skills/runbooks.

### Этап 5: MCP + examples

- добавить remote MCP provider;
- добавить example support bot с локальными tools;
- добавить пример подключения remote MCP log search.

## Тестирование

Unit tests:

- parsing instruction manifest;
- shortlist selection и загрузка только выбранных файлов;
- serialization/deserialization `SupportThreadState`;
- OpenAI request/response parsing, включая tool calls;
- tool loop: success, tool error, max rounds reached, wait for user, finish request.
- workflow pseudo-tools возвращают `SupportAction`, а не выполняют side effects напрямую.
- message metadata содержит audit summary для tool calls и выбранных инструкций.

Integration-style tests:

- fake `Thread` snapshot + fake `LlmClient` + fake tools;
- новый user message продолжает выполнение из metadata;
- `wait_for_user` сохраняет phase и не закрывает тред;
- `finish_request` сохраняет phase и возвращает close effect.

MCP tests:

- remote MCP provider на mock endpoint;
- недоступный MCP endpoint не должен ломать весь bot startup, если настроен как optional.

## Открытые вопросы

1. Когда deterministic manifest selection перестанет хватать, нужен ли embedding/vector index как generated artifact.
2. Какой минимальный shape Mattermost interactive approval должен быть, если появятся tools с подтверждением.
3. Нужен ли отдельный trace sink помимо message metadata для отладки больших LLM/tool payloads.

## Принятые решения

1. Новый слой делаем как `lib/support-bot`, а не как example-only app.
2. API crate на первом этапе экспериментальный.
3. LLM v1 — OpenAI-compatible Chat Completions.
4. Локальные tools — Rust trait, без отдельного приложения.
5. Внешние tools — remote MCP endpoints.
6. Stdio MCP transport не поддерживаем и не проектируем для v1.
7. Resume-state — в thread metadata; audit trail tool/LLM steps — в message metadata.
8. Инструкции — custom documented filesystem format с manifest/index; embeddings позже только как generated index.
9. Workflow actions (`send_user_message`, `wait_for_user`, `finish_request`) — pseudo-tools, которые возвращают `SupportAction`.
10. Safety v1 — не выдавать боту dangerous tools; вместо опасных действий уведомлять инженера.
11. Future approval должен строиться через engineer sink и allowlist инженеров.
