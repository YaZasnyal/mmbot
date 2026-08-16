# RFC 0008: Qwen Code через ACP как runtime для Layer 4

**Статус**: ACP-only реализация готова; ожидается сравнение `master` и ветки в тестовой среде
**Дата**: 2026-08-15
**Автор**: Обсуждение в парном программировании

## Содержание

- [Резюме](#резюме)
- [Контекст](#контекст)
- [Решение](#решение)
- [Архитектура](#архитектура)
- [ACP integration](#acp-integration)
- [Обработка сообщений](#обработка-сообщений)
- [Qwen session](#qwen-session)
- [Support skill и runbooks](#support-skill-и-runbooks)
- [Инструменты](#инструменты)
- [Ответ пользователю](#ответ-пользователю)
- [Модель и provider](#модель-и-provider)
- [Ошибки и восстановление](#ошибки-и-восстановление)
- [Безопасность и наблюдаемость](#безопасность-и-наблюдаемость)
- [Процесс интеграции](#процесс-интеграции)
- [Критерии приемки](#критерии-приемки)
- [Рассмотренные альтернативы](#рассмотренные-альтернативы)
- [Последствия для текущего кода](#последствия-для-текущего-кода)
- [Открытые вопросы](#открытые-вопросы)

## Резюме

Layer 4 больше не реализует LLM client, tool loop или multi-agent
orchestration. Он запускает рабочий Qwen Code fork как ACP agent и общается с
ним через готовый Rust crate
[`agent-client-protocol`](https://docs.rs/agent-client-protocol/latest/agent_client_protocol/).

Mattermost остаётся каналом сообщений. Один Mattermost thread соответствует
одной Qwen session. На каждое пользовательское сообщение bridge отправляет в
эту session prompt с вызовом локального support skill. Qwen Code сам загружает
runbooks, вызывает MCP tools, при необходимости использует subagents и
возвращает финальный ответ. Bridge публикует этот ответ в Mattermost.

Rust-код не знает о scout, communicator или способе их взаимодействия. Роли,
правила диагностики и стиль ответа описываются в skills, которые можно сначала
запускать и отлаживать локально обычной командой Qwen Code.

## Контекст

Текущий [RFC 0004](0004-support-bot-layer.md) выбрал собственный agent loop в
[`lib/support-bot`](../lib/support-bot/). Crate самостоятельно реализует:

- OpenAI-compatible Chat Completions client;
- prompt assembly и восстановление tool trace;
- registry локальных и remote MCP tools;
- tool loop, лимиты и workflow actions;
- фильтрацию пользовательского текста;
- часть session state в Mattermost metadata.

Рабочий Qwen Code fork лучше следует инструкциям и уже работает с целевой
моделью и provider. Qwen Code поддерживает ACP, Agent Skills, MCP, sessions и
subagents. Повторять эти возможности в Rust не требуется.

ACP уже имеет официальный Rust SDK. Он предоставляет client/agent roles,
готовое соединение, запуск внешнего ACP agent, создание и продолжение sessions,
отправку prompts, streaming updates и permission callbacks:

- [официальный Rust SDK](https://github.com/agentclientprotocol/rust-sdk);
- [Rust API](https://docs.rs/agent-client-protocol/latest/agent_client_protocol/);
- [Qwen Code channel/ACP architecture](https://qwenlm.github.io/qwen-code-docs/en/design/channels/channels-design/).

## Цели

1. Заменить собственный harness на рабочий Qwen Code fork.
2. Использовать готовый `agent-client-protocol`, не реализуя ACP вручную.
3. Хранить conversation state в Qwen session.
4. Передавать пользовательский запрос вместе с явным вызовом support skill.
5. Оставить runbooks локальными Markdown/Agent Skills.
6. Предоставить Qwen диагностические tools через MCP.
7. Работать с заданными model и provider через конфигурацию Qwen Code.
8. Сохранить Mattermost transport и thread serialization в Layers 1–3.

## Не-цели

1. Не описывать в Rust роли внутренних Qwen agents.
2. Не создавать agent mailbox или multi-agent protocol.
3. Не переносить Qwen agent loop в [`support-bot`](../lib/support-bot/).
4. Не писать собственный JSON-RPC/ACP transport.
5. Не проксировать MCP tool calls через Mattermost bridge.
6. Не публиковать reasoning, tool events или raw logs в Mattermost.
7. Не давать destructive tools в первой версии.

## Решение

Проект использует Qwen Code как внешний ACP agent process. Thin bridge:

1. запускает или подключает Qwen Code через `agent-client-protocol`;
2. создаёт/возобновляет ACP session для Mattermost thread;
3. отправляет prompt с вызовом локального support skill;
4. читает session updates;
5. публикует только финальный agent response;
6. сохраняет ACP session ID и delivery cursor.

Всё остальное принадлежит Qwen Code и support skill:

- system behavior;
- выбор runbook;
- анализ logs/metrics;
- MCP tool calls;
- subagents и их коммуникация;
- context compaction;
- формулировка применимого пользовательского ответа.

[RFC 0004](0004-support-bot-layer.md) считается заменённым в части LLM client,
tool registry, prompt assembly, tool loop и resume-state.

[RFC 0006](0006-markdown-instruction-repository.md) сохраняет цель обычной
Markdown-базы знаний, но её runtime loading выполняет Qwen Agent Skills.

## Архитектура

```text
Mattermost user thread
        │
        ▼
mattermost-bot + thread-bot
        │ ThreadInvocation
        ▼
thin Layer 4 ACP bridge
        │ agent-client-protocol
        ▼
Qwen Code fork --acp
        │ one Qwen session per Mattermost thread
        │
        ├── local support skill
        ├── Markdown runbooks
        ├── MCP logs/metrics tools
        └── optional Qwen-managed subagents
        │
        ▼ final agent response
ThreadEffect::Reply → Mattermost
```

### Компоненты

| Компонент | Ответственность |
| --- | --- |
| [`mattermost-bot`](../lib/mattermost-bot/) | Mattermost API, WebSocket и plugin lifecycle |
| [`thread-bot`](../lib/thread-bot/) | Thread tracking, persistence и per-thread serialization |
| ACP bridge | Process lifecycle, thread/session mapping, prompts и финальный response |
| `agent-client-protocol` | ACP connection, sessions, streaming и callbacks |
| Qwen Code fork | Agent runtime, model/provider, skills, MCP и subagents |
| Support skill | Runbook workflow, safety rules и формат ответа |

## ACP integration

Bridge использует официальный crate как ACP client. Собственные ACP message
types, JSON-RPC router и stdio framing не создаются.

Минимальный lifecycle:

1. Запустить configured Qwen executable с ACP mode либо подключиться к уже
   запущенному ACP transport.
2. Выполнить ACP initialize и проверить capabilities.
3. Создать новую session или возобновить сохранённую.
4. Отправить prompt.
5. Читать session updates до финального response или ошибки.
6. Закрыть session/connection при shutdown.

Предпочтительный transport для первой версии — дочерний Qwen process со
stdin/stdout. HTTP daemon не нужен, пока один bot process владеет одним Qwen
runtime. Его можно добавить позже без изменения Mattermost/Qwen session model.

Bridge реализует ACP client callbacks только там, где это необходимо:

- permission request: deny по умолчанию, разрешать только утверждённые tools;
- filesystem callbacks: не предоставлять, если support skill не работает с
  workspace files;
- terminal callbacks: не предоставлять в первой версии;
- session updates: собирать final text и безопасные operational events.

Версия `agent-client-protocol` и поддерживаемая ACP version Qwen fork должны
быть зафиксированы в workspace dependencies и contract test.

## Обработка сообщений

1. [`thread-bot`](../lib/thread-bot/) получает новый Mattermost post и
   сериализует обработку внутри thread.
2. Bridge проверяет admission/routing и загружает сохранённый ACP session ID.
3. При первом сообщении bridge создаёт Qwen session и сохраняет mapping.
4. Bridge отправляет prompt:

```text
/support

Mattermost thread: <thread-id>
Mattermost post: <post-id>
User message:
<message text>
```

5. Qwen запускает локальный `support` skill.
6. Skill выбирает runbook, вызывает MCP tools и выполняет нужный внутренний
   workflow. Qwen может использовать subagents, но bridge этого не отслеживает.
7. Bridge игнорирует reasoning/tool updates и ждёт финальный agent response.
8. Финальный текст публикуется через `ThreadEffect::Reply`.
9. Следующее сообщение пользователя отправляется в ту же Qwen session.

Если Qwen просит уточнение, это обычный финальный ответ. Пользователь отвечает
в Mattermost, а bridge продолжает ту же session. Отдельная workflow state
machine для `ask_user` не нужна.

Входящие posts не отправляются параллельно в одну ACP session. Новое сообщение
ждёт завершения текущего prompt. Явная команда остановки может отменить
активный ACP request.

## Qwen session

Один Mattermost root thread соответствует одной ACP/Qwen session. Qwen session
является источником conversation history, loaded skills, tool history и
compaction state.

Thread metadata хранит только integration state:

```json
{
  "support_runtime": {
    "version": 1,
    "acp_session_id": "session-id",
    "last_inbound_post_id": "post-id",
    "last_outbound_post_id": "post-id",
    "status": "active|failed"
  }
}
```

Полный Qwen transcript не копируется в PostgreSQL metadata. Полный Mattermost
thread не пересобирается в prompt на каждом ходе.

Sessions разных Mattermost threads изолированы. Если support workflow должен
использовать решения из прошлых обращений, skill вызывает отдельный
ACL-aware read-only tool, а не читает чужие transcripts напрямую.

## Support skill и runbooks

Локальный `support` skill — основной способ управлять поведением Qwen. Его
можно запускать тем же prompt вручную до подключения Mattermost.

Skill обязан:

- выбирать и загружать подходящие runbooks on demand;
- использовать tools для внутренних данных, а не выдумывать результаты;
- не просить пользователя смотреть недоступные ему logs или admin systems;
- задавать только вопросы, на которые пользователь может ответить;
- отделять подтверждённые факты от гипотез;
- не включать reasoning, tool traces, credentials и raw logs в финальный ответ;
- честно сообщать, когда данных недостаточно;
- выдавать только применимый пользователю итог или уточняющий вопрос.

Роли вроде scout/communicator можно описывать внутри skill и запускать через
возможности Qwen Code. Для bridge это implementation detail.

Runbooks остаются Markdown и используют progressive disclosure:

- `SKILL.md` содержит routing и общий workflow;
- конкретные сценарии и команды лежат в references;
- Qwen загружает только нужные документы;
- frontmatter и internal links проверяются до запуска.

Миграция из
[`examples/support_bot/instructions`](../examples/support_bot/instructions/)
не должна создавать вторую вручную поддерживаемую копию базы знаний.

## Инструменты

Диагностические integrations предоставляются Qwen Code как MCP servers. Bridge
не регистрирует model-facing tools и не выполняет tool calls.

В первой версии support runtime получает только необходимые read-only tools:

- поиск logs по разрешённым identifiers и временному диапазону;
- чтение metrics/status;
- поиск известных случаев;
- загрузка support knowledge, если она не полностью представлена skills.

MCP service отвечает за ACL, redaction secrets, timeout и ограничение размера
результата. Shell, filesystem write, deployment и mutation tools запрещены.

## Ответ пользователю

Bridge публикует только финальный text response ACP session. Следующие события
не являются ответом и остаются внутренними:

- reasoning/thinking updates;
- tool calls и tool results;
- permission requests;
- subagent progress;
- intermediate assistant messages.

Если финальный response пуст, ACP/Qwen завершился с ошибкой или нарушил
протокол, bridge отправляет один короткий service message и сохраняет
диагностику для инженера. Эвристический парсинг `<think>` не используется как
основная граница безопасности.

Повторная доставка предотвращается связкой входного Mattermost post ID и
сохранённого outbound post ID. После restart один prompt не должен создавать
два ответа.

## Модель и provider

Model, provider, credentials и model-specific parameters задаются в Qwen Code
fork. Rust bridge не формирует OpenAI/Anthropic requests.

Startup probe проверяет:

1. запуск Qwen ACP agent;
2. успешный ACP initialize;
3. выбранную model/provider комбинацию;
4. доступность support skill;
5. доступность обязательных MCP servers;
6. выполнение одного test prompt.

Silent fallback на другую модель запрещён, если он явно не настроен в Qwen.

## Ошибки и восстановление

| Ошибка | Поведение |
| --- | --- |
| Qwen process не запустился | Bounded restart, затем один service response и engineer log |
| ACP initialize несовместим | Fail startup; не включать legacy loop автоматически |
| Session не найдена после restart | Создать новую session с коротким recovery context и записать warning |
| Model/provider error | Сохранить correlation context; вернуть безопасный service response |
| MCP tool error | Qwen/skill должен сообщить об отсутствии данных, а не придумать результат |
| Prompt timeout | Отменить request, перезапустить runtime при необходимости |
| Duplicate Mattermost event | Не отправлять повторный ACP prompt |
| Duplicate final response | Не создавать повторный Mattermost post |

## Безопасность и наблюдаемость

1. ACP permission callbacks используют deny-by-default.
2. Qwen process получает только необходимые credentials и MCP configuration.
3. Mattermost user text считается недоверенным input.
4. Project-local Qwen/MCP configuration требует review.
5. Raw tool results и reasoning не сохраняются в Mattermost metadata.
6. Logs связывают Mattermost thread/post, ACP session и request IDs.
7. IDs не используются как metric labels.
8. Model/provider и skill version записываются в audit summary.

Существующие observability decisions из
[RFC 0005](0005-observability-metrics.md) сохраняются для Layers 1–3. Layer 4
добавляет ACP process/session/prompt duration и result class.

## Процесс интеграции

### Фаза 0: локальный spike

До изменений production path:

1. Зафиксировать Qwen executable и ACP version рабочего fork.
2. Подключить `agent-client-protocol` в маленьком Rust spike.
3. Запустить Qwen как ACP agent.
4. Создать session и отправить `/support` с тестовым запросом.
5. Проверить runbook loading и один read-only MCP tool.
6. Получить финальный ответ без reasoning/tool output.
7. Отправить уточнение в ту же session.
8. Возобновить session после restart.

Subagents не являются gate для этого spike. Если support skill использует их,
они проверяются как часть Qwen behavior, а не ACP bridge API.

### Фаза 1: ACP-only feature branch

Сделать Qwen ACP единственным Layer 4 path и удалить custom LLM client, prompt
assembly, tool registry, MCP client, tool loop и связанные tests/docs. Не
добавлять runtime fallback или rollout switch в одну сборку.

### Фаза 2: проверка ветки

Проверить уточняющие вопросы, несколько ходов, restart, duplicate input,
безопасный user error и подробный отчёт в engineer thread.

### Фаза 3: независимое сравнение

В рабочей тестовой среде отдельно запустить версию из `master` и версию из
feature branch. После подтверждения поведения merge ветки становится cutover;
rollback выполняется возвратом deployment на предыдущую revision.

## Критерии приемки

- предоставленная model/provider комбинация работает через Qwen fork;
- Rust использует `agent-client-protocol`, custom ACP code отсутствует;
- один Mattermost thread продолжает одну Qwen session;
- `/support` skill загружается при каждом новом session workflow;
- runbook и MCP evidence влияют на финальный ответ;
- пользователь не получает просьбу самостоятельно искать внутренние logs;
- reasoning, tool events и subagent progress не попадают в Mattermost;
- уточнение пользователя продолжает предыдущий контекст;
- restart восстанавливает или безопасно пересоздаёт session;
- duplicate events не создают duplicate responses;
- provider/runtime failure не запускает legacy harness;
- локальный prompt и Mattermost prompt дают сопоставимое поведение skill.

## Рассмотренные альтернативы

### Продолжать `lib/support-bot` agent loop

Отклонено: это повторная реализация возможностей Qwen Code.

### Собственный ACP client

Отклонено: официальный Rust crate уже предоставляет protocol types,
connection/session lifecycle и transports.

### Явные scout/communicator abstractions в Rust

Отклонено: роли принадлежат skills/Qwen runtime. Bridge должен видеть только
session, prompt, updates и final response.

### Прямой Qwen Mattermost channel plugin

Возможный следующий шаг. В первой версии сохраняются готовые
[`mattermost-bot`](../lib/mattermost-bot/) и [`thread-bot`](../lib/thread-bot/).
Если ACP bridge окажется единственным потребителем Layers 1–3, отдельный RFC
может заменить их нативным Qwen channel plugin.

### Общая Qwen session для всех Mattermost threads

Отклонено из-за смешивания пользователей и контекста. Междиалоговые знания
должны приходить через ACL-aware tool.

## Последствия для текущего кода

### Сохраняется

- [`mattermost-api`](../lib/mattermost-api/);
- [`mattermost-bot`](../lib/mattermost-bot/);
- [`thread-bot`](../lib/thread-bot/);
- admission/routing;
- Markdown runbooks;
- Mattermost delivery и engineer observability.

### Заменено в feature branch

- `llm.rs`;
- `conversation.rs`;
- `tools.rs`;
- `remote_mcp.rs`;
- `user_thread_run.rs`;
- tool loop в
  [`handler/user_workflow.rs`](../lib/support-bot/src/handler/user_workflow.rs);
- heuristic output sanitizing в `output.rs`.

### Добавляется

- dependency на `agent-client-protocol`;
- thin ACP bridge;
- Mattermost thread ↔ ACP session mapping;
- local `support` skill и его references;
- configured read-only MCP servers;
- ACP contract и behavior tests.

Реализация находится в существующем `support-bot`: `acp.rs` владеет Qwen/ACP,
а `handler/user_workflow.rs` связывает Mattermost thread с session.

## Решения локального spike

1. Проверенный Qwen Code `0.21.12` сообщает ACP `0.14.x`; Rust bridge использует
   `agent-client-protocol` `2.0.0` со stable ACP v1 API.
2. Fork запускается как дочерний process командой `qwen --acp`. Для support
   runtime задаётся короткий `--system-prompt`, иначе coding prompt почти
   исчерпывает контекст локальной модели до второго хода.
3. `session/load` после полного process restart проверен живым трёхходовым smoke;
   session ID сохраняется.
4. Canonical skill находится в [`.qwen/skills/support`](../.qwen/skills/support/)
   и ссылается на существующее дерево runbooks без копирования.
5. Один Qwen process принадлежит одному bot instance и обслуживает несколько
   изолированных sessions последовательно по Mattermost thread.
