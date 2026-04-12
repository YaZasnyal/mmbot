# RFC 0003: Cron-доступ для ThreadHandler плагинов

**Статус**: Черновик  
**Дата**: 2026-04-12  
**Автор**: Обсуждение в парном программировании

## Контекст

Layer 3 (`lib/thread-bot`) реализован согласно [RFC 0002](0002-thread-bot-architecture.md) и предоставляет:

- `ThreadHandler` trait для обработки тредов;
- per-thread actor model с debounce и handler interruption;
- persistence через `ThreadStore` / `PgThreadStore`;
- reconciliation при (пере)подключении;
- управление lifecycle через `ThreadEffect` и control reactions.

Layer 2 (`lib/mattermost-bot`) уже имеет cron-инфраструктуру: `Plugin::setup_cron()` позволяет плагинам регистрировать периодические задачи через `cron_tab::Cron<Utc>`. `ThreadBotPlugin<H>` реализует `Plugin`, но не переопределяет `setup_cron` — используется default no-op.

Проблема: `ThreadHandler` не имеет доступа к cron-инфраструктуре и не может выполнять периодические задачи. Вся логика handler-а событийная — реагирует только на WebSocket events.

## Цели

1. Дать `ThreadHandler` возможность регистрировать cron-задачи.
2. Предоставить из cron-колбэков контролируемый доступ к данным тредов (чтение) и к actor-командам (wake, close).
3. Сохранить чёткие границы абстракций: cron-**логика** живёт в Layer 4 (конкретный handler), thread-bot предоставляет **механизм и доступ**.
4. Не ломать существующий API `ThreadHandler`.

## Не-цели

1. Не реализовывать конкретные cron-сценарии (пинг дежурного, авто-закрытие) — это ответственность Layer 4.
2. Не делать per-thread таймеры или inactivity timeouts внутри actor loop — это отдельная фича, если понадобится.
3. Не менять существующую cron-инфраструктуру Layer 2 (`cron_tab`, `Plugin::setup_cron`).
4. Не давать возможность спавнить акторов из cron (см. [обоснование](#почему-без-spawn-on-demand)).

## Проблема и сценарии использования

### Сценарий 1: Пинг дежурного для зависших тредов

Каждые 5 минут проверять активные треды. Если тред не получал новых сообщений больше часа, отправить напоминание в канал или mention дежурного.

Требуется: чтение списка активных тредов, доступ к `updated_at`, возможность отправить сообщение через Mattermost API.

### Сценарий 2: Авто-закрытие abandoned тредов

Раз в день проверять активные треды. Если тред не обновлялся 72 часа, закрыть его (пометить как `Stopped`). При закрытии handler должен получить `on_thread_closed` с соответствующей причиной, чтобы отправить прощальное сообщение.

Требуется: чтен��е списка тредов, закрытие через actor (чтобы отработал полный lifecycle), новый `ThreadCloseReason` для различения автоматического закрытия.

### Сценарий 3: Генерация отчётов

Раз в неделю собрать статистику: сколько тредов открыто, сколько закрыто за неделю, среднее время обработки. Для более глубокого анализа может потребоваться полный snapshot треда (сообщения + реакции).

Требуется: чтение тредов по статусу, доступ к `build_thread_snapshot()`, возможность отправить сообщение через Mattermost API.

### Сценарий 4: Повторная обработка стухшего треда

Тред активен, handler ответил и ждёт реакции пользователя. Прошло 30 минут — handler хочет отправить follow-up. Cron-задача может "разбудить" актор треда, вызвав повторный handler run.

Требуется: возможность отправить команду `Wake` существующему актору.

## Текущая архитектура (что есть)

### `Plugin::setup_cron` в Layer 2

```rust
// lib/mattermost-bot/src/plugin.rs

fn setup_cron(
    self: Arc<Self>,
    _scheduler: &mut cron_tab::Cron<chrono::Utc>,
    _config: Arc<Configuration>,
) {
    // Default: no cron jobs
}
```

Вызывается один раз при старте бота. Колбэки cron — синхронные замыкания (`Fn() + Send + 'static`). Для async работы используется `tokio::runtime::Handle::current().block_on(...)`.

### `ThreadCommand` в actor.rs

```rust
pub(crate) enum ThreadCommand {
    NewMessage { post: models::Post },
    ControlReaction { effect: ThreadEffect, change: ReactionChange },
    Reconcile,
    Shutdown,
}
```

### `ThreadCloseReason` в handler.rs

```rust
pub enum ThreadCloseReason {
    ResolvedByReaction,
    StoppedByReaction,
    ResolvedByHandler,
    StoppedByHandler,
}
```

### `ThreadBotPlugin<H>` в runtime.rs

```rust
pub struct ThreadBotPlugin<H: ThreadHandler> {
    handler: Arc<H>,
    store: Arc<dyn ThreadStore>,
    config: ThreadBotConfig,
    actors: Arc<Mutex<HashMap<String, mpsc::Sender<ThreadCommand>>>>,
    bot_user_id: Arc<RwLock<Option<String>>>,
}
```

Реализует `Plugin` для Layer 2. В текущей реализации `setup_cron` не переопределён — используется default no-op.

### `build_thread_snapshot` в actor.rs

```rust
pub(crate) async fn build_thread_snapshot(
    thread_id: &str,
    store: &dyn ThreadStore,
    mm_config: &Configuration,
) -> Result<Thread, ThreadBotError>
```

Строит полный `Thread` snapshot: запрашивает посты через Mattermost API, реакции из БД, расставляет `is_new` флаги по `last_processed_post_at`. Не требует `Arc<H>` — работает только со store и MM API.

## Решение

### Обзор

Добавляем три компонента:

1. **`ThreadBotHandle`** — Clone-friendly фасад для доступа к данным и командам thread-bot из cron-колбэков.
2. **`ThreadHandler::setup_cron()`** — default no-op метод на trait, аналогичный `Plugin::setup_cron`.
3. **Новые `ThreadCommand` варианты** — `Wake` и `Close` для управления акторами из cron.

```text
┌──────────────────────────────────────────────────────────────┐
│  Layer 4: SupportHandler (impl ThreadHandler)                │
│                                                              │
│  fn setup_cron(self, scheduler, handle) {                    │
│      scheduler.add_fn("...", move || {                       │
│          handle.list_threads(...)      // read from store    │
│          handle.wake_thread(...)       // command to actor   │
│          handle.stop_thread(...)       // command to actor   │
│          handle.config()               // Mattermost API     │
│          handle.build_thread_snapshot() // full snapshot     │
│      });                                                     │
│  }                                                           │
└──────────────────────┬───────────────────────────────────────┘
                       │ uses
┌──────────────────────▼───────────────────────────────────────┐
│  Layer 3: ThreadBotHandle                                    │
│                                                              │
│  ┌─────────────┐  ┌──────────────────┐  ┌────────────────┐   │
│  │ ThreadStore │  │ actors map       │  │ Configuration  │   │
│  │ (read/meta) │  │ (Wake, Close)    │  │ (MM API calls) │   │
│  └─────────────┘  └──────────────────┘  └────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

### `ThreadBotHandle`

Фасад, через который cron-колбэки взаимодействуют с thread-bot. Все поля приватные, доступ через методы.

```rust
/// Фасад для доступа к thread-bot из cron-задач и внешних триггеров.
///
/// Предоставляет read-доступ к данным тредов через `ThreadStore`
/// и контролируемый write-доступ через actor-команды.
///
/// `Clone`-friendly — можно захватывать в cron-замыканиях.
#[derive(Clone)]
pub struct ThreadBotHandle {
    store: Arc<dyn ThreadStore>,
    config: Arc<Configuration>,
    actors: Arc<Mutex<HashMap<String, mpsc::Sender<ThreadCommand>>>>,
    bot_user_id: Arc<RwLock<Option<String>>>,
}
```

#### Read-операции (делегируются в `ThreadStore`)

```rust
impl ThreadBotHandle {
    /// Список тредов по статусу.
    ///
    /// Пример: все активные треды для проверки на timeout.
    pub async fn list_threads(
        &self,
        statuses: &[ThreadStatus],
        // time interval
    ) -> Result<Vec<ThreadRecord>, ThreadBotError> {
        self.store.list_threads_by_status(statuses).await
    }

    /// Получить запись треда по thread_id.
    ///
    /// Возвращает легковесный `ThreadRecord` без загрузки сообщений.
    pub async fn get_thread(
        &self,
        thread_id: &str,
    ) -> Result<Option<ThreadRecord>, ThreadBotError> {
        self.store.get_thread(thread_id).await
    }

    /// Получить записи сообщений треда из БД.
    ///
    /// Возвращает `ThreadMessageRecord` (references + metadata из Postgres),
    /// НЕ полные сообщения из Mattermost API. Для полных сообщений
    /// используйте `build_thread_snapshot()`.
    pub async fn get_thread_messages(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadMessageRecord>, ThreadBotError> {
        self.store.list_thread_messages(thread_id).await
    }

    /// Получить реакции треда из БД.
    pub async fn get_thread_reactions(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadReaction>, ThreadBotError> {
        self.store.list_thread_reactions(thread_id).await
    }
}
```

#### Snapshot (Mattermost API + БД)

```rust
impl ThreadBotHandle {
    /// Собрать полный snapshot треда.
    ///
    /// **Дорогая операция**: делает HTTP-запросы к Mattermost API
    /// для получения актуальных сообщений. Реакции берутся из БД.
    /// Каждое сообщение и реакция помечены `is_new` относительно
    /// `last_processed_post_at`.
    ///
    /// Используйте для генерации отчётов. Для простых проверок
    /// (timeout, статус) достаточно `get_thread()`.
    pub async fn build_thread_snapshot(
        &self,
        thread_id: &str,
    ) -> Result<Thread, ThreadBotError> {
        // Делегирует в actor::build_thread_snapshot()
        crate::actor::build_thread_snapshot(thread_id, &*self.store, &self.config).await
    }
}
```

Функция `build_thread_snapshot` в `actor.rs` меняет видимость с `pub(crate)` на `pub`.

#### Actor-команды

```rust
impl ThreadBotHandle {
    /// Разбудить актор треда — вызвать повторный handler run.
    ///
    /// НЕ отменяет текущий handler, если он уже выполняется.
    /// Устанавливает `has_pending = true` и сбрасывает debounce,
    /// так что handler запустится после завершения текущего.
    ///
    /// Возвращает `Ok(true)` если команда отправлена актору,
    /// `Ok(false)` если актор не найден (тред закрыт или бот
    /// только перезапустился и ещё не прошёл reconciliation).
    pub async fn wake_thread(&self, thread_id: &str) -> Result<bool, ThreadBotError> {
        let actors = self.actors.lock().await;
        if let Some(tx) = actors.get(thread_id) {
            if !tx.is_closed() {
                let _ = tx.send(ThreadCommand::Wake).await;
                return Ok(true);
            }
        }
        tracing::debug!(
            thread_id = %thread_id,
            "wake_thread: no actor found, thread may be closed or not yet reconciled"
        );
        Ok(false)
    }

    /// Закрыть тред как Resolved через актор.
    ///
    /// Если актор существует: отправляет `Close` команду. Актор отменит
    /// текущий handler, обновит статус в БД, вызовет `on_thread_closed`
    /// с причиной `ResolvedExternally`, и завершится.
    ///
    /// Если актора нет: прямое обновление статуса в БД.
    /// `on_thread_closed` в этом случае **не вызывается** (edge case,
    /// см. обоснование в секции "Почему без spawn-on-demand").
    pub async fn resolve_thread(&self, thread_id: &str) -> Result<(), ThreadBotError> {
        self.close_thread_impl(thread_id, ThreadCloseReason::ResolvedExternally).await
    }

    /// Закрыть тред как Stopped через актор.
    ///
    /// Семантика аналогична `resolve_thread`, но с `ThreadStatus::Stopped`
    /// и причиной `StoppedExternally`.
    pub async fn stop_thread(&self, thread_id: &str) -> Result<(), ThreadBotError> {
        self.close_thread_impl(thread_id, ThreadCloseReason::StoppedExternally).await
    }

    /// Внутренняя реализация закрытия.
    async fn close_thread_impl(
        &self,
        thread_id: &str,
        reason: ThreadCloseReason,
    ) -> Result<(), ThreadBotError> {
        let actors = self.actors.lock().await;
        if let Some(tx) = actors.get(thread_id) {
            if !tx.is_closed() {
                let _ = tx.send(ThreadCommand::Close { reason }).await;
                return Ok(());
            }
        }
        drop(actors);

        // Нет актора — прямое обновление БД
        let target_status = match reason {
            ThreadCloseReason::ResolvedExternally => ThreadStatus::Resolved,
            ThreadCloseReason::StoppedExternally => ThreadStatus::Stopped,
            _ => ThreadStatus::Stopped,
        };

        tracing::warn!(
            thread_id = %thread_id,
            reason = ?reason,
            "Closing thread without actor — on_thread_closed will NOT be called"
        );

        self.store.update_thread_status(thread_id, target_status).await
    }
}
```

#### Прямые записи в store

```rust
impl ThreadBotHandle {
    /// Установить metadata треда (полная замена JSONB).
    ///
    /// Прямая запись в БД, без участия актора.
    /// Безопасно для handler-owned данных.
    ///
    /// **Рекомендация**: если и handler, и cron пишут metadata,
    /// используйте key-namespacing (например, `cron.*` vs `handler.*`).
    pub async fn set_thread_metadata(
        &self,
        thread_id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        self.store.set_thread_metadata(thread_id, metadata).await
    }
}
```

#### Getters

```rust
impl ThreadBotHandle {
    /// Конфигурация Mattermost API.
    ///
    /// Используйте для прямых API-вызовов из cron
    /// (например, отправка сообщений через `posts_api::create_post`).
    pub fn config(&self) -> &Arc<Configuration> {
        &self.config
    }

    /// User ID бота (если был получен в `on_start`).
    pub fn bot_user_id(&self) -> Option<String> {
        // block_on безопасен здесь — RwLock::read не делает await на I/O
        self.bot_user_id.try_read().ok().and_then(|guard| guard.clone())
    }
}
```

### Почему `ThreadBotHandle` а не `Arc<dyn ThreadStore>`

1. **Прямой `Arc<dyn ThreadStore>` обходит actor serialization.** Если cron напрямую меняет статус треда через `update_thread_status`, актор об этом не узнает. Handler может продолжать работат�� с устаревшим состоянием. `ThreadBotHandle` маршрутизирует lifecycle-операции через акторов.

2. **Нужен доступ не только к store.** Cron-колбэки нуждаются в `Configuration` (для MM API вызовов), `bot_user_id` (для фильтрации), actor-командах (`Wake`, `Close`). Один `ThreadBotHandle` инкапсулирует все зависимости.

3. **Контроль над API.** Через `ThreadBotHandle` можно расширять интерфейс, не экспонируя внутренности. Например, в будущем можно добавить `build_thread_snapshot` без изменения `ThreadStore`.

### Почему без spawn-on-demand

Первоначально рассматривался вариант с `ActorRegistry`, который мог бы спавнить актор при `wake_thread()` если его не существует. Отклонено по следующим причинам:

1. **Spawn требует `Arc<H>`.** `ThreadBotHandle` не generic по `H`, следовательно, нужен trait object или closure. Это добавляет `spawn_fn: Arc<dyn Fn(String) -> ...>` и значительно усложняет конструкцию.

2. **Edge case практически нереализуем.** Активный тред без актора возможен только в окне между перезапуском бота и первым `Hello` event (который запускает reconciliation). Это окно обычно составляет доли секунды. Cron-задачи не успевают сработать.

3. **YAGNI.** Если edge case станет проблемой, spawn-on-demand можно добавить позже без изменения публичного API `ThreadBotHandle`.

**Текущее поведение edge case:**

| Операция | Актор есть | Актора нет |
|---|---|---|
| `wake_thread` | Отправляет `Wake`, `Ok(true)` | `Ok(false)` + debug log |
| `resolve_thread` | Отправляет `Close`, полный lifecycle | Прямой DB update, **без** `on_thread_closed` |
| `stop_thread` | Отправляет `Close`, полный lifecycle | Прямой DB update, **без** `on_thread_closed` |

### Изменения в `ThreadHandler`

Добавляется один новый default-метод:

```rust
// handler.rs

#[async_trait]
pub trait ThreadHandler: Send + Sync + 'static {
    // ... существующие методы без изменений ...

    /// Настроить периодические задачи для этого handler-а.
    ///
    /// Вызывается один раз при старте бота. Handler регистрирует
    /// cron-задачи через `scheduler`, используя `handle` для доступа
    /// к данным тредов и actor-командам.
    ///
    /// Cron-замыкания синхронные (`Fn() + Send + 'static`).
    /// Для async работы используйте `tokio::runtime::Handle::current().block_on(...)`.
    ///
    /// # Аргументы
    ///
    /// - `self: Arc<Self>` — handler доступен через `Arc` для захвата
    ///   в замыкания. Клонируйте `Arc` перед каждым `scheduler.add_fn()`.
    /// - `scheduler` — cron-планировщик из `cron_tab`. Cron-выражения
    ///   используют 6 полей: `sec min hour dom month dow`.
    /// - `handle` — фасад для доступа к данным и командам thread-bot.
    ///   `Clone`-friendly, клонируйте для каждого замыкания.
    ///
    /// # Пример
    ///
    /// ```rust,no_run
    /// fn setup_cron(
    ///     self: Arc<Self>,
    ///     scheduler: &mut cron_tab::Cron<chrono::Utc>,
    ///     handle: ThreadBotHandle,
    /// ) {
    ///     let h = handle.clone();
    ///     scheduler.add_fn("0 */5 * * * *", move || {
    ///         let h = h.clone();
    ///         tokio::runtime::Handle::current().block_on(async move {
    ///             let active = h.list_threads(&[ThreadStatus::Active])
    ///                 .await.unwrap_or_default();
    ///             for thread in active {
    ///                 // ... проверка и действие ...
    ///             }
    ///         });
    ///     }).unwrap();
    /// }
    /// ```
    fn setup_cron(
        self: Arc<Self>,
        _scheduler: &mut cron_tab::Cron<chrono::Utc>,
        _handle: ThreadBotHandle,
    ) {
        // Default: no cron jobs
    }
}
```

### Изменения в `ThreadCloseReason`

Добавляются два варианта для закрытия извне (через `ThreadBotHandle`):

```rust
// handler.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadCloseReason {
    ResolvedByReaction,
    StoppedByReaction,
    ResolvedByHandler,
    StoppedByHandler,
    /// Тред закрыт как Resolved через ThreadBotHandle (cron, admin, etc.)
    ResolvedExternally,
    /// Тред закрыт как Stopped через ThreadBotHandle (cron, admin, etc.)
    StoppedExternally,
}
```

Это позволяет `on_thread_closed` различать причину закрытия и реагировать по-разному:

```rust
async fn on_thread_closed(
    &self,
    thread: &Thread,
    reason: ThreadCloseReason,
    ctx: &ThreadContext,
) -> Result<(), ThreadBotError> {
    match reason {
        ThreadCloseReason::StoppedExternally => {
            // Отправить "Тред автоматически закрыт из-за неактивности"
        }
        ThreadCloseReason::ResolvedByReaction => {
            // Отправить "Рады, что помогли!"
        }
        _ => {}
    }
    Ok(())
}
```

### Изменения в `ThreadCommand`

Добавляются два новых варианта:

```rust
// actor.rs

pub(crate) enum ThreadCommand {
    NewMessage { post: models::Post },
    ControlReaction { effect: ThreadEffect, change: ReactionChange },
    Reconcile,
    Shutdown,
    /// Разбудить актор — вызвать повторный handler run.
    /// Отправляется из ThreadBotHandle::wake_thread().
    Wake,
    /// Закрыть тред извне. Отправляется из ThreadBotHandle::{resolve,stop}_thread().
    Close { reason: ThreadCloseReason },
}
```

### Изменения в actor loop

#### Обработка `Wake`

```rust
// В select! { biased; ... } в функции thread_actor()

Some(ThreadCommand::Wake) => {
    // НЕ отменяем текущий handler — если он уже работает,
    // просто помечаем, что нужен повторный запуск.
    has_pending = true;
    debounce_deadline = Instant::now(); // немедленный запуск
    tracing::debug!(thread_id = %a.thread_id, "Thread woken by external trigger");
}
```

**Почему не отменяем handler:**

`Wake` означает "пожалуйста, перепроверь тред когда удобно". Если handler уже работает, отмена приведёт к потере работы. Вместо этого после завершения текущего handler run сработает debounce и запустит новый run с актуальным snapshot.

Это отличается от `NewMessage`, где отмена оправдана: handler работает с устаревшим snapshot без нового сообщения.

#### Обработка `Close`

```rust
// В select! { biased; ... } в функции thread_actor()

Some(ThreadCommand::Close { reason }) => {
    tracing::info!(
        thread_id = %a.thread_id,
        reason = ?reason,
        "Thread closed by external trigger"
    );

    // Отменяем текущий handler
    if handler_running {
        handler_fut = pending_handler();
        handler_running = false;
    }

    // Определяем целевой статус
    let target_status = match reason {
        ThreadCloseReason::ResolvedExternally => ThreadStatus::Resolved,
        ThreadCloseReason::StoppedExternally => ThreadStatus::Stopped,
        // Close from handle uses only *Externally variants,
        // but handle gracefully just in case
        _ => ThreadStatus::Stopped,
    };

    // Обновляем статус в БД
    if let Err(e) = a.store
        .update_thread_status(&a.thread_id, target_status)
        .await
    {
        tracing::error!(
            thread_id = %a.thread_id,
            error = %e,
            "Failed to update thread status on external close"
        );
    }

    // Собираем snapshot для on_thread_closed
    match build_thread_snapshot(&a.thread_id, &*a.store, &a.mm_config).await {
        Ok(snapshot) => {
            if let Err(e) = a.handler
                .on_thread_closed(&snapshot, reason, &a.ctx)
                .await
            {
                tracing::error!(
                    thread_id = %a.thread_id,
                    reason = ?reason,
                    error = %e,
                    "on_thread_closed failed"
                );
            }
        }
        Err(e) => {
            tracing::error!(
                thread_id = %a.thread_id,
                error = %e,
                "Failed to build snapshot for on_thread_closed"
            );
        }
    }

    break; // Актор завершается
}
```

### Изменения в `ThreadBotPlugin<H>` (runtime.rs)

#### Конструктор `ThreadBotHandle`

Приватный метод, вызываемый из `setup_cron`:

```rust
impl<H: ThreadHandler> ThreadBotPlugin<H> {
    /// Создать ThreadBotHandle, разделяющий state с этим plugin.
    fn create_handle(&self, config: Arc<Configuration>) -> ThreadBotHandle {
        ThreadBotHandle::new(
            Arc::clone(&self.store),
            config,
            Arc::clone(&self.actors),
            Arc::clone(&self.bot_user_id),
        )
    }
}
```

#### Переопределение `Plugin::setup_cron`

```rust
#[async_trait::async_trait]
impl<H: ThreadHandler> mattermost_bot::Plugin for ThreadBotPlugin<H> {
    // ... существующие методы без изменений ...

    fn setup_cron(
        self: Arc<Self>,
        scheduler: &mut cron_tab::Cron<chrono::Utc>,
        config: Arc<Configuration>,
    ) {
        let handle = self.create_handle(Arc::clone(&config));
        Arc::clone(&self.handler).setup_cron(scheduler, handle);
    }
}
```

**Важно**: `Arc::clone(&self.handler)` создаёт новый `Arc<H>`, который передаётся в `setup_cron` как `self: Arc<Self>`. Исходный `Arc<H>` в `ThreadBotPlugin` остаётся нетронутым.

### Изменения в `lib.rs`

```rust
// lib.rs

pub mod handle; // Новый модуль

// Добавить к re-exports:
pub use handle::ThreadBotHandle;
```

### Изменения в `Cargo.toml`

Новых зависимостей не требуется. `cron_tab` и `chrono` уже доступны через `mattermost_bot` (который re-export-ит их).

Для использования в `setup_cron` handler-у нужно импортировать:

```rust
use mattermost_bot::{cron_tab, chrono};
```

Или добавить прямые зависимости `cron_tab` и `chrono` в crate конкретного handler-а.

## Thread safety

### Concurrent cron + actor metadata writes

Cron может вызвать `set_thread_metadata()` одновременно с handler-ом, возвращающим `ThreadEffect::SetThreadMetadata`. Оба пишут в одно поле `metadata JSONB` в таблице `threads`.

**Гарантии:**

1. PostgreSQL гарантирует атомарность отдельных `UPDATE`. Один из writes "победит" — данные не повреждаются.
2. Семантика обоих — **полная замена** `metadata`, не merge. Последний write wins.

**Рекомендация для handler-авторов:**

Если и handler, и cron пишут metadata, используйте key-namespacing:

```json
{
    "handler": { "state": "waiting_for_reply", "attempts": 3 },
    "cron": { "last_checked_at": "2026-04-12T10:00:00Z" }
}
```

Handler пишет весь объект `metadata`, но обновляет только "свои" ключи, сохраняя чужие. Это convention, не enforcement в коде — на первом этапе достаточно.

### Actors map lock

`ThreadBotHandle` разделяет `Arc<Mutex<HashMap<...>>>` с `ThreadBotPlugin`.

**Правила:**

- Не держать lock через `.await` (кроме `Mutex::lock()` самого).
- `wake_thread` / `close_thread_impl` берут lock, извлекают `tx.clone()`, дропают lock, затем делают `tx.send().await`.
- Время удержания lock — O(1): один `HashMap::get` + `clone`.

### Cron callback threading model

`cron_tab` вызывает замыкания из своего thread pool. `block_on(async { ... })` выполняет async код в контексте текущего tokio runtime (через `Handle::current()`).

**Ограничение**: нельзя вызывать `block_on` из async контекста (panic). `cron_tab` запускает callback из sync thread, так что это безопасно.

## Структура файлов

```text
lib/thread-bot/src/
├── lib.rs                  ← добавить mod handle, re-export ThreadBotHandle
├── handle.rs               ← НОВЫЙ: ThreadBotHandle struct + impl
├── handler.rs              ← добавить setup_cron, расширить ThreadCloseReason
├── actor.rs                ← добавить Wake/Close в ThreadCommand, обработка в select!,
│                              сделать build_thread_snapshot pub
├── runtime.rs              ← реализовать setup_cron в Plugin impl
├── store.rs                ← без изменений
├── pg_store.rs             ← без изменений
├── types.rs                ← без изменений
└── error.rs                ← без изменений
```

## Полный пример использования

```rust
use std::sync::Arc;
use thread_bot::{
    async_trait, ThreadHandler, ThreadEffect, ThreadBotError,
    Thread, ThreadContext, ThreadRecord, ThreadBotHandle,
    ThreadCloseReason, ThreadStatus,
    types::ReactionChange,
};
use mattermost_bot::{cron_tab, chrono};
use mattermost_api::apis::posts_api;
use mattermost_api::models::CreatePostRequest;

struct SupportHandler {
    alert_channel_id: String,
}

#[async_trait]
impl ThreadHandler for SupportHandler {
    fn id(&self) -> &'static str { "support" }

    async fn should_track(&self, thread: &Thread, _ctx: &ThreadContext)
        -> Result<bool, ThreadBotError>
    {
        // Track threads in support channels
        Ok(thread.info.channel_id == self.alert_channel_id)
    }

    async fn handle(&self, thread: &Thread, _ctx: &ThreadContext)
        -> Result<Vec<ThreadEffect>, ThreadBotError>
    {
        // ... основная логика обработки ...
        Ok(vec![ThreadEffect::Reply {
            message: "Working on it...".to_string(),
            metadata: serde_json::Value::Null,
        }])
    }

    async fn on_thread_closed(
        &self,
        thread: &Thread,
        reason: ThreadCloseReason,
        ctx: &ThreadContext,
    ) -> Result<(), ThreadBotError> {
        let message = match reason {
            ThreadCloseReason::StoppedExternally => {
                "This thread was automatically closed due to inactivity."
            }
            ThreadCloseReason::ResolvedByReaction => {
                "Glad we could help! Thread resolved."
            }
            _ => return Ok(()),
        };

        let req = CreatePostRequest::new(
            thread.info.channel_id.clone(),
            message.to_string(),
        );
        let _ = posts_api::create_post(&ctx.config, req, None).await;

        Ok(())
    }

    fn setup_cron(
        self: Arc<Self>,
        scheduler: &mut cron_tab::Cron<chrono::Utc>,
        handle: ThreadBotHandle,
    ) {
        // ── Каждые 5 минут: проверка stale тредов ──
        let h = handle.clone();
        let alert_channel = self.alert_channel_id.clone();
        scheduler.add_fn("0 */5 * * * *", move || {
            let h = h.clone();
            let alert_channel = alert_channel.clone();
            tokio::runtime::Handle::current().block_on(async move {
                let active = h.list_threads(&[ThreadStatus::Active])
                    .await
                    .unwrap_or_default();

                for thread in active {
                    let idle = chrono::Utc::now() - thread.updated_at;
                    if idle > chrono::Duration::hours(1) {
                        let msg = format!(
                            ":warning: Thread `{}` has been idle for {}h. \
                             @oncall please check.",
                            thread.thread_id,
                            idle.num_hours()
                        );
                        let req = CreatePostRequest::new(
                            alert_channel.clone(),
                            msg,
                        );
                        let _ = posts_api::create_post(h.config(), req, None).await;
                    }
                }
            });
        }).expect("Failed to register stale thread check");

        // ── Ежедневно в 9:00: авто-закрытие abandoned тредов ──
        let h = handle.clone();
        scheduler.add_fn("0 0 9 * * *", move || {
            let h = h.clone();
            tokio::runtime::Handle::current().block_on(async move {
                let active = h.list_threads(&[ThreadStatus::Active])
                    .await
                    .unwrap_or_default();

                for thread in active {
                    let idle = chrono::Utc::now() - thread.updated_at;
                    if idle > chrono::Duration::hours(72) {
                        tracing::info!(
                            thread_id = %thread.thread_id,
                            idle_hours = idle.num_hours(),
                            "Auto-closing abandoned thread"
                        );
                        if let Err(e) = h.stop_thread(&thread.thread_id).await {
                            tracing::error!(
                                thread_id = %thread.thread_id,
                                error = %e,
                                "Failed to auto-close thread"
                            );
                        }
                    }
                }
            });
        }).expect("Failed to register auto-close job");

        // ── Каждый понедельник в 10:00: еженедельный отчёт ──
        let h = handle.clone();
        let self_clone = Arc::clone(&self);
        scheduler.add_fn("0 0 10 * * 1", move || {
            let h = h.clone();
            let handler = Arc::clone(&self_clone);
            tokio::runtime::Handle::current().block_on(async move {
                let active = h.list_threads(&[ThreadStatus::Active])
                    .await
                    .unwrap_or_default();
                let resolved = h.list_threads(&[ThreadStatus::Resolved])
                    .await
                    .unwrap_or_default();
                let stopped = h.list_threads(&[ThreadStatus::Stopped])
                    .await
                    .unwrap_or_default();

                let report = format!(
                    "## Weekly Support Report\n\
                     - Active threads: {}\n\
                     - Resolved threads: {}\n\
                     - Stopped threads: {}",
                    active.len(),
                    resolved.len(),
                    stopped.len()
                );

                let req = CreatePostRequest::new(
                    handler.alert_channel_id.clone(),
                    report,
                );
                let _ = posts_api::create_post(h.config(), req, None).await;
            });
        }).expect("Failed to register weekly report");

        // ── Каждые 30 минут: wake тредов, ждущих follow-up ──
        let h = handle;
        scheduler.add_fn("0 */30 * * * *", move || {
            let h = h.clone();
            tokio::runtime::Handle::current().block_on(async move {
                let active = h.list_threads(&[ThreadStatus::Active])
                    .await
                    .unwrap_or_default();

                for thread in active {
                    // Проверяем metadata: handler пометил тред как "ждёт follow-up"
                    if let Some(needs_followup) = thread.metadata
                        .get("handler")
                        .and_then(|h| h.get("needs_followup"))
                        .and_then(|v| v.as_bool())
                    {
                        if needs_followup {
                            let _ = h.wake_thread(&thread.thread_id).await;
                        }
                    }
                }
            });
        }).expect("Failed to register follow-up check");
    }
}
```

## Предлагаемый план реализации

### Этап 1 — Новые типы и изменения в handler.rs

1. Расширить `ThreadCloseReason` двумя вариантами: `ResolvedExternally`, `StoppedExternally`.
2. Добавить `setup_cron` default-метод в `ThreadHandler`.
3. Добавить `use crate::handle::ThreadBotHandle` import (для сигнатуры `setup_cron`).

**Проверка**: `cargo build` — существующий код не ломается (новый метод имеет default impl).

### Этап 2 — Новые команды в actor.rs

1. Добавить `Wake` и `Close { reason: ThreadCloseReason }` в `ThreadCommand`.
2. Обработать `Wake` в `select!`: `has_pending = true`, `debounce_deadline = Instant::now()`.
3. Обработать `Close` в `select!`: отменить handler, обновить статус, собрать snapshot, вызвать `on_thread_closed`, `break`.
4. Изменить видимость `build_thread_snapshot` с `pub(crate)` на `pub`.

**Проверка**: `cargo build`, `cargo clippy -- -D warnings`.

### Этап 3 — ThreadBotHandle (handle.rs)

1. Создать файл `src/handle.rs`.
2. Реализовать `ThreadBotHandle` struct с конструктором `new(...)`.
3. Реализовать read-методы: `list_threads`, `get_thread`, `get_thread_messages`, `get_thread_reactions`.
4. Реализовать `build_thread_snapshot`.
5. Реализовать `wake_thread`, `resolve_thread`, `stop_thread`, `close_thread_impl`.
6. Реализовать `set_thread_metadata`.
7. Реализовать getters: `config`, `bot_user_id`.

**Проверка**: `cargo build`.

### Этап 4 — Интеграция в runtime.rs и lib.rs

1. Добавить `mod handle` в `lib.rs`.
2. Добавить `pub use handle::ThreadBotHandle` к re-exports.
3. Добавить `create_handle` метод на `ThreadBotPlugin<H>`.
4. Переопределить `setup_cron` в `impl Plugin for ThreadBotPlugin<H>`.

**Проверка**: `cargo build`, `cargo clippy -- -D warnings`, `cargo fmt`.

### Этап 5 — Тесты

1. **Unit test: Wake команда** — актор получает `Wake`, handler вызывается повторно с актуальным snapshot.
2. **Unit test: Wake без отмены** — если handler уже работает, `Wake` не отменяет его, а ставит `has_pending`.
3. **Unit test: Close команда** — актор получает `Close`, `on_thread_closed` вызывается с правильным reason, актор завершается.
4. **Unit test: Close без актора** — `ThreadBotHandle::stop_thread` обновляет статус в БД напрямую.
5. **Unit test: read-операции** — `list_threads`, `get_thread` через handle возвращают правильные данные из mock store.
6. **Integration test: setup_cron вызывается** — проверить, что при старте бота handler получает `setup_cron` с валидным `ThreadBotHandle`.
7. **Integration test: cron → wake → handler run** — зарегистрировать cron-задачу, дождаться её срабатывания, проверить что handler вызывается.

## Открытые вопросы

1. **Per-thread inactivity timer.** Текущий RFC покрывает только глобальный cron (один scheduler на весь handler). Per-thread таймеры (например, "через 30 минут после последнего сообщения в ЭТОМ треде") — это отдельная задача, которая потребует изменений в actor loop (добавление inactivity timer branch в `select!`). Это можно сделать в следующем RFC.

2. **Metadata merge semantics.** Сейчас `set_thread_metadata` делает полную замену. Если cron и handler оба пишут metadata одновременно, один перезаписывает другого. JSON merge patch (`jsonb_set` в Postgres) решил бы проблему, но усложняет API. Для первой версии достаточно convention (key-namespacing).

3. **Cron-задачи при reconnect.** `cron_tab` запускается один раз при старте бота и работает независимо от WebSocket reconnections. Это правильное поведение: cron не должен перезапускаться при reconnect. Но если handler полагается на `bot_user_id` в cron, а `on_start` ещё не отработал, `bot_user_id()` вернёт `None`.

## Влияние на документацию

- Обновить `CLAUDE.md`: добавить `setup_cron` в описание workflow создания плагина.
- Обновить `TASKS.md`: добавить задачу и ссылку на RFC.
- Добавить пример в `examples/`: handler с cron-задачами.
