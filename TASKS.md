# Tasks - mattermost-bot improvements

## 🟢 Low Priority

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
