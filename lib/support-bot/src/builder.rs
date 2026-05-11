use crate::config::SupportBotConfig;
use crate::debug::DebugCommandHandler;
use crate::error::Result;
use crate::handler::SupportBotHandler;
use crate::instructions::InstructionRepository;
use crate::llm::LlmClient;
use crate::metrics::SupportBotMetricsHandle;
use crate::tools::{
    register_default_workflow_tools, register_remote_mcp_tools, SupportTool, ToolRegistry,
};
use std::sync::Arc;

pub const DEFAULT_SUPPORT_SYSTEM_PROMPT: &str = concat!(
    "You are a technical support assistant for Mattermost-based services.\n",
    "Use concise, actionable, and safe guidance.\n",
    "Rule: before giving diagnostic procedures or operational guidance, use the ",
    "`instructions` tool to load `/index`, then follow Markdown links to relevant instructions.\n",
    "Do not invent runbooks or commands when instructions were not loaded. ",
    "If instructions are missing, say so explicitly and ask for escalation/context.\n",
    "Follow this policy strictly: \"используй инструкции через tool, не выдумывай\"."
);

pub struct SupportBotBuilder {
    id: &'static str,
    config: SupportBotConfig,
    llm: Arc<dyn LlmClient>,
    tools: ToolRegistry,
    debug_handler: Option<Arc<dyn DebugCommandHandler>>,
    system_prompt: String,
    include_default_workflow_tools: bool,
    include_remote_mcp_tools: bool,
    metrics: Option<SupportBotMetricsHandle>,
}

impl SupportBotBuilder {
    pub fn new(id: &'static str, config: SupportBotConfig, llm: Arc<dyn LlmClient>) -> Self {
        let system_prompt = config.system_prompt.clone();
        Self {
            id,
            config,
            llm,
            tools: ToolRegistry::new(),
            debug_handler: None,
            system_prompt,
            include_default_workflow_tools: true,
            include_remote_mcp_tools: true,
            metrics: None,
        }
    }

    pub fn with_system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = system_prompt.into();
        self
    }

    pub fn with_debug_handler(mut self, handler: Arc<dyn DebugCommandHandler>) -> Self {
        self.debug_handler = Some(handler);
        self
    }

    pub fn with_default_workflow_tools(mut self, enabled: bool) -> Self {
        self.include_default_workflow_tools = enabled;
        self
    }

    pub fn with_remote_mcp_tools(mut self, enabled: bool) -> Self {
        self.include_remote_mcp_tools = enabled;
        self
    }

    pub fn with_metrics(mut self, metrics: SupportBotMetricsHandle) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_tool<T>(mut self, tool: T) -> Result<Self>
    where
        T: SupportTool + 'static,
    {
        self.tools.register(tool)?;
        Ok(self)
    }

    pub fn with_instruction_repository(
        mut self,
        repository: InstructionRepository,
    ) -> Result<Self> {
        self.tools.register(repository)?;
        Ok(self)
    }

    pub async fn build(mut self) -> Result<SupportBotHandler> {
        if self.include_default_workflow_tools {
            register_default_workflow_tools(&mut self.tools)?;
        }

        if self.include_remote_mcp_tools {
            register_remote_mcp_tools(&mut self.tools, &self.config.tools).await?;
        }

        let mut handler = SupportBotHandler::new(
            self.id,
            self.config,
            self.llm,
            Arc::new(self.tools),
            self.system_prompt,
        );

        if let Some(debug_handler) = self.debug_handler {
            handler = handler.with_debug_handler(debug_handler);
        }

        if let Some(metrics) = self.metrics {
            handler = handler.with_metrics(metrics);
        }

        Ok(handler)
    }
}

#[cfg(test)]
#[path = "tests/builder.rs"]
mod tests;
