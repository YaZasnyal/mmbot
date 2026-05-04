# RFC 0005: Observability и метрики для bot workspace

**Статус**: Черновик  
**Дата**: 2026-05-04  
**Автор**: Обсуждение в парном программировании

## Контекст

В workspace уже есть несколько слоёв, которые отвечают за разные части
жизненного цикла Mattermost-бота:

- Layer 2 `lib/mattermost-bot` держит WebSocket connection, декодирует события,
  управляет plugin lifecycle и fanout событий по плагинам;
- Layer 3 `lib/thread-bot` превращает события в tracked threads, управляет
  per-thread actors, persistence, reconciliation и `ThreadEffect`;
- Layer 4 `lib/support-bot` реализует LLM support workflow, tools,
  инструкции, engineer notifications и debug export.

Для эксплуатации support-бота нужны метрики не только на LLM/tool уровне.
Важно видеть, сколько акторов живёт, сколько сообщений приходит через
WebSocket, где теряются события, сколько времени занимают handler runs,
сколько ошибок возникает при записи в store и как ведут себя несколько ботов
в одном процессе.

## Цели

1. Добавить единый, opinionated подход к метрикам во всех runtime-слоях.
2. Использовать `prometheus-client` как выбранную библиотеку для registry,
   metric families и text/OpenMetrics exposition.
3. Сделать метрики опциональными: все crates должны работать без настройки
   метрик и без глобального registry.
4. Разрешить несколько ботов в одном процессе и одном registry через
   обязательный label `bot`.
5. Развести метрики по слоям так, чтобы каждый слой измерял только то, что
   реально знает.
6. Зафиксировать label policy, чтобы избежать high-cardinality метрик и утечек
   пользовательских данных.

## Не-цели

1. Не добавлять HTTP `/metrics` server в library crates.
2. Не вводить глобальный metrics singleton или default registry.
3. Не тащить LLM/tool-specific метрики в `thread-bot`.
4. Не размечать метрики thread/post/channel/user IDs.
5. Не строить tracing backend, distributed tracing или log correlation в рамках
   этого RFC.

## Выбор библиотеки

Выбираем crate `prometheus-client`.

Причины:

- явный `prometheus_client::registry::Registry`, который можно создать снаружи
  и передать в application wiring;
- typed `Family` для label sets;
- encoding в text/OpenMetrics format через `prometheus_client::encoding::text`;
- нет необходимости использовать глобальный registry.

Application code отвечает за создание registry и exposition endpoint. Library
crates только регистрируют metric families и инкрементят handles.

## Основная модель API

Каждый слой получает свой metrics module:

```rust
mattermost_bot::metrics::{
    MattermostBotMetrics,
    MattermostBotMetricsHandle,
}

thread_bot::metrics::{
    ThreadBotMetrics,
    ThreadBotMetricsHandle,
}

support_bot::metrics::{
    SupportBotMetrics,
    SupportBotMetricsHandle,
}
```

Паттерн в каждом модуле одинаковый:

```rust
let mut registry = prometheus_client::registry::Registry::default();

let mm_metrics = MattermostBotMetrics::register(&mut registry);
let thread_metrics = ThreadBotMetrics::register(&mut registry);
let support_metrics = SupportBotMetrics::register(&mut registry);

let bot_name = "support_bot";

let handler = SupportBotBuilder::new(bot_name, config, llm)
    .with_metrics(support_metrics.for_bot(bot_name))
    .build()
    .await?;

let plugin = ThreadBotPlugin::new(handler, store)
    .with_metrics(thread_metrics.for_bot(bot_name));

let mut bot = Bot::with_config(mm_config)?
    .with_metrics(mm_metrics.for_bot(bot_name))
    .with_plugin(plugin);
```

`register()` вызывается один раз на process/application wiring. `for_bot()`
создаёт дешёвый cloneable handle с фиксированным label `bot`.

Если metrics handle не передан, слой использует no-op path.

## Labels

### Обязательные labels

- `bot` — стабильное имя экземпляра бота внутри процесса. По умолчанию удобно
  использовать `ThreadHandler::id()` / `SupportBotBuilder::id`.

### Разрешённые labels

- `plugin` — статический `Plugin::id()`;
- `handler` — статический `ThreadHandler::id()`;
- `route` — `user`, `engineer`, `ignored`;
- `event_type` — нормализованный Mattermost event kind;
- `message_kind` — `text`, `ping`, `other`;
- `command` — actor command kind;
- `effect` — `ThreadEffect` kind;
- `tool_name` — имя зарегистрированного support tool;
- `target` — `user`, `engineer`, `same_thread`;
- `hook` — plugin lifecycle hook;
- `outcome` — `success`, `error`, `skipped`, `cancelled` и похожие
  bounded значения;
- `reason` — bounded enum reason, например close/reconnect reason.

### Запрещённые labels

Нельзя использовать:

- thread IDs, post IDs, channel IDs, user IDs;
- сырые error strings;
- URL, token, model prompt, message text;
- tool arguments или произвольные user-provided значения.

Исключение для `tool_name` допустимо, потому что tools регистрируются
конфигурацией приложения и являются bounded набором.

## Layer 2: `mattermost-bot`

Layer 2 знает WebSocket lifecycle и plugin fanout.

Метрики:

- `mattermost_bot_ws_connections_total{bot,outcome}` — попытки подключения;
- `mattermost_bot_ws_connection_duration_seconds{bot,outcome}` — длительность
  одной WS сессии;
- `mattermost_bot_ws_messages_total{bot,message_kind,outcome}` — входящие WS
  messages и parse outcome;
- `mattermost_bot_events_total{bot,event_type}` — успешно нормализованные
  события;
- `mattermost_bot_plugin_events_total{bot,plugin,event_type,outcome}` — fanout
  событий по plugins;
- `mattermost_bot_plugin_event_duration_seconds{bot,plugin,event_type}` —
  длительность `Plugin::process_event`;
- `mattermost_bot_plugin_lifecycle_total{bot,plugin,hook,outcome}` —
  `on_start` / `on_shutdown`.

Layer 2 не знает thread IDs и не должен измерять thread-specific семантику.

## Layer 3: `thread-bot`

Layer 3 знает tracked threads, actor lifecycle, handler runs, effects,
reconciliation и store calls.

Метрики:

- `thread_bot_active_actors{bot,handler}` — gauge активных actor tasks;
- `thread_bot_actor_starts_total{bot,handler,outcome}`;
- `thread_bot_actor_stops_total{bot,handler,reason}`;
- `thread_bot_actor_commands_total{bot,handler,command,outcome}`;
- `thread_bot_handler_runs_total{bot,handler,outcome}`;
- `thread_bot_handler_duration_seconds{bot,handler,outcome}`;
- `thread_bot_effects_total{bot,handler,effect,outcome}`;
- `thread_bot_reconcile_runs_total{bot,handler,outcome}`;
- `thread_bot_reconcile_duration_seconds{bot,handler,outcome}`;
- `thread_bot_store_operations_total{bot,handler,operation,outcome}`;
- `thread_bot_store_operation_duration_seconds{bot,handler,operation,outcome}`.

Для store metrics предпочтителен wrapper `InstrumentedThreadStore`, который
реализует `ThreadStore` и делегирует в inner store. Так не нужно загрязнять
`PgThreadStore` метриками и можно инструментировать любой store implementation.

Gauge actor count должен увеличиваться при spawn actor task и уменьшаться в
task cleanup перед удалением sender из actor map.

## Layer 4: `support-bot`

Layer 4 знает support-specific route, LLM loop, tools, workflow actions,
engineer notification и user-visible replies.

Метрики:

- `support_bot_thread_events_total{bot,route,event,outcome}`;
- `support_bot_handle_duration_seconds{bot,route,outcome}`;
- `support_bot_llm_requests_total{bot,outcome}`;
- `support_bot_llm_duration_seconds{bot,outcome}`;
- `support_bot_tool_calls_total{bot,tool_name,outcome}`;
- `support_bot_tool_duration_seconds{bot,tool_name,outcome}`;
- `support_bot_replies_total{bot,target,outcome}`;
- `support_bot_thread_closes_total{bot,reason,outcome}`.

LLM model можно логировать через tracing, но не добавлять label по умолчанию:
в разных deployment model name может быть пользовательским или high-cardinality.

## Example wiring

`examples/support_bot` должен показать полный wiring:

1. создать один `Registry`;
2. зарегистрировать metrics families для всех трёх слоёв;
3. создать handles через `for_bot("support_bot")`;
4. передать handles в `SupportBotBuilder`, `ThreadBotPlugin` и `Bot`;
5. оставить exposition endpoint вне v1 implementation или вынести его в
   отдельный пример/документацию.

Это сохраняет library crates независимыми от `axum`, `hyper` или другого HTTP
stack.

## Тестирование

1. No-metrics path: каждый слой ведёт себя как раньше, если handle не передан.
2. Shared registry: два bot labels в одном registry дают отдельные series.
3. `mattermost-bot`: synthetic WS/event path считает parse success/error и
   plugin fanout duration.
4. `thread-bot`: actor spawn/stop меняет gauge; handler success/error меняет
   counters; effects пишут bounded labels.
5. `InstrumentedThreadStore`: success/error outcomes и duration пишутся вокруг
   store methods.
6. `support-bot`: LLM success/error, tool success/error и ignored route
   отражаются в metrics.

Минимальный набор команд:

```bash
cargo test -p mattermost-bot
cargo test -p thread-bot
cargo test -p support-bot
```

## План внедрения

1. Добавить workspace dependency `prometheus-client`.
2. Добавить metrics modules и no-op handles во все три runtime crates.
3. Подключить optional `.with_metrics(...)` builder methods без изменения
   default behavior.
4. Инструментировать `support-bot` LLM/tools/routes.
5. Инструментировать `thread-bot` actors/handler/effects/reconcile.
6. Инструментировать `mattermost-bot` WS/plugin lifecycle.
7. Добавить `InstrumentedThreadStore`.
8. Обновить `examples/support_bot` README с примером registry wiring.

## Открытые вопросы

1. Нужен ли готовый `/metrics` HTTP helper в example crate, или application code
   должен всегда выбирать transport сам?
2. Добавлять ли process/runtime metrics отдельным optional feature позже?
3. Нужен ли label `model` для LLM requests как opt-in, если deployment
   гарантирует bounded model set?
