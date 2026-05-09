# RFC 0007: Idle actor teardown и lazy snapshot для `thread-bot`

**Статус**: Черновик
**Дата**: 2026-05-09
**Автор**: Обсуждение в парном программировании

## Контекст

`lib/thread-bot` сейчас совмещает две разные вещи:

- persisted thread state: `threads`, `thread_messages`, reactions, metadata,
   checkpoints и snapshot через Mattermost API;
- runtime execution: per-thread actor, debounce, interruption,
   `ThreadHandler::handle()` и применение `ThreadEffect`.

На практике actor нужен не как долгоживущее состояние треда, а как временный
синхронизатор обработки:

- сериализовать команды внутри одного Mattermost-треда;
- отменять устаревший handler run при новом сообщении;
- выдерживать debounce;
- последовательно применять effects.

После завершения обработки actor часто больше ничего полезного не хранит.
Состояние треда уже находится в БД. Если позже приходит новое сообщение, actor
можно создать заново.

Текущая долгоживущая модель создаёт проблемы для engineer/debug-тредов
`support-bot`:

- такие треды не имеют естественного close-сценария;
- после business close Layer 3 перестаёт маршрутизировать новые сообщения;
- debug/admin команды зависят от lifecycle actor-managed треда;
- появляется соблазн вводить отдельные passive threads и отдельный raw executor.

Более простая модель: все tracked треды хранятся одинаково, actor создаётся
только на время обработки и уничтожается после периода неактивности.

Дополнительно текущий handler API всегда передаёт L4 полный `Thread` snapshot.
Это может быть дорого: snapshot строится через Mattermost API и может быть не
нужен для простых команд, маршрутизации, debug actions или side effects. В L3
уже есть `ThreadContext::build_thread_snapshot(thread_id)`, поэтому полный
snapshot лучше сделать opt-in действием L4.

## Цели

1. Сделать actor короткоживущим executor/synchronizer, а не признаком “живого”
   треда.
2. Добавить idle teardown: actor завершается после configurable периода без
   pending work, running handler и новых команд.
3. Убрать необходимость в отдельном passive thread execution path.
4. Убрать L3-owned business lifecycle: `resolved`, `stopped`, `stale`,
   `waiting_for_user` и похожие состояния должны жить в metadata/таблицах L4,
   а не в `thread-bot`.
5. Сделать full `Thread` snapshot lazy: L4 получает lightweight invocation и
   сам вызывает `ctx.build_thread_snapshot(thread_id)`, если нужен полный
   transcript.
6. Расширить effects target-ами, чтобы Layer 4 мог явно писать в другой тред
   или обновлять metadata другого треда.
7. Сохранять созданные bot replies в DB сразу после успешного `create_post`,
   вместе с metadata, не ожидая WebSocket echo.
8. Перенести создание связанных engineer/debug-тредов из L4 handler code в
   L3 effect executor, чтобы cancellation handler-а не создавал дубликаты и не
   оставлял несшитые треды.
9. Сделать reconciliation пропорциональным количеству missed posts, а не
   общему количеству tracked threads.

## Не-цели

1. Не вводить separate `Passive`/`ActorManaged` execution mode.
2. Не делать generic event store поверх Mattermost.
3. Не менять Layer 2 `mattermost-bot`.
4. Не переносить LLM/tool workflow в `thread-bot`.
5. Не решать business lifecycle support-заявок на уровне Layer 3.

## Основная модель

### Actor как временный синхронизатор

Actor создаётся для `thread_id`, когда появляется работа:

- новое root сообщение, принятое handler decision;
- reply в уже известный persisted thread;
- reaction в known thread;
- `Wake` из `ThreadBotHandle`;
- reconciliation replay нашёл missed post.

Actor выполняет текущую работу:

1. сохраняет входящие сообщения/reactions в store;
2. выдерживает debounce;
3. вызывает handler;
4. применяет effects;
5. обновляет progress markers;
6. ждёт новые команды до `idle_timeout`;
7. если новых команд нет — завершает task и удаляется из `actors`.

Если после teardown приходит новое событие для того же `thread_id`, runtime
создаёт actor заново. Это штатный путь, не recovery path.

### Thread lifecycle как L4 state

`ThreadStatus` в Layer 3 удаляется. У persisted thread record остаются только
системные поля:

```rust
pub struct ThreadRecord {
   pub thread_id: String,
   pub root_post_id: String,
   pub channel_id: String,
   pub creator_user_id: String,
   pub thread_kind: Option<String>,
   pub metadata: serde_json::Value,
   // progress markers, timestamps...
}
```

Layer 3 не знает, открыт тред, закрыт, stale или ждёт пользователя. Он знает
только, что Mattermost thread tracked, где его root post, какой у него
`thread_kind`, и какие сообщения/reactions уже сохранены и обработаны.

L4 сам хранит lifecycle state:

- support-bot может хранить `support_bot.status = "finished"` или `"stale"` в
   thread metadata;
- другой handler может переоткрыть тред;
- cron может искать stale треды и пинговать пользователя или инженера;
- другой L4 может держать свой enum/таблицу состояний без изменений в
   `thread-bot`.

Layer 3 отвечает за доставку persisted thread context и serialization, а не за
business policy “можно ли ещё обрабатывать этот тред”.

## Handler API

### Tracking decision

should_track() -> bool нужно заменить на более выразительное решение.

Пример формы:

```rust
pub enum TrackDecision {
   Ignore,
   Track {
         thread_kind: Option<String>,
         metadata: serde_json::Value,
   },
}
```

`thread_kind` — свободная строка от L4 для queries, metrics и link resolution,
а не actor lifecycle mode и не закрытый enum. Layer 3 не валидирует значения.
Например:

- support_user;
- support_engineer;
- debug;
- application-specific значения.

metadata позволяет сразу создать thread record с нужным initial state.

### Lazy handler invocation

Текущий API:

```rust
async fn handle(
   &self,
   thread: &Thread,
   ctx: &ThreadContext,
) -> Result<Vec<ThreadEffect>, ThreadBotError>;
```

Новая идея: передавать lightweight invocation вместо полного snapshot.

Пример формы:

```rust
pub struct ThreadInvocation {
   pub thread: ThreadRecord,
   pub trigger: ThreadTrigger,
}

pub enum ThreadTrigger {
   NewMessage {
         post_id: String,
         user_id: String,
         message: Option<String>,
   },
   Reaction {
         change: ReactionChange,
   },
   Wake,
}
```

Минимальные требования к invocation:

- содержит thread_id, root_post_id, channel_id, thread_kind, metadata и
   progress markers;
- содержит trigger, чтобы L4 мог быстро понять причину вызова;
- не требует Mattermost API call для построения полного transcript.

Если L4 нужен полный тред, он вызывает:

```rust
let thread = ctx.build_thread_snapshot(&invocation.thread.thread_id).await?;
```

Это сохраняет текущую возможность, но делает её opt-in.

### Reconciliation

Reconciliation не должен идти по списку tracked threads. После перехода к
короткоживущим actors и L4-owned lifecycle такой scan становится линейно
дороже со временем: старые закрытые/stale/debug треды остаются tracked, но не
должны каждый reconnect создавать actor и строить snapshot.

Целевая модель:

- runtime snapshot-ит channel checkpoints и временно блокирует их advance;
- для каждого checkpointed channel runtime читает posts after checkpoint;
- каждый missed post прогоняется через тот же routing path, что и WebSocket
   `Posted` event;
- root post может создать новый tracked thread через tracking decision;
- reply post ищет persisted thread по root id, сохраняется в DB и создаёт actor
   только для этого конкретного треда;
- bot-authored missed posts сохраняются, но не триггерят handler;
- external missed posts доставляются в handler как `ThreadTrigger::NewMessage`,
   а не как отдельный `Reconcile` режим;
- L4 строит full snapshot только если ему нужен transcript.

После scan-а channel помечается reconciled, и обычные WebSocket events снова
могут advance-ить checkpoint.

`max_reconcile_window` в этой модели не закрывает треды. Если downtime больше
окна, runtime может skip-нуть backlog для канала и передвинуть checkpoint к
текущему моменту, но это policy потери missed posts, а не lifecycle transition.

Missed reactions не восстанавливаются через full scan всех старых тредов. Если
L4 нужна надёжная обработка reaction-based команд во время downtime, это должен
быть отдельный bounded/app-specific reconciliation path. Базовый Layer 3
reconciliation остаётся post/checkpoint-driven.

## Effects

Actor остаётся единственным executor-ом ThreadEffect.

Не нужен отдельный execute_raw_post_effects: engineer/debug commands проходят
через тот же actor/effect path, просто actor уничтожается после idle timeout.

### Reply target

ThreadEffect::Reply получает явный target:

```rust
pub enum ThreadTarget {
   CurrentThread,
   Thread {
         channel_id: String,
         root_post_id: String,
   },
   LinkedThread {
         relation: String,
   },
}
```

Возможная форма:

```rust
ThreadEffect::Reply {
   target: ThreadTarget,
   message: String,
   metadata: serde_json::Value,
}
```

Default semantics для старых call sites: CurrentThread.

`ThreadTarget::LinkedThread` резолвится только внутри effect executor-а. Если
связанный тред уже существует, Reply пишется туда. Если связи нет, executor
должен получить её из предшествующего `EnsureLinkedThread` в том же batch или
вернуть ошибку эффекта. L4 не должен самостоятельно создавать Mattermost post
для engineer/debug root thread через raw `mattermost_api`.

После успешного posts_api::create_post executor сразу сохраняет созданный post
в thread_messages:

- post_id = created.id;
- thread_id = target root thread;
- is_bot_message = true;
- metadata = effect.metadata;
- timestamps из Mattermost response.

WebSocket echo остаётся idempotent update, а не единственный источник записи bot
message.

### Linked thread effect

Для создания engineer/debug-треда нужен compound effect, а не callback
`on_effect_applied(effect_result) -> Vec<Effects>`.

Целевая форма:

```rust
pub enum ThreadEffect {
   EnsureLinkedThread {
         relation: String,
         target_channel_id: String,
         root_message: String,
         root_metadata: serde_json::Value,
         target_thread_kind: Option<String>,
   },
   Reply {
         target: ThreadTarget,
         message: String,
         metadata: serde_json::Value,
   },
}
```

`relation` — application-owned имя связи, например `support_engineer`.

Семантика `EnsureLinkedThread`:

1. Executor ищет существующую связь `(source_thread_id, relation)`.
2. Если связь есть — effect является no-op.
3. Если связи нет — executor создаёт root post в `target_channel_id`.
4. После успешного create_post executor в одной критической секции:
   - upsert-ит target thread record для созданного root post;
   - записывает двустороннюю связь source -> target;
   - выставляет `thread_kind = target_thread_kind` для target thread, если
      задано;
   - сохраняет root post в `thread_messages` с `root_metadata`.
5. Следующие effects в том же batch могут использовать
   `ThreadTarget::LinkedThread { relation }`.

Это решает проблему отмены handler run: пока handler вычисляет effects, он не
создаёт внешних Mattermost post-ов. После того как batch передан actor-у,
effects применяются последовательно как L3-owned critical section. Новый input
может отменить следующий handler run, но не должен прерывать середину
применения одного effect batch.

Связь не дублируется в `threads.metadata`. Source of truth — нормализованная
таблица `thread_links` или эквивалентные typed columns в Postgres. Metadata
остаётся application-owned JSON для состояния L4, а не местом для системных
foreign key между тредами.

Для чтения связи L4 получает typed API, например:

```rust
ctx.get_thread_link(thread_id, "support_engineer").await?;
ctx.get_source_thread_link(engineer_thread_id, "support_engineer").await?;
```

Это особенно важно для engineer/debug команд: source user thread должен
находиться через persisted link, а не через root post props/metadata. Props у
Mattermost post можно оставить как удобный human/debug hint, но не как основной
индекс.

Важно: `EnsureLinkedThread` даёт at-most-one создание в рамках живого actor-а и
нормального retry после handler cancellation. Он не обещает strict exactly-once
при crash между Mattermost `create_post` и записью связи в DB. Если это станет
важно, можно расширить модель outbox/idempotency marker-ом, но для ранней
альфы это лишняя сложность.

### Metadata target

SetThreadMetadata тоже получает target:

```rust
ThreadEffect::SetThreadMetadata {
   target: ThreadMetadataTarget,
   metadata: serde_json::Value,
}
```

Где target может быть:

```rust
pub enum ThreadMetadataTarget {
   CurrentThread,
   ThreadId(String),
   RootPostId(String),
}

impl Default for ThreadMetadataTarget {
   fn default() -> Self {
      Self::CurrentThread
   }
}
```

Default semantics: если target не указан на call site или используется helper
constructor, применяется `CurrentThread`. Это сохраняет старое поведение для
обычных `SetThreadMetadata`.

`ThreadId` и `RootPostId` позволяют L4 обновлять metadata другого треда, если
это действительно application state. Для системной связи source/engineer нужно
использовать `EnsureLinkedThread` и `thread_links`, а не metadata patch.

SetMessageMetadata уже таргетируется по post_id; его можно оставить как есть.

Для двусторонних source/engineer связей предпочтителен `EnsureLinkedThread`, а
не пара `Reply + SetThreadMetadata`: связь должна быть одним доменным effect-ом,
иначе L4 снова будет знать слишком много о порядке применения и компенсациях.

### No close effects in Layer 3

`MarkResolved`, `MarkStopped` и `on_thread_closed` удаляются из Layer 3 API.

Если L4 хочет “закрыть” тред, он должен записать собственное состояние через
`SetThreadMetadata` или свой store. Если после этого приходят новые сообщения,
Layer 3 всё равно доставляет invocation в L4, а L4 решает:

- проигнорировать сообщение;
- ответить “тред закрыт”;
- переоткрыть workflow;
- перевести состояние в `stale`, `waiting_for_user`, `escalated` и т.п.

## Runtime routing

### New root post

1. Runtime строит lightweight initial context.
2. Вызывает tracking decision.
3. Если Ignore — ничего не сохраняет.
4. Если Track:
   - upsert thread record;
   - сохраняет root message;
   - upsert channel checkpoint;
   - ensure/spawn actor;
   - отправляет NewMessage.

### Reply post

1. Runtime ищет thread record по root id.
2. Если record найден:
   - сохраняет message;
   - advance checkpoint;
   - если message от bot — не триггерит handler;
   - иначе ensure/spawn actor и отправляет NewMessage.
3. Business lifecycle state не фильтрует routing.

### Reaction

1. Runtime ищет thread record by post.
2. Сохраняет reaction, если она относится к known thread.
3. Вызывает reaction hooks или доставляет reaction trigger в L4.
4. Если есть effect или trigger для L4 — ensure/spawn actor.
5. Business lifecycle state не фильтрует routing.

### Actor idle timeout

ThreadBotConfig получает настройку:

```rust
pub struct ThreadBotConfig {
   pub debounce: Duration,
   pub channel_buffer: usize,
   pub max_reconcile_window: Option<Duration>,
   pub actor_idle_timeout: Duration,
}
```

Default: `Duration::ZERO`.

`actor_idle_timeout = 0` означает “завершить actor сразу после того, как нет
pending work, running handler и reschedule”. Это не меняет наблюдаемую
семантику: actor является executor/synchronizer на время работы, а не cache
business state. Ненулевой timeout — только performance optimization, чтобы
переиспользовать actor при burst сообщений и не пересоздавать task на каждый
короткий reply/reaction.

Actor не завершается по idle timeout, если:

- есть has_pending;
- handler running;
- reschedule requested;
- команда пришла до истечения timeout.

## Support-bot

### User support thread

support-bot получает invocation и сам решает, нужен ли full snapshot.

Для обычного user flow snapshot нужен, потому что LLM context строится из
transcript:

```rust
let thread = ctx.build_thread_snapshot(&invocation.thread.thread_id).await?;
```

Если L4 metadata говорит, что support-тред `finished`/`stale`/`waiting`, support-bot может:

- не звать LLM;
- ответить коротким workflow-сообщением;
- переоткрыть тред через metadata effect;
- выполнить другой policy.

Это решение остается внутри L4.

### Engineer/debug thread

Engineer channel больше не требует separate passive path.

Tracking decision может создать обычный persisted thread с thread_kind
support_engineer.

Debug command handling может работать по lightweight invocation:

- для !support state snapshot engineer-треда не нужен;
- для !support debug-report source user thread находится reverse lookup-ом в
   `thread_links` по текущему engineer thread id и `relation`;
- Mattermost full snapshot engineer-треда не нужен;
- основной user-thread report строится через
   ctx.build_thread_snapshot(source_thread_id).

Actor для engineer-треда живёт только во время команды и idle window, затем
уничтожается.

Создание engineer thread переносится из `MattermostSupportNotifier` в effects:

```rust
let effects = vec![
   ThreadEffect::EnsureLinkedThread {
         relation: "support_engineer".to_string(),
         target_channel_id: engineer_channel_id,
         root_message,
         root_metadata,
         target_thread_kind: Some("support_engineer".to_string()),
   },
   ThreadEffect::Reply {
         target: ThreadTarget::LinkedThread {
            relation: "support_engineer".to_string(),
         },
         message: mirrored_user_message,
         metadata,
   },
];
```

Support-bot state может по-прежнему хранить `EngineerThreadRef` для удобства
debug/report UI, но source of truth для связи должен быть L3 store. Handler не
зависит от результата create_post и не получает `created.id` напрямую.

## Миграция и совместимость

Проект ещё не задеплоен, поэтому отдельные forward migrations не нужны:
правим существующую initial migration и тестовые fixtures.

1. Добавить `thread_kind` в `threads`, nullable или default.
2. Добавить таблицу связей тредов, например:

   - `source_thread_id`;
   - `relation`;
   - `target_thread_id`;
   - `target_channel_id`;
   - `target_root_post_id`;
   - `created_at`, `updated_at`;
   - unique `(source_thread_id, relation)`;
   - unique `(target_thread_id)`, если один target не должен быть привязан к
      нескольким source.

3. Добавить `actor_idle_timeout` в config builder с default `Duration::ZERO`.
4. Обновить ThreadHandler API на tracking decision и lazy invocation.
5. Обновить support-bot под lazy snapshot и linked-thread effects.
6. Удалить actor-level `Reconcile` command/path и thread-list scan на
   reconnect.
7. Удалить отдельный raw-post executor и fake engineer snapshot.
8. Обновить tests, examples и README/RFC references.

Отдельный compatibility adapter для старого `handle(&Thread, ...)` не нужен:
API можно перевести атомарно в рамках текущей альфы.

## Тест-план

### thread-bot

- actor создаётся при новом сообщении;
- actor завершается после idle timeout;
- новое сообщение после teardown создаёт actor заново;
- idle timeout не завершает running handler;
- Reschedule удерживает actor до завершения work;
- reconnect не вызывает list-all/list-active tracked threads;
- reconciliation сканирует checkpointed channels и обрабатывает только missed
   posts;
- missed reply в known thread создаёт actor только для этого thread;
- missed root post проходит tracking decision и создаёт thread при Track;
- bot-authored missed posts сохраняются, но не запускают handler;
- `max_reconcile_window` skip-ает слишком старый backlog без lifecycle updates;
- сообщения после L4-owned close/stale state доходят до handler;
- L3 не содержит MarkResolved/MarkStopped/on_thread_closed business lifecycle;
- Reply с default target пишет в текущий thread;
- Reply с explicit target пишет в указанный thread;
- EnsureLinkedThread создаёт target root, target thread record и thread link;
- повторный EnsureLinkedThread для того же `(source, relation)` не создаёт
   второй Mattermost root post;
- Reply в LinkedThread пишет в уже созданный target;
- created reply сразу сохраняется в thread_messages с metadata;
- WebSocket echo created reply идемпотентно обновляет ту же запись;
- SetThreadMetadata обновляет current и explicit target;
- lazy invocation path не вызывает Mattermost thread API без запроса L4;
- ctx.build_thread_snapshot продолжает строить полный transcript.

### support-bot

- user LLM flow явно строит full snapshot;
- closed/stale user support thread не вызывает LLM, если такова policy L4;
- engineer !support state работает без full snapshot;
- engineer !support debug-report экспортирует source user thread;
- user flow создаёт engineer thread через EnsureLinkedThread, а не через raw
   Mattermost API внутри handler-а;
- engineer actor исчезает после idle timeout;
- следующая engineer command после teardown снова обрабатывается;
- bot-authored engineer replies сохраняются, но не запускают debug command loop.

## Закрытые решения

1. Поле классификации называется `thread_kind`.
2. `thread_kind` — свободная строка, owned by L4. Layer 3 хранит и индексирует
   её, но не валидирует и не интерпретирует.
3. Business lifecycle удаляется из Layer 3: нет `ThreadStatus`,
   `MarkResolved`, `MarkStopped` и `on_thread_closed`.
4. Default `actor_idle_timeout` — `Duration::ZERO`; ненулевое значение является
   только оптимизацией переиспользования actor-а.
