# Tasks - mmbot project roadmap

## 🔴 High Priority

**Эти задачи критичны для основной функциональности ботов**

---

### 1. Plugin lifecycle hooks
**Компонент**: `lib/mattermost-bot`  
**Файл**: `lib/mattermost-bot/src/plugin.rs`

**Описание**: Добавить методы для инициализации и очистки ресурсов плагинов.

**Решение**:
```rust
#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    fn id(&self) -> &'static str;
    
    /// Called once when bot starts (before connecting to WebSocket)
    async fn on_start(&self, config: &Arc<Configuration>) -> Result<()> {
        Ok(())
    }
    
    /// Called when bot is shutting down (after WebSocket closed)
    async fn on_shutdown(&self, config: &Arc<Configuration>) -> Result<()> {
        Ok(())
    }
    
    fn filter(&self, event: &Arc<Event>) -> bool { true }
    async fn process_event(&self, event: &Arc<Event>, config: &Arc<Configuration>);
    fn setup_cron(...) { }
}
```

**Use cases**:
- Открытие DB connections при старте
- Восстановление состояния из персистентного storage
- Graceful cleanup ресурсов при shutdown
- Регистрация slash commands в Mattermost

**Статус**: ⏳ Pending

---

### 2. Event middleware / pipeline
**Компонент**: `lib/mattermost-bot`  
**Файл**: `lib/mattermost-bot/src/lib.rs`

**Описание**: Добавить middleware для pre/post обработки событий.

**Решение**:
```rust
#[async_trait]
pub trait EventMiddleware: Send + Sync {
    /// Process event before it reaches plugins
    /// Returns false to skip event (won't be sent to plugins)
    async fn before(&self, event: &Arc<Event>) -> bool {
        true
    }
    
    /// Process event after all plugins handled it
    async fn after(&self, event: &Arc<Event>) {
        // optional hook
    }
}

impl Bot {
    pub fn with_middleware(mut self, middleware: impl EventMiddleware + 'static) -> Self {
        self.middleware.push(Arc::new(middleware));
        self
    }
}
```

**Use cases**:
- **Игнорирование собственных сообщений** (чтобы бот не отвечал сам себе в цикл)
- Логирование всех событий
- Фильтрация по глобальным условиям (например, игнорировать всех ботов)
- Rate limiting на уровне источника событий
- Enrichment событий (добавление дополнительных данных)

**Первый middleware**: `IgnoreSelfMiddleware` - игнорирует сообщения от самого бота.

**Статус**: ⏳ Pending

---

### 3. Персистентность состояния плагинов
**Компонент**: `lib/mattermost-bot`  
**Файл**: `lib/mattermost-bot/src/storage.rs` (новый)

**Описание**: Использовать `plugin.id()` для хранения состояния между запусками.

**Решение**:
```rust
pub struct PluginContext {
    pub config: Arc<Configuration>,
    pub storage: Arc<dyn Storage>,
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, plugin_id: &str, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, plugin_id: &str, key: &str, value: Vec<u8>) -> Result<()>;
    async fn delete(&self, plugin_id: &str, key: &str) -> Result<()>;
    async fn list_keys(&self, plugin_id: &str) -> Result<Vec<String>>;
}
```

**Реализации**:
- `InMemoryStorage` - по умолчанию (данные теряются при перезапуске)
- `FileStorage` - сохранение в JSON/CBOR файлы
- `SurrealDbStorage` - для production (опциональная feature)

**Use cases**:
- Сохранение прогресса обработки событий
- Кэширование данных между запусками
- Хранение пользовательских настроек плагина

**Статус**: ⏳ Pending

---

## 🟡 Medium Priority

**Задачи, которые добавят удобство, но не критичны для базовой функциональности**

---

### 4. Добавить больше типов событий Mattermost
**Компонент**: `lib/mattermost-bot`  
**Файл**: `lib/mattermost-bot/src/types.rs`

**Описание**: Расширить `EventType` по мере необходимости.

**Приоритетные события**:
- `UserAdded` / `UserRemoved` - изменения участников канала
- `ChannelCreated` / `ChannelDeleted` - управление каналами
- `DirectAdded` - новый direct message
- `ChannelViewed` - пользователь открыл канал
- `Typing` - пользователь печатает (low priority, может быть много событий)
- `StatusChange` - статус пользователя изменился (online/away/offline)
- `PreferencesChanged` - изменение настроек пользователя

**Подход**: Добавляем события по мере реального использования в проекте.

**Статус**: ⏳ Pending (добавляем по необходимости)

---

### 5. Конфигурируемый reconnect delay с exponential backoff
**Компонент**: `lib/mattermost-bot`  
**Файл**: `lib/mattermost-bot/src/lib.rs:47`

**Проблема**: Hardcoded задержка в 1 секунду при переподключении.

**Решение**: Сделать конфигурируемым:
```rust
pub struct BotConfig {
    pub initial_reconnect_delay: Duration,  // default: 1s
    pub max_reconnect_delay: Duration,      // default: 60s
    pub backoff_multiplier: f64,            // default: 2.0
}
```

**Поведение**:
- Удвоение задержки при каждой неудаче (до максимума)
- Сброс до начальной задержки при успешном подключении
- Логирование количества попыток

**Статус**: ⏳ Pending

---

### 6. Structured logging для событий
**Компонент**: `lib/mattermost-bot`  
**Файл**: `lib/mattermost-bot/src/lib.rs`

**Описание**: Улучшить логирование с structured fields.

**Пример**:
```rust
tracing::info!(
    event_type = ?event.data,
    channel_id = ?channel_id,
    user_id = ?user_id,
    plugin_count = filtered_plugins.len(),
    "Processing event"
);
```

**Статус**: ⏳ Pending

---

## 🟢 Low Priority

**Nice-to-have улучшения, которые не блокируют разработку**

---

### 7. Метрики и наблюдаемость
**Компонент**: `lib/mattermost-bot`  
**Файл**: `lib/mattermost-bot/src/metrics.rs` (новый)

**Решение**: Добавить опциональную feature `metrics`:
```rust
pub struct BotMetrics {
    pub events_received: AtomicU64,
    pub events_processed: AtomicU64,
    pub plugins_executed: AtomicU64,
    pub reconnections: AtomicU64,
    pub uptime_start: Instant,
}

impl Bot {
    pub fn metrics(&self) -> Option<&BotMetrics> { ... }
}
```

**Use cases**:
- Мониторинг в production
- Health check endpoints
- Prometheus metrics export

**Статус**: ⏳ Pending (делаем если появится реальная необходимость)

---

### 8. Health checks API
**Компонент**: `lib/mattermost-bot`  
**Файл**: `lib/mattermost-bot/src/lib.rs`

**Решение**:
```rust
impl Bot {
    pub fn is_connected(&self) -> bool { ... }
    pub fn uptime(&self) -> Duration { ... }
    pub fn last_event_time(&self) -> Option<Instant> { ... }
}
```

**Статус**: ⏳ Pending

---

## 🎯 Future Layers (Roadmap)

**Планируемая архитектура проекта - 4 слоя абстракции**

---

### Layer 3: lib/thread-bot

**Назначение**: Упрощение работы с тредами (thread-based боты)

**Планируемая функциональность**:
- Отслеживание тредов в каналах
- Сериализация обработчиков для одного треда (избегание race conditions)
- Прерывание старых обработчиков при новых сообщениях в треде
- Управление состоянием треда
- События: `thread_created`, `thread_message_added`, `thread_resolved`

**Интерфейс** (черновик):
```rust
#[async_trait]
pub trait ThreadHandler: Send + Sync + 'static {
    fn id(&self) -> &'static str;
    
    /// Called when new thread is created
    async fn on_thread_start(&self, thread: &Thread, ctx: &ThreadContext);
    
    /// Called when new message added to thread
    async fn on_thread_message(&self, message: &Post, thread: &Thread, ctx: &ThreadContext);
    
    /// Called when thread is considered "done" (timeout or explicit signal)
    async fn on_thread_end(&self, thread: &Thread, ctx: &ThreadContext);
}
```

**Ключевая фича**: Автоматическая сериализация - сообщения в одном треде обрабатываются последовательно, в разных тредах - параллельно.

**Статус**: 📋 Planned (начнём после завершения lib/mattermost-bot)

---

### Layer 4: Support Bot (конкретная реализация)

**Назначение**: Бот технической поддержки с LLM интеграцией

**Планируемая функциональность**:
- Мониторинг тредов в каналах поддержки
- Ответы на вопросы пользователей с помощью LLM
- Интеграция с документацией проекта (RAG)
- Инструменты для поиска известных проблем
- Управление через реакции (👍/👎/🔄)
- Автоматическая категоризация проблем
- Эскалация сложных вопросов

**Гибридный подход**:
- Часть проблем решается без LLM (известные issue patterns)
- LLM используется для сложных/новых вопросов
- Бот выбирает категорию и инструменты автоматически

**Статус**: 📋 Planned (финальная цель проекта)

---

## 🏗️ Архитектурные принципы

### Почему процесс_event не возвращает Result?

**Решение**: `async fn process_event(&self, event: &Arc<Event>, config: &Arc<Configuration>)`

**Обоснование**:
- Ошибка в одном плагине не должна ломать обработку в других плагинах
- С ошибкой всё равно ничего нельзя сделать - событие уже произошло
- Плагины должны сами восстанавливаться от ошибок
- Если критическая ошибка - плагин может запаниковать, бот продолжит работу
- Для логирования ошибок плагины используют `tracing::error!` внутри

### Почему нет backpressure/rate limiting?

**Обоснование**:
- Обрабатываем одно WebSocket событие за раз
- Параллелизм только между плагинами для одного события
- Mattermost сам контролирует rate на стороне сервера
- Если нужен rate limiting - делаем через middleware

### Принцип минимализма

- Добавляем функциональность **по мере необходимости**
- Не усложняем API преждевременно
- Сохраняем простоту использования для базовых случаев

---

## Легенда статусов

- ⏳ **Pending** - ожидает выполнения
- 🚧 **In Progress** - в работе  
- ✅ **Done** - выполнено
- ⏸️ **On Hold** - отложено
- 📋 **Planned** - запланировано на будущее

---

**Создано**: 2026-02-23  
**Последнее обновление**: 2026-02-23 (после обсуждения архитектуры)
