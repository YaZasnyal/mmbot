use crate::acp::AcpRuntime;
use crate::admission::SupportThreadAdmissionHook;
use crate::config::SupportBotConfig;
use crate::debug::DebugCommandHandler;
use crate::handler::SupportBotHandler;
use crate::metrics::SupportBotMetricsHandle;
use std::sync::Arc;

pub struct SupportBotBuilder {
    handler: SupportBotHandler,
}

impl SupportBotBuilder {
    pub fn new(id: &'static str, config: SupportBotConfig, runtime: Arc<dyn AcpRuntime>) -> Self {
        Self {
            handler: SupportBotHandler::new(id, config, runtime),
        }
    }

    pub fn with_debug_handler(mut self, handler: Arc<dyn DebugCommandHandler>) -> Self {
        self.handler = self.handler.with_debug_handler(handler);
        self
    }

    pub fn with_admission_hook(mut self, hook: Arc<dyn SupportThreadAdmissionHook>) -> Self {
        self.handler = self.handler.with_admission_hook(hook);
        self
    }

    pub fn with_metrics(mut self, metrics: SupportBotMetricsHandle) -> Self {
        self.handler = self.handler.with_metrics(metrics);
        self
    }

    pub fn build(self) -> SupportBotHandler {
        self.handler
    }
}

#[cfg(test)]
#[path = "tests/builder.rs"]
mod tests;
