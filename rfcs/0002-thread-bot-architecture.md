# RFC 0002: Layer 3 `thread-bot` — архитектура, интерфейсы и схема БД

**Статус**: Черновик  
**Дата**: 2026-04-06  
**Последнее обновление**: 2026-04-08  
**Автор**: Обсуждение в парном программировании

## Контекст

Layer 2 (`lib/mattermost-bot`) на текущем этапе считаем завершённым.

Он уже предоставляет:
- подключение к Mattermost WebSocket;
- нормализацию событий в `EventType`;
- `Plugin` trait и lifecycle hooks;
- middleware pattern;
- cron support;
- integration tests с реальным Mattermost.

Согласно [RFC 0001](0001-storage-persistence.md), персистентность не должна жить в Layer 2. Она переносится в Layer 3, потому что именно слой работы с тредами понимает:
- что такое активный тред;
- какие сообщения уже были обработаны;
- как восстанавливаться после рестарта;
- какие данные следующего слоя нужно хранить рядом с тредом.

Цель Layer 3 — дать фреймворк для thread-based ботов: получать события из Layer 2, группировать их по тредам, сериализовать обработку внутри одного треда, хранить состояние в Postgres и предоставлять Layer 4 удобный интерфейс работы с целым тредом.

## Цели

1. Реализовать `lib/thread-bot` как отдельный слой над `mattermost-bot`.
2. Сделать его `Plugin`-ом Layer 2.
3. Хранить thread state, message references и metadata в Postgres через `sqlx`.
4. Давать обработчику полный актуальный снимок треда из Mattermost API, обогащённый metadata из Postgres.
5. Поддержать restart reconciliation, включая поиск новых тредов в канале после downtime.
6. Поддержать управление тредом через реакции.
7. Сохранить хорошую тестируемость: unit + integration tests.

## Не-цели

1. Не делать generic persistence framework для Layer 2.
2. Не встраивать в Layer 3 LLM-логику, tool calling, RAG или бизнес-логику поддержки.
3. Не делать сложную state machine с десятками статусов на первом этапе.
4. Не пытаться решить все варианты concurrency кроме модели “один активный обработчик на тред”.

## Архитектурное положение в проекте

```text
Layer 4: concrete support plugins / business logic / LLM
        ↓ uses
Layer 3: lib/thread-bot
        ↓ implements Plugin for
Layer 2: lib/mattermost-bot
        ↓ uses
Layer 1: lib/mattermost-api
```

## Основная идея

`ThreadBot<H>`:
- реализует `mattermost_bot::Plugin`;
- получает сырые `Posted`, `ReactionAdded`, `ReactionRemoved` события;
- определяет `thread_id`;
- сохраняет / обновляет данные треда в `ThreadStore`;
- управляет runtime-состоянием активных тредов;
- запускает `ThreadHandler` с debounce;
- при новом сообщении делает `abort()` текущей задачи и планирует новый запуск;
- при рестарте делает catch-up по каналам и реконсиляцию активных тредов через Mattermost API.

Один экземпляр `ThreadBot` обслуживает один `ThreadHandler`.
Если нужна разная логика для разных каналов/правил, пользователь подключает несколько плагинов Layer 4, каждый со своим `ThreadBot`.

## Модель интереса к треду

Layer 3 не должен жёстко решать, какие треды “интересны”.

Решение гибридное:
- у `ThreadBot` есть базовая конфигурация, например список `channel_id`;
- события приходят из Layer 2;
- `ThreadHandler::should_track()` решает, нужно ли начинать сопровождать тред;
- обработчик может позже явно сообщить, что тред больше не нужен;
- отдельные mention/команды верхнего слоя могут быть частью логики `should_track()`.

Следствие: Layer 3 отвечает за lifecycle tracked thread, но не за бизнес-правила отбора.

## Thread statuses

На первом этапе используем 4 статуса:

```rust
pub enum ThreadStatus {
    New,
    Active,
    Resolved,
    Stopped,
}
```

### Семантика

- `New` — transient-состояние: тред уже признан интересным и может быть записан в БД, но первый полноценный handler run ещё не завершился.
- `Active` — тред отслеживается, для него может быть запланирован или уже выполняется handler.
- `Resolved` — тред завершён штатно, например по реакции `✅` на root post.
- `Stopped` — тред принудительно остановлен, например по реакции `🛑`.

На практике `New` может жить очень недолго. Его основная ценность — явно различать:
- тред, который только что принят в tracking;
- тред, который уже прошёл хотя бы один цикл обработки.

### Почему без `Ignored`

На этом этапе не вводим `Ignored`.
Если handler решил не заниматься тредом, Layer 3 просто не начинает tracking:
- не создаёт thread record;
- не сохраняет сообщения такого треда в БД;
- не планирует handler run.

То есть `should_track() == false` означает: этот тред полностью остаётся вне Layer 3 persistence.

### Thread progress markers

Помимо `status`, треду нужны два progress marker'а:

- `seen` — до какого сообщения Layer 3 уже точно знает состояние треда;
- `processed` — до какого сообщения handler уже отработал и его результат считается принятым текущим state'ом.

Это нужно для двух целей:
1. корректно переживать restart/reconciliation;
2. не запускать handler повторно на треде, где последнее актуальное сообщение уже наше собственное и нового пользовательского ввода не было.

Практическая семантика:
- если в результате обработки бот отправил ответ, то `processed` сдвигается до созданного bot message;
- если ответа не было, то `processed` сдвигается до последнего сообщения из обработанной пачки;
- во время reconciliation, если последнее актуальное сообщение в треде уже наше, можно выровнять `processed = seen` и не запускать новый handler.

Для каждого `thread_id` Layer 3 держит runtime state:

```rust
struct ThreadRuntimeState {
    status: ThreadStatus,
    pending_restart: bool,
    scheduled_run_id: u64,
    handle: Option<tokio::task::JoinHandle<()>>,
    last_event_at: chrono::DateTime<chrono::Utc>,
}
```

Хранится в памяти, например в `Arc<tokio::sync::Mutex<HashMap<String, ThreadRuntimeState>>>`.

### Инварианты

1. В каждый момент времени на тред максимум один активный `JoinHandle`.
2. Новое сообщение в треде всегда инвалидирует предыдущую обработку.
3. Обработчик всегда запускается на актуальном snapshot треда из Mattermost API, обогащённом metadata из Postgres.
4. После `Resolved` или `Stopped` новые handler run для треда не планируются.

## Debounce + abort модель

Выбранная модель:
- при новом сообщении в tracked thread текущая задача прерывается через `JoinHandle::abort()`;
- затем запускается debounce-таймер;
- если во время debounce пришли ещё сообщения, таймер пересоздаётся;
- после debounce handler запускается ровно один раз на полном свежем состоянии.

### Последовательность

1. Приходит `Posted` для tracked thread.
2. Сообщение сохраняется в БД.
3. Если по треду есть активный `handle`, выполняется `handle.abort()`.
4. Runtime state помечается как `pending_restart = true`.
5. Планируется запуск через `debounce_duration`.
6. Если до истечения debounce пришло ещё сообщение:
   - оно сохраняется в БД;
   - предыдущий scheduled restart считается устаревшим;
   - создаётся новый debounce window.
7. Когда debounce истёк:
   - Layer 3 запрашивает актуальный thread snapshot из Mattermost API по `root_id`;
   - обогащает его metadata из Postgres по `post_id`;
   - создаёт `ThreadContext`;
   - запускает `handler.handle(...)`.

### Почему именно `abort()`, а не cooperative cancellation

Это решение проще для первого этапа и соответствует ожиданию пользователя:
- новый входящий сигнал должен быстро отменять устаревшую обработку;
- потеря части compute допустима;
- важнее не отвечать на каждое промежуточное сообщение.

Недостаток: handler не может гарантированно сделать cleanup в момент отмены. Для первого этапа это принимается.

## Reconciliation при старте

После рестарта нельзя опираться только на WebSocket stream, потому что часть событий могла быть пропущена.

### Startup barrier и буферизация входящих событий

Во время startup reconciliation нельзя сразу обновлять `channel_checkpoints` по live входящим событиям: иначе можно перескочить сообщения, которые reconciliation ещё не успела догнать.

Источник событий для Layer 3 — не собственный WebSocket reader, а обычный `Plugin::process_event()` из Layer 2. `ThreadBot` принимает события из Layer 2 и складывает их во внутренний unbounded async-канал.

Поэтому startup делится на две фазы:

#### Фаза 1. `Bootstrapping`

1. `ThreadBot` запускает внутренний main loop task.
2. `process_event()` принимает события от Layer 2 и складывает их во внутренний unbounded канал.
3. Main loop читает эти события и во время bootstrapping складывает их во временный backlog, не пуская сразу в routing pipeline.
4. В этой фазе:
   - не запускаются handler run;
   - не обновляются `channel_checkpoints`;
   - не выполняются irreversible side effects.
5. Параллельно выполняется startup reconciliation.

#### Фаза 2. `Live`

После завершения reconciliation:
1. накопленный backlog входящих событий проигрывается в обычный routing pipeline;
2. применяется deduplication / idempotent upsert по `post_id`;
3. только после успешного прохождения события через ingestion pipeline обновляется `channel_checkpoint`;
4. новые события, приходящие из Layer 2 через `process_event()`, уже направляются напрямую в основной pipeline.

### Почему это важно

Именно такая startup barrier гарантирует, что checkpoint означает:
> "до этого места pipeline действительно догнал и учёл события",

а не:
> "мы когда-то увидели событие в сокете, но ещё не встроили его в локальное состояние".

### Требование к pipeline

Все пути поступления событий должны быть идемпотентными:
- replay backlog событий, пришедших через `process_event()`;
- channel catch-up;
- reconciliation известных тредов.

Сообщение, которое пришло из нескольких источников, должно безопасно сходиться через `upsert` по `post_id`.

При этом rerun handler-а нужен не просто когда появился новый post, а когда `seen > processed` по правилам thread progress.

### Замечание по памяти

Очередь входящих событий во время bootstrapping хранится в памяти. Для первой версии это допустимо. Если позже реальный поток окажется слишком большим, можно будет добавить bounded queue или spill-to-disk стратегию.

### Выбранная стратегия

При старте `ThreadBot` выполняет два шага.

#### Шаг 1. Catch-up по каналам
пше Для каждого отслеживаемого канала:
1. Берётся timestamp последнего успешно увиденного события/сообщения для этого канала.
2. Через Mattermost API запрашиваются сообщения канала после этого момента.
3. Для каждого найденного сообщения определяется его `thread_id`.
4. Эти сообщения прогоняются через тот же routing pipeline, что и обычные live events:
   - для новых тредов вызывается `should_track()`;
   - интересные треды создаются в БД;
   - неинтересные треды игнорируются.

Это нужно для того, чтобы после downtime обнаруживать не только уже известные активные треды, но и совсем новые треды, появившиеся пока бот был выключен.

#### Шаг 2. Реконсиляция уже известных активных тредов

1. Загружаются из `ThreadStore` все треды в статусах `New` и `Active`.
2. Для каждого треда запрашивается его полная актуальная история из Mattermost API по `root_id`.
3. Актуальные сообщения сопоставляются с локальными message references и metadata по `post_id`.
4. Обновляется `seen`-позиция треда (`last_seen_post_id`, `last_seen_post_at`).
5. Если последнее актуальное сообщение в треде уже наше, то можно выровнять `processed = seen` и не запускать новый handler.
6. Иначе, если `seen` ушёл вперёд относительно `processed`, для треда планируется новый handler run.

### Почему так

- catch-up по каналу нужен, чтобы не пропустить совершенно новые треды;
- thread API нужен, чтобы получать актуальный текст сообщений и видеть edits;
- Postgres хранит orchestration state и metadata, а Mattermost остаётся source of truth для контента;
- все persisted timestamp fields в БД хранятся как `TIMESTAMPTZ`.

## Реакции

### Архитектурное решение

Реакции обрабатываются через делегирование в `ThreadHandler`, а не жестко зашиты в Layer 3.

**Ключевые принципы**:
1. Layer 3 различает **control reactions** (на root post) и **feedback reactions** (на сообщения бота).
2. Control reactions обрабатываются через синхронный метод `ThreadHandler::on_control_reaction()`.
3. Handler получает **событие изменения реакции** (`ReactionChange`), а не только snapshot состояния.
4. Handler возвращает `Option<ThreadEffect>` для выполнения действий.
5. Feedback reactions сохраняются в БД только для сообщений бота (с флагом `is_bot_message`).

### Control reactions на root post

**Когда вызывается**:
- Только для реакций на root post треда.
- Только если тред в статусе `New` или `Active`.
- Для завершенных тредов (`Resolved`, `Stopped`) реакции игнорируются.

**Почему синхронный метод**:
- Нельзя делать долгую async работу внутри обработки реакции.
- Если нужна async работа, handler возвращает `ThreadEffect`, который Layer 3 применит.
- Это предотвращает блокировку event loop и упрощает reasoning о порядке выполнения.

**Почему передается `ThreadRecord`, а не `Thread`**:
- Сборка полного `Thread` snapshot дорогая (запрос к Mattermost API + join с БД).
- Для control reactions обычно достаточно metadata треда (`ThreadRecord`).
- Полный `Thread` собирается только если нужен (например, для `on_thread_closed`).

**Default implementation**:
```rust
fn on_control_reaction(
    &self,
    thread_record: &ThreadRecord,
    change: &ReactionChange,
) -> Option<ThreadEffect> {
    default_control_reactions(change)
}
```

Где `default_control_reactions`:
- `✅` (`white_check_mark`) на `ReactionAdded` → `ThreadEffect::MarkResolved`
- `🛑` (`stop_sign`) на `ReactionAdded` → `ThreadEffect::MarkStopped`
- Все остальное → `None`

### Feedback reactions на сообщения бота

**Определение**:
- Handler определяет, какие emoji считаются feedback через метод `is_feedback_reaction()`.
- Default: `thumbsup`, `thumbsdown`, `+1`, `-1`.

**Обработка**:
- Layer 3 проверяет: это наше сообщение (`is_bot_message = true`) + feedback reaction?
- Если да, сохраняет в `thread_reactions` для аналитики.
- Не вызывает handler, не меняет статус треда.
- Layer 4 может позже анализировать feedback через `ThreadStore`.

**Почему только на наши посты**:
- Экономия места в БД.
- Feedback имеет смысл только для ответов бота.
- Флаг `is_bot_message` выставляется при применении `ThreadEffect::Reply`.

### ReactionRemoved

**Обработка**:
- События `ReactionRemoved` сохраняются в `thread_reactions` с `action = Removed`.
- Передаются в `on_control_reaction()` для гибкости.
- Default implementation игнорирует `ReactionRemoved` (не меняет статус).
- Layer 4 может реализовать кастомную логику если нужно.

**Почему не влияет на статус по умолчанию**:
- Удаление `✅` не должно автоматически reopenить тред.
- Это упрощает модель и избегает race conditions.
- Если нужна такая логика, Layer 4 может ее реализовать.

## Интерфейсы

### `ThreadHandler`

```rust
#[async_trait::async_trait]
pub trait ThreadHandler: Send + Sync + 'static {
    /// Уникальный идентификатор обработчика.
    fn id(&self) -> &'static str;

    /// Решает, нужно ли начинать сопровождать тред.
    async fn should_track(
        &self,
        thread: &Thread,
        ctx: &ThreadContext,
    ) -> Result<bool, ThreadBotError>;

    /// Основная обработка треда.
    ///
    /// Вызывается после debounce и получает полный снимок треда.
    async fn handle(
        &self,
        thread: &Thread,
        ctx: &ThreadContext,
    ) -> Result<HandleResult, ThreadBotError>;

    /// Синхронная обработка control reactions на root post.
    ///
    /// Вызывается ТОЛЬКО если тред в статусе New или Active.
    /// Не может делать async работу - для этого возвращайте ThreadEffect.
    ///
    /// # Параметры
    /// - `thread_record`: легковесная запись треда из БД (без сообщений)
    /// - `change`: событие изменения реакции (кто, что, когда)
    ///
    /// # Возвращает
    /// - `Some(effect)` если нужно выполнить действие
    /// - `None` если реакция игнорируется
    fn on_control_reaction(
        &self,
        thread_record: &ThreadRecord,
        change: &ReactionChange,
    ) -> Option<ThreadEffect> {
        // Default: стандартная логика ✅/🛑
        default_control_reactions(change)
    }

    /// Определяет, является ли emoji feedback reaction.
    ///
    /// Feedback reactions сохраняются в БД только для сообщений бота
    /// (с флагом is_bot_message = true) для последующей аналитики.
    ///
    /// Default: 👍 и 👎
    fn is_feedback_reaction(&self, emoji_name: &str) -> bool {
        matches!(emoji_name, "thumbsup" | "thumbsdown" | "+1" | "-1")
    }

    /// Вызывается при штатном или принудительном закрытии треда.
    async fn on_thread_closed(
        &self,
        thread: &Thread,
        reason: ThreadCloseReason,
        ctx: &ThreadContext,
    ) -> Result<(), ThreadBotError> {
        Ok(())
    }
}
```

### `HandleOutcome`

Нужен явный результат, чтобы handler мог влиять на lifecycle треда.

```rust
pub enum HandleOutcome {
    /// Продолжать отслеживание.
    Continue,
    /// Обработчик решил, что тред больше не нужен.
    Stop,
    /// Обработчик считает тред решённым.
    Resolve,
}
```

### `ThreadEffect`

`ThreadEffect` описывает side effects, которые handler хочет выполнить по итогам обработки.

```rust
pub enum ThreadEffect {
    Noop,
    Reply {
        message: String,
        metadata: serde_json::Value,
    },
    UpdateMessage {
        post_id: String,
        message: Option<String>,
        metadata: Option<serde_json::Value>,
    },
    SetThreadMetadata {
        metadata: serde_json::Value,
    },
    MarkResolved,
    MarkStopped,
}
```

`UpdateMessage.metadata` и `SetThreadMetadata.metadata` на первом этапе работают по модели полной замены, без merge semantics.

### `HandleResult`

```rust
pub struct HandleResult {
    pub outcome: HandleOutcome,
    pub effects: Vec<ThreadEffect>,
}
```

Правила применения `effects` batch:
- эффекты применяются строго в том порядке, в котором были записаны handler-ом;
- mixed batch допустимы (`Reply + MarkResolved`, и т.п.);
- после первой ошибки дальнейшие эффекты не выполняются.

После успешного применения `effects` Layer 3 обновляет thread progress:
- если был успешно создан bot reply, `processed` сдвигается до этого сообщения;
- иначе `processed` сдвигается до последнего сообщения из обработанного snapshot.

Если применение batch завершилось ошибкой, дальнейшие эффекты не выполняются.
При этом ошибка считается обработанной на уровне orchestration, чтобы не зациклиться на одном и том же batch бесконечно; восстановление происходит через новые сообщения или следующую reconciliation.

### Почему effects лучше direct helpers

Layer 4 не должен выполнять network/database side effects прямо внутри `handle()`.
Вместо этого handler формирует список намерений, а Layer 3 применяет их после успешного завершения `handle()`.

Плюсы модели:
- проще для borrow checker;
- меньше coupling между handler и инфраструктурой;
- проще тестировать;
- abort безопаснее: если задача убита, эффекты просто не коммитятся;
- порядок применения side effects контролируется Layer 3;
- mixed batch (`Reply + MarkResolved`, и т.п.) обрабатывается естественно в порядке записи.

Важно: внешние side effects (Mattermost API) и локальный commit в Postgres не образуют единой атомарной транзакции. Если сообщение было успешно отправлено, но commit в БД не удался, это считается recoverable inconsistency и должно догоняться через новые события или reconciliation.

Это лучше, чем скрывать lifecycle decision в побочных эффектах.

### `ThreadCloseReason`

```rust
pub enum ThreadCloseReason {
    ResolvedByReaction,
    StoppedByReaction,
    ResolvedByHandler,
    StoppedByHandler,
}
```

### `ThreadStore`

`ThreadStore` скрывает конкретную БД за интерфейсом.

```rust
#[async_trait::async_trait]
pub trait ThreadStore: Send + Sync + 'static {
    async fn upsert_thread(&self, input: UpsertThread) -> Result<ThreadRecord, ThreadBotError>;

    async fn get_thread(&self, thread_id: &str) -> Result<Option<ThreadRecord>, ThreadBotError>;

    /// Найти тред по любому post_id в треде (легковесный запрос).
    /// Используется для routing реакций и сообщений к нужному треду.
    async fn get_thread_by_post(&self, post_id: &str) -> Result<Option<ThreadRecord>, ThreadBotError>;

    async fn list_threads_by_status(
        &self,
        statuses: &[ThreadStatus],
    ) -> Result<Vec<ThreadRecord>, ThreadBotError>;

    async fn update_thread_status(
        &self,
        thread_id: &str,
        status: ThreadStatus,
    ) -> Result<(), ThreadBotError>;

    async fn set_thread_metadata(
        &self,
        thread_id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError>;

    async fn update_thread_seen(
        &self,
        thread_id: &str,
        post_id: &str,
        seen_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ThreadBotError>;

    async fn update_thread_processed(
        &self,
        thread_id: &str,
        post_id: &str,
        processed_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ThreadBotError>;

    async fn upsert_message(&self, input: UpsertThreadMessage) -> Result<ThreadMessageRecord, ThreadBotError>;

    async fn get_message(&self, post_id: &str) -> Result<Option<ThreadMessageRecord>, ThreadBotError>;

    async fn list_thread_messages(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadMessageRecord>, ThreadBotError>;

    async fn set_message_metadata(
        &self,
        post_id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError>;

    async fn append_reaction(
        &self,
        input: AppendReaction,
    ) -> Result<(), ThreadBotError>;

    /// Получить список реакций треда для сборки Thread snapshot.
    async fn list_thread_reactions(&self, thread_id: &str) 
        -> Result<Vec<ThreadReaction>, ThreadBotError>;
}
```

### Почему один trait, а не несколько

На первом этапе удобнее один `ThreadStore`:
- проще wiring;
- проще моки;
- проще plugin API.

Если позже появится реальная боль, можно разбить на `ThreadStore` / `ThreadMessageStore` / `ThreadReactionStore`.

## Доменные структуры

### `ThreadRecord`

Легковесная запись треда из БД, без загрузки сообщений.
Используется для быстрых проверок и control reactions.

```rust
pub struct ThreadRecord {
    pub thread_id: String,
    pub root_post_id: String,
    pub channel_id: String,
    pub creator_user_id: String,  // автор root post
    pub status: ThreadStatus,
    pub metadata: serde_json::Value,
    pub last_seen_post_id: Option<String>,
    pub last_seen_post_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_processed_post_id: Option<String>,
    pub last_processed_post_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

### `Thread`

Полный snapshot треда для handler.
Дорогой в сборке - собирается из Mattermost API + БД.

```rust
pub struct Thread {
    pub info: ThreadInfo,
    pub messages: Vec<ThreadMessage>,
    pub reactions: Vec<ThreadReaction>,
}
```

### `ThreadInfo`

```rust
pub struct ThreadInfo {
    pub thread_id: String,
    pub root_post_id: String,
    pub channel_id: String,
    pub creator_user_id: String,
    pub status: ThreadStatus,
    pub metadata: serde_json::Value,
    pub last_seen_post_id: Option<String>,
    pub last_seen_post_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_processed_post_id: Option<String>,
    pub last_processed_post_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

`ThreadInfo` можно создать из `ThreadRecord` через `From` trait.

### `ThreadMessage`

`ThreadMessage` — это не запись из Postgres, а runtime snapshot сообщения, собранный из актуальных данных Mattermost API и локальной metadata.

```rust
pub struct ThreadMessage {
    pub post_id: String,
    pub thread_id: String,
    pub user_id: String,
    pub message: String,
    pub root_id: Option<String>,
    pub parent_post_id: Option<String>,
    pub props: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

### `ThreadReaction`

```rust
pub struct ThreadReaction {
    pub post_id: String,
    pub user_id: String,
    pub emoji_name: String,
    pub action: ReactionAction,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

```rust
pub enum ReactionAction {
    Added,
    Removed,
}
```

### `ReactionChange`

Событие изменения реакции, передается в `on_control_reaction()`.

```rust
pub struct ReactionChange {
    pub post_id: String,        // root_post_id
    pub user_id: String,        // кто добавил/удалил
    pub emoji_name: String,     // какую реакцию
    pub action: ReactionAction, // Added или Removed
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

## `ThreadContext`

`ThreadContext` в этой модели должен быть в основном read-only.

```rust
pub struct ThreadContext {
    pub config: std::sync::Arc<mattermost_api::apis::configuration::Configuration>,
    pub store: std::sync::Arc<dyn ThreadStore>,
    pub plugin_id: &'static str,
}
```

Его роль:
- давать handler-у доступ к конфигу и store для чтения;
- предоставлять read-only контекст обработки;
- не выполнять side effects напрямую.

Предпочтительная модель хранения metadata на первом этапе:
- `thread.metadata` — редкая high-level metadata всего треда;
- `thread_messages.metadata` — основное место для рабочих данных Layer 4.

Идея в том, что ответы бота и связанные с ними tool calls / annotations / classification можно привязывать к конкретному сообщению. Это упрощает модель и позволяет обновлять данные по `post_id`, а не мержить большой thread-level JSON blob.

Side effects (`reply`, `update message`, `mark stopped`, `mark resolved`) описываются через `ThreadEffect` и возвращаются из `handle()` в составе `HandleResult`.

## `ThreadBot` API

Предлагаемый builder-style API:

```rust
let plugin = ThreadBot::new(handler, store)
    .with_channel(channel_id)
    .with_debounce(Duration::from_secs(3))
    .build();
```

**Примечание**: Конфигурация control reactions (resolve/stop emoji) убрана из `ThreadBot`, так как теперь это ответственность `ThreadHandler::on_control_reaction()`.

Предполагается, что Mattermost API отдаёт имя emoji, а не unicode-символ. Поэтому Layer 3 оперирует именно `emoji_name` из API и не делает дополнительную нормализацию на первом этапе.

### Структура

```rust
pub struct ThreadBot<H> {
    handler: Arc<H>,
    store: Arc<dyn ThreadStore>,
    tracked_channels: std::collections::HashSet<String>,
    debounce: std::time::Duration,
    runtime: Arc<tokio::sync::Mutex<std::collections::HashMap<String, ThreadRuntimeState>>>,
}
```

**Примечание**: Поля `resolve_reaction` и `stop_reaction` удалены, так как обработка реакций делегирована в `ThreadHandler::on_control_reaction()`.

## Как `ThreadBot` реализует `Plugin`

### `id()`

Возвращает ID handler-а или configurable plugin id.

### `on_start()`

1. Инициализирует внутренние runtime-структуры.
2. Поднимает startup barrier в режиме `Bootstrapping`.
3. Запускает внутренний main loop, который читает события из внутреннего unbounded канала.
4. Выполняет channel catch-up и reconciliation известных активных тредов.
5. Проигрывает buffered events через основной routing pipeline.
6. Переключается в режим `Live`, после чего новые события из `process_event()` идут напрямую в pipeline.

Важно: auto-migration должна происходить при создании `PgThreadStore`, а не в `on_start()`, чтобы store оставался самодостаточным DB adapter-ом.

### `filter()`

Быстрый фильтр по типу события:
- пропускаем `Posted`;
- пропускаем `ReactionAdded` / `ReactionRemoved`;
- остальное игнорируем.

Дополнительно можно быстро фильтровать по `channel_id`, если оно есть в событии.

### `process_event()`

`process_event()` не должен выполнять тяжёлую orchestration-логику.

Его роль:
- принять `Arc<Event>` от Layer 2;
- быстро отправить его во внутренний `tokio::sync::mpsc::UnboundedSender`;
- завершиться как можно быстрее, чтобы не тормозить общий plugin pipeline Layer 2.

Дальнейшая маршрутизация выполняется внутренним main loop task.

### `on_shutdown()`

1. Abort всех активных задач.
2. Чистка runtime state.
3. Без изменения persisted statuses, кроме случаев явно зафиксированных ранее.

## Event routing

### `Posted`

Для каждого нового поста:

1. Определить `thread_id`:
   - если у поста есть `root_id`, использовать его;
   - иначе `thread_id = post.id`, т.е. новый root post создаёт новый тред-кандидат.
2. Собрать candidate snapshot из актуальных данных Mattermost, достаточный для `should_track()`.
3. Вызвать `handler.should_track()` только если тред ещё не tracked.
4. Если `false`, дальше ничего не делать:
   - тред не сохраняется в БД;
   - сообщения треда не копируются в store.
5. Если `true`:
   - создать thread record;
   - сохранить текущее сообщение reference в `thread_messages` с `is_bot_message = false`;
   - перевести тред в `Active`.
6. Если тред уже `Active`:
   - сохранить сообщение reference в `thread_messages` с `is_bot_message = false`;
   - abort текущего handler run;
   - запланировать новый запуск через debounce.

### `ReactionAdded`

```rust
// В ThreadBot::process_event()
EventType::ReactionAdded(ref reaction) => {
    // 1. Сохраняем в БД всегда
    self.store.append_reaction(AppendReaction {
        thread_id: None,  // будет заполнено через JOIN в БД
        post_id: reaction.post_id.clone(),
        user_id: reaction.user_id.clone(),
        emoji_name: reaction.emoji_name.clone(),
        action: ReactionAction::Added,
    }).await?;
    
    // 2. Находим тред по post_id (легковесный запрос)
    let Some(thread_record) = self.store
        .get_thread_by_post(&reaction.post_id)
        .await? 
    else {
        return; // Не tracked тред
    };
    
    // 3. Проверяем - это root post?
    if thread_record.root_post_id == reaction.post_id {
        // === Control reaction на root post ===
        
        // Только для активных тредов
        if !matches!(thread_record.status, ThreadStatus::New | ThreadStatus::Active) {
            return;
        }
        
        let change = ReactionChange {
            post_id: reaction.post_id.clone(),
            user_id: reaction.user_id.clone(),
            emoji_name: reaction.emoji_name.clone(),
            action: ReactionAction::Added,
            created_at: Utc::now(),
        };
        
        // Вызываем handler (синхронно!)
        if let Some(effect) = self.handler.on_control_reaction(&thread_record, &change) {
            // Применяем effect
            self.apply_effect(effect, &thread_record).await?;
            
            // Если Resolved/Stopped - abort и вызываем on_thread_closed
            if matches!(effect, ThreadEffect::MarkResolved | ThreadEffect::MarkStopped) {
                self.abort_handler(&thread_record.thread_id).await;
                
                // Теперь собираем полный Thread для on_thread_closed
                let thread = self.build_thread_snapshot(&thread_record).await?;
                let reason = match effect {
                    ThreadEffect::MarkResolved => ThreadCloseReason::ResolvedByReaction,
                    ThreadEffect::MarkStopped => ThreadCloseReason::StoppedByReaction,
                    _ => unreachable!(),
                };
                self.handler.on_thread_closed(&thread, reason, &self.ctx).await?;
            }
        }
    } else {
        // === Реакция на обычное сообщение ===
        
        // Проверяем: это наше сообщение + feedback reaction?
        if let Some(msg) = self.store.get_message(&reaction.post_id).await? {
            if msg.is_bot_message 
                && self.handler.is_feedback_reaction(&reaction.emoji_name) 
            {
                tracing::info!(
                    post_id = %reaction.post_id,
                    emoji = %reaction.emoji_name,
                    user_id = %reaction.user_id,
                    "Received feedback reaction on bot message"
                );
                // Уже сохранили в БД на шаге 1
            }
        }
    }
}
```

### `ReactionRemoved`

Аналогично `ReactionAdded`, но с `action: ReactionAction::Removed`.
Default implementation `on_control_reaction()` игнорирует `Removed` события.

## Схема БД

Ниже черновая схема. Названия и индексы ещё можно обсудить.

### Таблица `threads`

```sql
CREATE TABLE threads (
    thread_id TEXT PRIMARY KEY,
    root_post_id TEXT NOT NULL UNIQUE,
    channel_id TEXT NOT NULL,
    creator_user_id TEXT NOT NULL,
    status TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    last_seen_post_id TEXT,
    last_seen_post_at TIMESTAMPTZ,
    last_processed_post_id TEXT,
    last_processed_post_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_threads_channel_id ON threads(channel_id);
CREATE INDEX idx_threads_status ON threads(status);
CREATE INDEX idx_threads_updated_at ON threads(updated_at DESC);
CREATE INDEX idx_threads_metadata_gin ON threads USING GIN(metadata);
```

**Поле `creator_user_id`**:
- Автор root post треда.
- Добавлено для удобства в `on_control_reaction()` - можно проверять права без загрузки сообщений.

### `thread_messages`

Таблица хранит не текст сообщений как source of truth, а references и локальную metadata.

```sql
CREATE TABLE thread_messages (
    post_id TEXT PRIMARY KEY,
    thread_id TEXT NOT NULL REFERENCES threads(thread_id) ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    is_bot_message BOOLEAN NOT NULL DEFAULT FALSE,
    root_id TEXT,
    parent_post_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    post_created_at TIMESTAMPTZ NOT NULL,
    post_updated_at TIMESTAMPTZ,
    post_deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_thread_messages_thread_id_post_created_at
    ON thread_messages(thread_id, post_created_at ASC);

CREATE INDEX idx_thread_messages_user_id ON thread_messages(user_id);
CREATE INDEX idx_thread_messages_metadata_gin ON thread_messages USING GIN(metadata);
CREATE INDEX idx_thread_messages_is_bot ON thread_messages(is_bot_message) 
    WHERE is_bot_message = TRUE;
```

**Поле `is_bot_message`**:
- Выставляется в `true` при применении `ThreadEffect::Reply`.
- Используется для фильтрации feedback reactions - сохраняем только реакции на наши посты.
- Partial index для эффективного поиска сообщений бота.

### Таблица `thread_reactions`

```sql
CREATE TABLE thread_reactions (
    id BIGSERIAL PRIMARY KEY,
    thread_id TEXT NOT NULL REFERENCES threads(thread_id) ON DELETE CASCADE,
    post_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    emoji_name TEXT NOT NULL,
    action TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_thread_reactions_thread_id ON thread_reactions(thread_id);
CREATE INDEX idx_thread_reactions_post_id ON thread_reactions(post_id);
CREATE INDEX idx_thread_reactions_emoji_name ON thread_reactions(emoji_name);
```

### Таблица `channel_checkpoints`

```sql
CREATE TABLE channel_checkpoints (
    channel_id TEXT PRIMARY KEY,
    last_seen_post_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);
```

`channel_checkpoints` используются для channel catch-up после рестарта.

Они нужны именно для обнаружения новых сообщений и новых тредов на уровне канала.
Прогресс внутри конкретного треда хранится не здесь, а прямо в таблице `threads` через поля `last_seen_*` и `last_processed_*`.

Важное правило:
- checkpoint обновляется не в момент получения события через `process_event()`,
- а только после того, как событие прошло через основной ingestion pipeline.

Во время bootstrapping входящие события сначала накапливаются во внутреннем backlog, затем replay-ятся после reconciliation.

### Таблица `thread_runs`

Минимальный аудит запусков handler-а нужен для отладки debounce/abort модели.

```sql
CREATE TABLE thread_runs (
    id BIGSERIAL PRIMARY KEY,
    thread_id TEXT NOT NULL REFERENCES threads(thread_id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    trigger TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ,
    error TEXT
);

CREATE INDEX idx_thread_runs_thread_id_started_at
    ON thread_runs(thread_id, started_at DESC);
```

Где:
- `status`: `running`, `completed`, `aborted`, `failed`
- `trigger`: `new_message`, `reconcile`, `manual_retry`, и т.п.

Это намеренно минимальная таблица: без полного snapshot payload, только история жизненного цикла запусков.

## Замечания по схеме

1. В `thread_messages` храним references на сообщения и metadata, но не используем БД как source of truth для текста.
2. Актуальный thread snapshot для handler-а всегда собирается из Mattermost API по `root_id`, а затем обогащается данными из Postgres по `post_id`.
3. `metadata JSONB` у треда и сообщения — extension point для Layer 4.
4. Основную рабочую metadata рекомендуется хранить на уровне сообщений, особенно для ответов бота, tool calls и результатов обработки.
5. Реакции храним append-only, а не как “текущее состояние”, чтобы не потерять аудит.
6. `upsert_message(...)` и `update_message_metadata(...)` работают по модели update-by-post-id.
7. Для `thread.metadata` на первом этапе предпочтительна полная замена или редкие точечные обновления; основной mutable state рекомендуется хранить на уровне сообщений.

## `PgThreadStore`

`PgThreadStore` — production implementation поверх `sqlx::PgPool`.

```rust
pub struct PgThreadStore {
    pool: sqlx::PgPool,
}
```

### Инициализация

```rust
impl PgThreadStore {
    pub async fn new(pool: sqlx::PgPool) -> Result<Self, ThreadBotError> {
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }
}
```

### Почему миграции тут

Это держит DB concerns внутри DB adapter-а:
- `ThreadBot` не знает про sqlx;
- trait остаётся абстрактным;
- инициализация production store полностью encapsulated.

## `MockThreadStore`

Для unit tests нужен mock/in-memory store, например:

```rust
pub struct MockThreadStore {
    threads: tokio::sync::Mutex<HashMap<String, ThreadRecord>>,
    messages: tokio::sync::Mutex<HashMap<String, ThreadMessageRecord>>,
    reactions: tokio::sync::Mutex<Vec<ThreadReactionRecord>>,
}
```

Он должен реализовывать тот же `ThreadStore`.

## Runtime execution model

Практическая модель выполнения для первой версии:
- один main loop task управляет orchestration Layer 3;
- `Plugin::process_event()` принимает события от Layer 2 и отправляет их во внутренний `tokio::sync::mpsc::UnboundedSender`;
- main loop читает соответствующий `UnboundedReceiver`;
- во время `Bootstrapping` main loop временно накапливает уже вычитанные события во внутреннем backlog;
- для каждого активного треда может существовать отдельный subtask handler-а;
- main loop отвечает за abort/restart этих subtasks и применение `ThreadEffect`.

Таким образом:
- вход Layer 3 — это Plugin API Layer 2;
- async transport внутри Layer 3 — unbounded `tokio::mpsc`;
- backlog нужен только как локальная структура для startup barrier.

## Структура модуля `lib/thread-bot/src`

```text
lib/thread-bot/src/
├── lib.rs
├── error.rs
├── types.rs
├── handler.rs
├── store.rs
├── pg_store.rs
├── thread_bot.rs
├── reconciliation.rs
├── runtime.rs
└── reactions.rs
```

### Назначение файлов

- `lib.rs` — public reexports и модульная склейка.
- `error.rs` — `ThreadBotError`.
- `types.rs` — `Thread`, `ThreadInfo`, `ThreadRecord`, `ThreadMessage`, `ThreadStatus`, `ReactionChange`, DTO для store.
- `handler.rs` — `ThreadHandler`, `HandleOutcome`, `ThreadCloseReason`, `ThreadContext`.
- `store.rs` — `ThreadStore` trait.
- `pg_store.rs` — `PgThreadStore` и sqlx реализация.
- `thread_bot.rs` — `ThreadBot` struct, `Plugin` impl, startup barrier, `build_thread_snapshot()` и применение `ThreadEffect`.
- `reconciliation.rs` — channel catch-up, reconciliation активных тредов и replay backlog входящих событий.
- `runtime.rs` — runtime state и debounce scheduling utilities.
- `reactions.rs` — `default_control_reactions()` helper и reaction routing logic.

## Зависимости `Cargo.toml`

Черновой набор:

```toml
[dependencies]
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
futures-util = "0.3"
mattermost-api = { path = "../mattermost-api" }
mattermost-bot = { path = "../mattermost-bot" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "chrono", "json", "macros"] }
thiserror = "2"
tokio = { version = "1", features = ["rt", "rt-multi-thread", "sync", "time"] }
tracing = "0.1"

[dev-dependencies]
anyhow = "1"
testcontainers = "0.23"
uuid = { version = "1", features = ["v4"] }
```

Набор dev-dependencies ещё надо уточнить под текущую инфраструктуру проекта.

## Тестовая стратегия

### Unit tests

Покрывают логику без реального Mattermost и Postgres:
- `should_track()` flow;
- перевод статусов;
- event routing;
- debounce invalidation;
- abort старого handler run;
- `HandleOutcome::{Continue, Stop, Resolve}`;
- reconcile decision logic;
- mapping reactions → thread transitions.

Используем `MockThreadStore` и тестовый `ThreadHandler`.

### Integration tests

Покрывают реальную интеграцию:
- Mattermost + Postgres;
- `ThreadBot` подключён как plugin в `mattermost_bot::Bot`;
- root post создаёт tracked thread;
- в БД сохраняются message references и metadata с флагом `is_bot_message`;
- handler получает свежий snapshot треда из Mattermost API;
- при нескольких быстрых сообщениях старый handler abort-ится, срабатывает debounce;
- control reactions на root post вызывают `on_control_reaction()`;
- `✅` через default implementation переводит тред в `Resolved`;
- `🛑` через default implementation переводит тред в `Stopped`;
- feedback reactions на bot messages сохраняются в БД;
- feedback reactions на user messages игнорируются;
- startup barrier буферизует входящие события во время reconciliation;
- `channel_checkpoints` обновляются только после прохождения события через ingestion pipeline;
- restart catch-up по каналам находит новые треды и новые сообщения;
- reconciliation известных активных тредов догоняет изменения после рестарта.

### Почему нужны оба типа тестов

- unit tests дают быстрый feedback на сложную stateful логику;
- integration tests доказывают, что plugin реально работает с Mattermost и Postgres;
- DB abstraction остаётся проверяемой и не мешает реальной интеграции.

## Открытые вопросы

Уже решённые вопросы:

1. **Обработка реакций**: Делегирована в `ThreadHandler::on_control_reaction()` - синхронный метод, возвращающий `Option<ThreadEffect>`. Default implementation обрабатывает `✅`/`🛑`.

2. **Feedback reactions**: Определяются через `ThreadHandler::is_feedback_reaction()`. Сохраняются только для bot messages (с флагом `is_bot_message = true`).

3. **ReactionRemoved**: События сохраняются в БД и передаются в `on_control_reaction()`, но default implementation игнорирует их.

4. **ThreadRecord vs Thread**: Control reactions получают легковесный `ThreadRecord` без загрузки сообщений. Полный `Thread` собирается только когда нужен (через `build_thread_snapshot()`).

5. **Нормализация emoji**: Не делаем, используем `emoji_name` как приходит из Mattermost API.

6. **Хранение metadata**: Основной mutable state на уровне сообщений (`thread_messages.metadata`), а не в thread-level JSON blob.

7. **is_bot_message флаг**: Выставляется при применении `ThreadEffect::Reply`, используется для фильтрации feedback reactions.

## Предлагаемый план реализации

### Этап 1 — каркас crate
- создать `lib/thread-bot`;
- определить public types;
- определить `ThreadHandler` и `ThreadStore`;
- определить `ThreadBotError`.

### Этап 2 — Postgres store
- реализовать SQL migrations;
- реализовать `PgThreadStore`;
- покрыть store integration tests.

### Этап 3 — plugin runtime
- реализовать `ThreadBot` как `Plugin`;
- обработать `Posted`;
- реализовать debounce + abort;
- запускать handler с полным snapshot.

### Этап 4 — reactions + lifecycle
- реализовать `on_control_reaction()` в `ThreadHandler`;
- реализовать `default_control_reactions()` helper;
- добавить `is_feedback_reaction()` метод;
- реализовать routing реакций через `get_thread_by_post()`;
- добавить `Resolved` / `Stopped` transitions через `ThreadEffect`;
- сохранять feedback reactions только для bot messages;
- вызывать `on_thread_closed()`.

### Этап 5 — reconciliation
- реализовать startup reconciliation;
- покрыть restart integration tests.

## Решение

Предлагается принять следующую архитектуру Layer 3:
- `lib/thread-bot` реализуется как plugin для `mattermost-bot`;
- persistence живёт в `ThreadStore`, production-реализация — `PgThreadStore` на Postgres/sqlx;
- обработчик работает с полным thread snapshot;
- concurrency model: `abort + debounce + full rerun`;
- restart model: startup barrier + buffering входящих событий + channel catch-up + reconciliation всех `New`/`Active` threads при старте;
- **reaction model**: делегирование в `ThreadHandler::on_control_reaction()` для гибкости, с default implementation для стандартных случаев;
- **feedback reactions**: определяются через `is_feedback_reaction()`, сохраняются только для bot messages;
- **легковесный routing**: `ThreadRecord` для быстрых проверок, полный `Thread` только когда нужен;
- extension points для Layer 4 — через `ThreadHandler`, `ThreadContext`, `ThreadEffect` и JSONB metadata.

**Статус**: Готов к реализации после финального review.
