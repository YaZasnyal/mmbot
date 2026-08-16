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

type ThreadEvents = Family<ThreadEventLabels, Counter>;
type DurationFamily = Family<DurationLabels, Histogram, fn() -> Histogram>;
type AcpRequests = Family<AcpLabels, Counter>;
type AcpDuration = Family<AcpLabels, Histogram, fn() -> Histogram>;
type Replies = Family<ReplyLabels, Counter>;
type ThreadCloses = Family<CloseLabels, Counter>;

#[derive(Clone, Debug)]
pub struct SupportBotMetrics {
    inner: Arc<SupportBotMetricsInner>,
}

#[derive(Debug)]
struct SupportBotMetricsInner {
    thread_events: ThreadEvents,
    handle_duration: DurationFamily,
    acp_requests: AcpRequests,
    acp_duration: AcpDuration,
    replies: Replies,
    thread_closes: ThreadCloses,
}

impl SupportBotMetrics {
    pub fn register(registry: &mut Registry) -> Self {
        let thread_events = ThreadEvents::default();
        let handle_duration =
            DurationFamily::new_with_constructor(duration_histogram as fn() -> Histogram);
        let acp_requests = AcpRequests::default();
        let acp_duration =
            AcpDuration::new_with_constructor(duration_histogram as fn() -> Histogram);
        let replies = Replies::default();
        let thread_closes = ThreadCloses::default();

        registry.register(
            "support_bot_thread_events",
            "Support-bot thread events by route and outcome.",
            thread_events.clone(),
        );
        registry.register(
            "support_bot_handle_duration_seconds",
            "Support-bot handler duration in seconds.",
            handle_duration.clone(),
        );
        registry.register(
            "support_bot_acp_requests",
            "Support-bot ACP requests by outcome.",
            acp_requests.clone(),
        );
        registry.register(
            "support_bot_acp_duration_seconds",
            "Support-bot ACP request duration in seconds.",
            acp_duration.clone(),
        );
        registry.register(
            "support_bot_replies",
            "Support-bot replies and notifications by target and outcome.",
            replies.clone(),
        );
        registry.register(
            "support_bot_thread_closes",
            "Support-bot thread close notifications by reason and outcome.",
            thread_closes.clone(),
        );

        Self {
            inner: Arc::new(SupportBotMetricsInner {
                thread_events,
                handle_duration,
                acp_requests,
                acp_duration,
                replies,
                thread_closes,
            }),
        }
    }

    pub fn for_bot(&self, bot: impl Into<String>) -> SupportBotMetricsHandle {
        SupportBotMetricsHandle {
            bot: bot.into(),
            inner: Some(Arc::clone(&self.inner)),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SupportBotMetricsHandle {
    bot: String,
    inner: Option<Arc<SupportBotMetricsInner>>,
}

impl SupportBotMetricsHandle {
    pub fn noop() -> Self {
        Self::default()
    }

    pub fn record_thread_event(
        &self,
        route: &'static str,
        event: &'static str,
        outcome: &'static str,
    ) {
        let Some(inner) = &self.inner else {
            return;
        };
        inner
            .thread_events
            .get_or_create(&ThreadEventLabels {
                bot: self.bot.clone(),
                route: route.to_string(),
                event: event.to_string(),
                outcome: outcome.to_string(),
            })
            .inc();
    }

    pub fn observe_handle_duration(
        &self,
        route: &'static str,
        outcome: &'static str,
        duration: Duration,
    ) {
        let Some(inner) = &self.inner else {
            return;
        };
        inner
            .handle_duration
            .get_or_create(&DurationLabels {
                bot: self.bot.clone(),
                route: route.to_string(),
                outcome: outcome.to_string(),
            })
            .observe(duration.as_secs_f64());
    }

    pub fn record_acp_request(&self, outcome: &'static str, duration: Duration) {
        let Some(inner) = &self.inner else {
            return;
        };
        let labels = AcpLabels {
            bot: self.bot.clone(),
            outcome: outcome.to_string(),
        };
        inner.acp_requests.get_or_create(&labels).inc();
        inner
            .acp_duration
            .get_or_create(&labels)
            .observe(duration.as_secs_f64());
    }

    pub fn record_reply(&self, target: &'static str, outcome: &'static str) {
        let Some(inner) = &self.inner else {
            return;
        };
        inner
            .replies
            .get_or_create(&ReplyLabels {
                bot: self.bot.clone(),
                target: target.to_string(),
                outcome: outcome.to_string(),
            })
            .inc();
    }

    pub fn record_thread_close(&self, reason: &'static str, outcome: &'static str) {
        let Some(inner) = &self.inner else {
            return;
        };
        inner
            .thread_closes
            .get_or_create(&CloseLabels {
                bot: self.bot.clone(),
                reason: reason.to_string(),
                outcome: outcome.to_string(),
            })
            .inc();
    }
}

fn duration_histogram() -> Histogram {
    Histogram::new(DURATION_BUCKETS)
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ThreadEventLabels {
    bot: String,
    route: String,
    event: String,
    outcome: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct DurationLabels {
    bot: String,
    route: String,
    outcome: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct AcpLabels {
    bot: String,
    outcome: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ReplyLabels {
    bot: String,
    target: String,
    outcome: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct CloseLabels {
    bot: String,
    reason: String,
    outcome: String,
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
