use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;
use std::sync::Arc;
use std::time::Duration;

const DURATION_BUCKETS: [f64; 11] = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

type ActiveActors = Family<HandlerLabels, Gauge>;
type ActorStarts = Family<OutcomeLabels, Counter>;
type ActorStops = Family<ReasonLabels, Counter>;
type ActorCommands = Family<CommandLabels, Counter>;
type HandlerRuns = Family<OutcomeLabels, Counter>;
type HandlerDuration = Family<OutcomeLabels, Histogram, fn() -> Histogram>;
type Effects = Family<EffectLabels, Counter>;
type ReconcileRuns = Family<OutcomeLabels, Counter>;
type ReconcileDuration = Family<OutcomeLabels, Histogram, fn() -> Histogram>;

#[derive(Clone, Debug)]
pub struct ThreadBotMetrics {
    inner: Arc<ThreadBotMetricsInner>,
}

#[derive(Debug)]
struct ThreadBotMetricsInner {
    active_actors: ActiveActors,
    actor_starts: ActorStarts,
    actor_stops: ActorStops,
    actor_commands: ActorCommands,
    handler_runs: HandlerRuns,
    handler_duration: HandlerDuration,
    effects: Effects,
    reconcile_runs: ReconcileRuns,
    reconcile_duration: ReconcileDuration,
}

impl ThreadBotMetrics {
    pub fn register(registry: &mut Registry) -> Self {
        let active_actors = ActiveActors::default();
        let actor_starts = ActorStarts::default();
        let actor_stops = ActorStops::default();
        let actor_commands = ActorCommands::default();
        let handler_runs = HandlerRuns::default();
        let handler_duration =
            HandlerDuration::new_with_constructor(duration_histogram as fn() -> Histogram);
        let effects = Effects::default();
        let reconcile_runs = ReconcileRuns::default();
        let reconcile_duration =
            ReconcileDuration::new_with_constructor(duration_histogram as fn() -> Histogram);

        registry.register(
            "thread_bot_active_actors",
            "Current active thread-bot actors.",
            active_actors.clone(),
        );
        registry.register(
            "thread_bot_actor_starts",
            "Thread-bot actor starts by outcome.",
            actor_starts.clone(),
        );
        registry.register(
            "thread_bot_actor_stops",
            "Thread-bot actor stops by reason.",
            actor_stops.clone(),
        );
        registry.register(
            "thread_bot_actor_commands",
            "Thread-bot actor commands by command and outcome.",
            actor_commands.clone(),
        );
        registry.register(
            "thread_bot_handler_runs",
            "Thread-bot handler runs by outcome.",
            handler_runs.clone(),
        );
        registry.register(
            "thread_bot_handler_duration_seconds",
            "Thread-bot handler run duration in seconds.",
            handler_duration.clone(),
        );
        registry.register(
            "thread_bot_effects",
            "Thread-bot effects by kind and outcome.",
            effects.clone(),
        );
        registry.register(
            "thread_bot_reconcile_runs",
            "Thread-bot reconciliation runs by outcome.",
            reconcile_runs.clone(),
        );
        registry.register(
            "thread_bot_reconcile_duration_seconds",
            "Thread-bot reconciliation duration in seconds.",
            reconcile_duration.clone(),
        );

        Self {
            inner: Arc::new(ThreadBotMetricsInner {
                active_actors,
                actor_starts,
                actor_stops,
                actor_commands,
                handler_runs,
                handler_duration,
                effects,
                reconcile_runs,
                reconcile_duration,
            }),
        }
    }

    pub fn for_bot(
        &self,
        bot: impl Into<String>,
        handler: impl Into<String>,
    ) -> ThreadBotMetricsHandle {
        ThreadBotMetricsHandle {
            bot: bot.into(),
            handler: handler.into(),
            inner: Some(Arc::clone(&self.inner)),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ThreadBotMetricsHandle {
    bot: String,
    handler: String,
    inner: Option<Arc<ThreadBotMetricsInner>>,
}

impl ThreadBotMetricsHandle {
    pub fn noop() -> Self {
        Self::default()
    }

    pub fn actor_started(&self, outcome: &'static str) {
        let Some(inner) = &self.inner else {
            return;
        };
        inner
            .active_actors
            .get_or_create(&self.handler_labels())
            .inc();
        inner
            .actor_starts
            .get_or_create(&self.outcome_labels(outcome))
            .inc();
    }

    pub fn actor_stopped(&self, reason: &'static str) {
        let Some(inner) = &self.inner else {
            return;
        };
        inner
            .active_actors
            .get_or_create(&self.handler_labels())
            .dec();
        inner
            .actor_stops
            .get_or_create(&ReasonLabels {
                bot: self.bot.clone(),
                handler: self.handler.clone(),
                reason: reason.to_string(),
            })
            .inc();
    }

    pub fn actor_command(&self, command: &'static str, outcome: &'static str) {
        let Some(inner) = &self.inner else {
            return;
        };
        inner
            .actor_commands
            .get_or_create(&CommandLabels {
                bot: self.bot.clone(),
                handler: self.handler.clone(),
                command: command.to_string(),
                outcome: outcome.to_string(),
            })
            .inc();
    }

    pub fn handler_run(&self, outcome: &'static str, duration: Duration) {
        let Some(inner) = &self.inner else {
            return;
        };
        let labels = self.outcome_labels(outcome);
        inner.handler_runs.get_or_create(&labels).inc();
        inner
            .handler_duration
            .get_or_create(&labels)
            .observe(duration.as_secs_f64());
    }

    pub fn effect(&self, effect: &'static str, outcome: &'static str) {
        let Some(inner) = &self.inner else {
            return;
        };
        inner
            .effects
            .get_or_create(&EffectLabels {
                bot: self.bot.clone(),
                handler: self.handler.clone(),
                effect: effect.to_string(),
                outcome: outcome.to_string(),
            })
            .inc();
    }

    pub fn reconcile(&self, outcome: &'static str, duration: Duration) {
        let Some(inner) = &self.inner else {
            return;
        };
        let labels = self.outcome_labels(outcome);
        inner.reconcile_runs.get_or_create(&labels).inc();
        inner
            .reconcile_duration
            .get_or_create(&labels)
            .observe(duration.as_secs_f64());
    }

    fn handler_labels(&self) -> HandlerLabels {
        HandlerLabels {
            bot: self.bot.clone(),
            handler: self.handler.clone(),
        }
    }

    fn outcome_labels(&self, outcome: &'static str) -> OutcomeLabels {
        OutcomeLabels {
            bot: self.bot.clone(),
            handler: self.handler.clone(),
            outcome: outcome.to_string(),
        }
    }
}

fn duration_histogram() -> Histogram {
    Histogram::new(DURATION_BUCKETS)
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct HandlerLabels {
    bot: String,
    handler: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct OutcomeLabels {
    bot: String,
    handler: String,
    outcome: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ReasonLabels {
    bot: String,
    handler: String,
    reason: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct CommandLabels {
    bot: String,
    handler: String,
    command: String,
    outcome: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct EffectLabels {
    bot: String,
    handler: String,
    effect: String,
    outcome: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus_client::encoding::text::encode;

    #[test]
    fn active_actor_gauge_tracks_start_and_stop() {
        let mut registry = Registry::default();
        let metrics = ThreadBotMetrics::register(&mut registry);
        let handle = metrics.for_bot("support_bot", "support_handler");

        handle.actor_started("success");
        handle.actor_stopped("shutdown");

        let mut output = String::new();
        encode(&mut output, &registry).unwrap();

        assert!(output.contains(
            "thread_bot_active_actors{bot=\"support_bot\",handler=\"support_handler\"} 0"
        ));
        assert!(output.contains("thread_bot_actor_starts_total{bot=\"support_bot\",handler=\"support_handler\",outcome=\"success\"} 1"));
        assert!(output.contains("thread_bot_actor_stops_total{bot=\"support_bot\",handler=\"support_handler\",reason=\"shutdown\"} 1"));
    }
}
