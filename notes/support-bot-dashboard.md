# Support Bot Dashboard Sketch

Companion notes for `notes/support-bot-dashboard.json`. The JSON is a Grafana
draft; if import fails, use this document to rebuild the board manually.

## Variables

- `DS_PROMETHEUS`: datasource variable, type `prometheus`.
- `bot`: query variable, multi-select, include all.
  - Query: `label_values(support_bot_thread_events_total, bot)`
  - Default: `support_bot`

## Layout

```text
┌───────────────┬───────────────┬───────────────┬───────────────┐
│ 1 Actors      │ 2 LLM Errors  │ 3 Handler p95 │ 4 WS Errors   │
├───────────────┴───────────────┬───────────────┴───────────────┤
│ 5 Mattermost Events           │ 6 Support Route Outcomes      │
├───────────────────────────────┼───────────────────────────────┤
│ 7 LLM Latency                 │ 8 Tool Latency p95            │
├───────────────────────────────┼───────────────────────────────┤
│ 9 Tool Calls                  │ 10 Thread Effects             │
├───────────────────────────────┼───────────────────────────────┤
│ 11 Replies / Notifications    │ 12 WS Connection Outcomes     │
└───────────────────────────────┴───────────────────────────────┘
```

## Panels

1. Active Actors
   - Type: Stat
   - Unit: `short`
   - Query:
     ```promql
     sum(thread_bot_active_actors{bot=~"$bot"})
     ```
   - Display: green single stat, show sparkline/area if available.

2. LLM Error Ratio
   - Type: Stat
   - Unit: `percentunit`
   - Thresholds: green `< 1%`, red `>= 1%`
   - Query:
     ```promql
     sum(rate(support_bot_llm_requests_total{bot=~"$bot",outcome="error"}[$__rate_interval]))
       /
     clamp_min(sum(rate(support_bot_llm_requests_total{bot=~"$bot"}[$__rate_interval])), 0.001)
     ```

3. Handler p95
   - Type: Stat
   - Unit: seconds
   - Thresholds: green `< 1s`, yellow `>= 1s`, red `>= 3s`
   - Query:
     ```promql
     histogram_quantile(
       0.95,
       sum by (le) (
         rate(thread_bot_handler_duration_seconds_bucket{bot=~"$bot"}[$__rate_interval])
       )
     )
     ```

4. WS Parse Errors
   - Type: Stat
   - Unit: ops/sec
   - Thresholds: green at `0`, orange/red above `1`
   - Query:
     ```promql
     sum(rate(mattermost_bot_ws_messages_total{bot=~"$bot",outcome="parse_error"}[$__rate_interval]))
     ```

5. Mattermost Events
   - Type: Time series
   - Unit: req/sec
   - Legend: table, bottom, show last/max
   - Query:
     ```promql
     sum by (event_type) (
       rate(mattermost_bot_events_total{bot=~"$bot"}[$__rate_interval])
     )
     ```

6. Support Route Outcomes
   - Type: Time series
   - Unit: ops/sec
   - Legend: `{{route}} / {{outcome}}`
   - Query:
     ```promql
     sum by (route, outcome) (
       rate(support_bot_thread_events_total{bot=~"$bot"}[$__rate_interval])
     )
     ```

7. LLM Latency
   - Type: Time series
   - Unit: seconds
   - Legend: p50, p95, p99
   - Queries:
     ```promql
     histogram_quantile(0.50, sum by (le) (rate(support_bot_llm_duration_seconds_bucket{bot=~"$bot"}[$__rate_interval])))
     histogram_quantile(0.95, sum by (le) (rate(support_bot_llm_duration_seconds_bucket{bot=~"$bot"}[$__rate_interval])))
     histogram_quantile(0.99, sum by (le) (rate(support_bot_llm_duration_seconds_bucket{bot=~"$bot"}[$__rate_interval])))
     ```

8. Tool Latency p95
   - Type: Time series
   - Unit: seconds
   - Legend: `{{tool_name}} p95`
   - Query:
     ```promql
     histogram_quantile(
       0.95,
       sum by (tool_name, le) (
         rate(support_bot_tool_duration_seconds_bucket{bot=~"$bot"}[$__rate_interval])
       )
     )
     ```
   - Styling: keep legend as table; this panel can get noisy when many tools
     are registered.

9. Tool Calls
   - Type: Table
   - Unit: ops/sec
   - Query:
     ```promql
     sum by (tool_name, outcome) (
       rate(support_bot_tool_calls_total{bot=~"$bot"}[$__rate_interval])
     )
     ```
   - Mode: instant query, table format.
   - Styling: enable column filters; sort value descending.

10. Thread Effects
    - Type: Time series
    - Unit: ops/sec
    - Legend: `{{effect}} / {{outcome}}`
    - Query:
      ```promql
      sum by (effect, outcome) (
        rate(thread_bot_effects_total{bot=~"$bot"}[$__rate_interval])
      )
      ```

11. Replies And Notifications
    - Type: Time series
    - Unit: ops/sec
    - Legend: `{{target}} / {{outcome}}`
    - Query:
      ```promql
      sum by (target, outcome) (
        rate(support_bot_replies_total{bot=~"$bot"}[$__rate_interval])
      )
      ```

12. WS Connection Outcomes
    - Type: Time series
    - Unit: ops/sec
    - Legend: `{{outcome}}`
    - Query:
      ```promql
      sum by (outcome) (
        rate(mattermost_bot_ws_connections_total{bot=~"$bot"}[$__rate_interval])
      )
      ```

## Suggested Alerts

- LLM error ratio: warn above `1%` for 10 minutes, critical above `5%`.
- Handler p95: warn above `3s` for 10 minutes.
- Active actors: warn if it keeps increasing while route throughput is flat.
- WS parse errors: warn on any sustained non-zero rate.
- Tool action errors: warn on
  `sum(rate(support_bot_tool_calls_total{outcome=~"error|action_error"}[5m])) > 0`.

## Notes

- Prefer rates for counters and current values for gauges.
- Avoid adding `thread_id`, `channel_id`, `post_id`, `user_id`, URLs, raw error
  text, or model prompt fields as labels.
- If `bot=All` is selected, latency histograms aggregate across bots. That is
  useful for fleet view but can hide one slow bot; switch to a single bot when
  debugging.
