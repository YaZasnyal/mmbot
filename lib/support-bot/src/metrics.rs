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
type LlmRequests = Family<LlmLabels, Counter>;
type LlmDuration = Family<LlmLabels, Histogram, fn() -> Histogram>;
type ToolCalls = Family<ToolLabels, Counter>;
type ToolDuration = Family<ToolLabels, Histogram, fn() -> Histogram>;
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
    llm_requests: LlmRequests,
    llm_duration: LlmDuration,
    tool_calls: ToolCalls,
    tool_duration: ToolDuration,
    replies: Replies,
    thread_closes: ThreadCloses,
}

impl SupportBotMetrics {
    pub fn register(registry: &mut Registry) -> Self {
        let thread_events = ThreadEvents::default();
        let handle_duration =
            DurationFamily::new_with_constructor(duration_histogram as fn() -> Histogram);
        let llm_requests = LlmRequests::default();
        let llm_duration =
            LlmDuration::new_with_constructor(duration_histogram as fn() -> Histogram);
        let tool_calls = ToolCalls::default();
        let tool_duration =
            ToolDuration::new_with_constructor(duration_histogram as fn() -> Histogram);
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
            "support_bot_llm_requests",
            "Support-bot LLM requests by outcome.",
            llm_requests.clone(),
        );
        registry.register(
            "support_bot_llm_duration_seconds",
            "Support-bot LLM request duration in seconds.",
            llm_duration.clone(),
        );
        registry.register(
            "support_bot_tool_calls",
            "Support-bot tool calls by tool name and outcome.",
            tool_calls.clone(),
        );
        registry.register(
            "support_bot_tool_duration_seconds",
            "Support-bot tool call duration in seconds.",
            tool_duration.clone(),
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
                llm_requests,
                llm_duration,
                tool_calls,
                tool_duration,
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

    pub fn record_llm_request(&self, outcome: &'static str, duration: Duration) {
        let Some(inner) = &self.inner else {
            return;
        };
        let labels = LlmLabels {
            bot: self.bot.clone(),
            outcome: outcome.to_string(),
        };
        inner.llm_requests.get_or_create(&labels).inc();
        inner
            .llm_duration
            .get_or_create(&labels)
            .observe(duration.as_secs_f64());
    }

    pub fn record_tool_call(&self, tool_name: &str, outcome: &'static str, duration: Duration) {
        let Some(inner) = &self.inner else {
            return;
        };
        let labels = ToolLabels {
            bot: self.bot.clone(),
            tool_name: tool_name.to_string(),
            outcome: outcome.to_string(),
        };
        inner.tool_calls.get_or_create(&labels).inc();
        inner
            .tool_duration
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
struct LlmLabels {
    bot: String,
    outcome: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ToolLabels {
    bot: String,
    tool_name: String,
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
