use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;
use std::sync::Arc;
use std::time::Duration;

const DURATION_BUCKETS: [f64; 11] = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

type WsConnections = Family<OutcomeLabels, Counter>;
type WsConnectionDuration = Family<OutcomeLabels, Histogram, fn() -> Histogram>;
type WsMessages = Family<WsMessageLabels, Counter>;
type Events = Family<EventLabels, Counter>;
type PluginEvents = Family<PluginEventLabels, Counter>;
type PluginEventDuration = Family<PluginDurationLabels, Histogram, fn() -> Histogram>;
type PluginLifecycle = Family<LifecycleLabels, Counter>;

#[derive(Clone, Debug)]
pub struct MattermostBotMetrics {
    inner: Arc<MattermostBotMetricsInner>,
}

#[derive(Debug)]
struct MattermostBotMetricsInner {
    ws_connections: WsConnections,
    ws_connection_duration: WsConnectionDuration,
    ws_messages: WsMessages,
    events: Events,
    plugin_events: PluginEvents,
    plugin_event_duration: PluginEventDuration,
    plugin_lifecycle: PluginLifecycle,
}

impl MattermostBotMetrics {
    pub fn register(registry: &mut Registry) -> Self {
        let ws_connections = WsConnections::default();
        let ws_connection_duration =
            WsConnectionDuration::new_with_constructor(duration_histogram as fn() -> Histogram);
        let ws_messages = WsMessages::default();
        let events = Events::default();
        let plugin_events = PluginEvents::default();
        let plugin_event_duration =
            PluginEventDuration::new_with_constructor(duration_histogram as fn() -> Histogram);
        let plugin_lifecycle = PluginLifecycle::default();

        registry.register(
            "mattermost_bot_ws_connections",
            "Mattermost WebSocket connection attempts by outcome.",
            ws_connections.clone(),
        );
        registry.register(
            "mattermost_bot_ws_connection_duration_seconds",
            "Mattermost WebSocket connection duration in seconds.",
            ws_connection_duration.clone(),
        );
        registry.register(
            "mattermost_bot_ws_messages",
            "Mattermost WebSocket inbound messages by kind and outcome.",
            ws_messages.clone(),
        );
        registry.register(
            "mattermost_bot_events",
            "Mattermost normalized events by event type.",
            events.clone(),
        );
        registry.register(
            "mattermost_bot_plugin_events",
            "Mattermost plugin event fanout by plugin and outcome.",
            plugin_events.clone(),
        );
        registry.register(
            "mattermost_bot_plugin_event_duration_seconds",
            "Mattermost plugin event processing duration in seconds.",
            plugin_event_duration.clone(),
        );
        registry.register(
            "mattermost_bot_plugin_lifecycle",
            "Mattermost plugin lifecycle hooks by outcome.",
            plugin_lifecycle.clone(),
        );

        Self {
            inner: Arc::new(MattermostBotMetricsInner {
                ws_connections,
                ws_connection_duration,
                ws_messages,
                events,
                plugin_events,
                plugin_event_duration,
                plugin_lifecycle,
            }),
        }
    }

    pub fn for_bot(&self, bot: impl Into<String>) -> MattermostBotMetricsHandle {
        MattermostBotMetricsHandle {
            bot: bot.into(),
            inner: Some(Arc::clone(&self.inner)),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MattermostBotMetricsHandle {
    bot: String,
    inner: Option<Arc<MattermostBotMetricsInner>>,
}

impl MattermostBotMetricsHandle {
    pub fn noop() -> Self {
        Self::default()
    }

    pub fn ws_connection(&self, outcome: &'static str, duration: Duration) {
        let Some(inner) = &self.inner else {
            return;
        };
        let labels = OutcomeLabels {
            bot: self.bot.clone(),
            outcome: outcome.to_string(),
        };
        inner.ws_connections.get_or_create(&labels).inc();
        inner
            .ws_connection_duration
            .get_or_create(&labels)
            .observe(duration.as_secs_f64());
    }

    pub fn ws_message(&self, message_kind: &'static str, outcome: &'static str) {
        let Some(inner) = &self.inner else {
            return;
        };
        inner
            .ws_messages
            .get_or_create(&WsMessageLabels {
                bot: self.bot.clone(),
                message_kind: message_kind.to_string(),
                outcome: outcome.to_string(),
            })
            .inc();
    }

    pub fn event(&self, event_type: &'static str) {
        let Some(inner) = &self.inner else {
            return;
        };
        inner
            .events
            .get_or_create(&EventLabels {
                bot: self.bot.clone(),
                event_type: event_type.to_string(),
            })
            .inc();
    }

    pub fn plugin_event(
        &self,
        plugin: &'static str,
        event_type: &'static str,
        outcome: &'static str,
        duration: Duration,
    ) {
        let Some(inner) = &self.inner else {
            return;
        };
        inner
            .plugin_events
            .get_or_create(&PluginEventLabels {
                bot: self.bot.clone(),
                plugin: plugin.to_string(),
                event_type: event_type.to_string(),
                outcome: outcome.to_string(),
            })
            .inc();
        inner
            .plugin_event_duration
            .get_or_create(&PluginDurationLabels {
                bot: self.bot.clone(),
                plugin: plugin.to_string(),
                event_type: event_type.to_string(),
            })
            .observe(duration.as_secs_f64());
    }

    pub fn plugin_lifecycle(
        &self,
        plugin: &'static str,
        hook: &'static str,
        outcome: &'static str,
    ) {
        let Some(inner) = &self.inner else {
            return;
        };
        inner
            .plugin_lifecycle
            .get_or_create(&LifecycleLabels {
                bot: self.bot.clone(),
                plugin: plugin.to_string(),
                hook: hook.to_string(),
                outcome: outcome.to_string(),
            })
            .inc();
    }
}

fn duration_histogram() -> Histogram {
    Histogram::new(DURATION_BUCKETS)
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct OutcomeLabels {
    bot: String,
    outcome: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct WsMessageLabels {
    bot: String,
    message_kind: String,
    outcome: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct EventLabels {
    bot: String,
    event_type: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct PluginEventLabels {
    bot: String,
    plugin: String,
    event_type: String,
    outcome: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct PluginDurationLabels {
    bot: String,
    plugin: String,
    event_type: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct LifecycleLabels {
    bot: String,
    plugin: String,
    hook: String,
    outcome: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus_client::encoding::text::encode;

    #[test]
    fn encodes_plugin_lifecycle_and_ws_message_labels() {
        let mut registry = Registry::default();
        let metrics = MattermostBotMetrics::register(&mut registry);
        let handle = metrics.for_bot("support_bot");

        handle.ws_message("text", "event");
        handle.plugin_lifecycle("thread-bot", "on_start", "success");

        let mut output = String::new();
        encode(&mut output, &registry).unwrap();

        assert!(output.contains("mattermost_bot_ws_messages_total{bot=\"support_bot\",message_kind=\"text\",outcome=\"event\"} 1"));
        assert!(output.contains("mattermost_bot_plugin_lifecycle_total{bot=\"support_bot\",plugin=\"thread-bot\",hook=\"on_start\",outcome=\"success\"} 1"));
    }
}
