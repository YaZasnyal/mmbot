# Tasks - mmbot project roadmap

## 📋 RFC (Request for Comments)

Задачи, которые были проработаны, но решено не реализовывать или отложить:

- [RFC 0001: Storage и персистентность](rfcs/0001-storage-persistence.md) - Решено НЕ добавлять Storage в Layer 2

---

## 🔴 High Priority

**Эти задачи критичны для основной функциональности ботов**

**Статус**: ✅ Все High Priority задачи для Layer 2 завершены!

---

### ~~1. Plugin lifecycle hooks~~ ✅ DONE
**Статус**: ✅ Завершено (коммит a4a59ee)

---

### ~~2. Event middleware / pipeline~~ ✅ DONE  
**Статус**: ✅ Завершено (коммит b1a2aed)

Реализован `IgnoreSelf` middleware как пример композитного паттерна.

---

### ~~3. Персистентность состояния плагинов~~ ❌ ОТКЛОНЕНО

**Статус**: ❌ Намеренно НЕ реализуется в Layer 2

**Причина**: См. [RFC 0001](rfcs/0001-storage-persistence.md)

Ключевые выводы:
- Реконсиляция per-thread неизбежна при старте
- Layer 3 (thread-bot) реализует свою персистентность
- Layer 2 остаётся простым event streaming фреймворком
- Плагинам для простых use cases персистентность не нужна

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
