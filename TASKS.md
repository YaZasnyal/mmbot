# Tasks - mattermost-bot improvements

## 🔴 High Priority

### 1. Параллельная обработка событий
**Файл**: `lib/mattermost-bot/src/lib.rs:97-102`

**Проблема**: Плагины обрабатываются последовательно. Если один плагин долго работает, остальные ждут.

**Решение**: Spawn задачи параллельно с помощью `tokio::spawn` и `join_all`.

**Статус**: ⏳ Pending

---

### 4. Graceful shutdown
**Файл**: `lib/mattermost-bot/src/lib.rs`

**Проблема**: Нет способа корректно остановить бота.

**Решение**: Добавить поддержку `CancellationToken` из `tokio_util`.

**Статус**: ⏳ Pending

---

## 🟡 Medium Priority

### 5. Единообразное логирование
**Файлы**: 
- `lib/mattermost-bot/src/lib.rs:42` - `eprintln!`
- `lib/mattermost-bot/src/lib.rs:103` - `println!`

**Проблема**: Смешивание `tracing`, `println!` и `eprintln!`.

**Решение**: Использовать только `tracing` для всего логирования.

**Статус**: ⏳ Pending

---

### 6. Типизированные ошибки
**Файл**: `lib/mattermost-bot/src/lib.rs`

**Проблема**: Используется `Box<dyn std::error::Error>` - нет структурированных ошибок.

**Решение**: Создать `enum BotError` с помощью `thiserror`:
- `WebSocket` - ошибки WebSocket
- `MissingToken` - отсутствует токен
- `InvalidSchema` - неверная схема URL
- `EventParse` - ошибка парсинга события

**Статус**: ⏳ Pending

---

### 7. Безопасное построение URL
**Файл**: `lib/mattermost-bot/src/lib.rs:51-57`

**Проблема**: `replace()` заменяет все вхождения, не только префикс.

**Решение**: Использовать `strip_prefix()` для безопасной замены схемы.

**Статус**: ⏳ Pending

---

### 8. Публичные поля в структурах событий
**Файл**: `lib/mattermost-bot/src/types.rs`

**Проблема**: `PostedEvent` имеет публичные поля, а `ReactionAddedEvent` и `ReactionRemovedEvent` - приватные.

**Решение**: Сделать все поля публичными для consistency.

**Статус**: ⏳ Pending

---

## 🟢 Low Priority

### 9. Builder pattern для Bot
**Файл**: `lib/mattermost-bot/src/lib.rs`

**Текущий API**:
```rust
let mut bot = Bot::new(config);
bot.add_plugin(plugin1);
bot.add_plugin(plugin2);
```

**Улучшение**: Добавить fluent API:
```rust
Bot::new(config)
    .with_plugin(plugin1)
    .with_plugin(plugin2)
    .build()
    .await;
```

**Статус**: ⏳ Pending

---

### 10. Конфигурируемый reconnect delay
**Файл**: `lib/mattermost-bot/src/lib.rs:47`

**Проблема**: Hardcoded задержка в 1 секунду.

**Решение**: Сделать конфигурируемым с экспоненциальным backoff:
- `reconnect_delay` - начальная задержка
- `max_reconnect_delay` - максимальная задержка
- Удвоение задержки при каждой неудаче
- Сброс при успешном подключении

**Статус**: ⏳ Pending

---

### 11. Метрики и наблюдаемость
**Файл**: `lib/mattermost-bot/src/lib.rs`

**Решение**: Добавить структуру `BotMetrics` с атомарными счетчиками:
- `events_received` - получено событий
- `events_processed` - обработано событий
- `events_failed` - ошибки обработки
- `reconnections` - количество переподключений

**Статус**: ⏳ Pending

---

### 12. Обработка других типов WebSocket сообщений
**Файл**: `lib/mattermost-bot/src/lib.rs:103`

**Проблема**: Mattermost отправляет разные типы: `hello`, `status_change`, `typing`, и т.д. Сейчас они только логируются.

**Решение**: Структурированно обрабатывать или как минимум типизированно логировать.

**Статус**: ⏳ Pending

---

### 13. Реализовать TODO комментарии
**Файлы**:
- `lib/mattermost-bot/src/lib.rs:40` - добавить tracing log для graceful close
- `lib/mattermost-bot/src/lib.rs:41` - добавить tracing log для ошибки
- `lib/mattermost-bot/src/lib.rs:66` - улучшить аутентификацию

**Статус**: ⏳ Pending

---

### 14. Добавить unit тесты
**Файл**: `lib/mattermost-bot/tests/`

**Тесты для**:
- URL конструирования (http -> ws, https -> wss)
- Event десериализации
- Plugin фильтрации
- Nested decoder

**Статус**: ⏳ Pending

---

## Легенда статусов
- ⏳ **Pending** - ожидает выполнения
- 🚧 **In Progress** - в работе
- ✅ **Done** - выполнено
- ⏸️ **On Hold** - отложено

---

**Создано**: 2026-02-23  
**Последнее обновление**: 2026-02-23
