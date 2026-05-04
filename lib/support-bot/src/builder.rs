use crate::config::SupportBotConfig;
use crate::debug::DebugCommandHandler;
use crate::error::Result;
use crate::handler::SupportBotHandler;
use crate::instructions::InstructionRepository;
use crate::llm::LlmClient;
use crate::tools::{
    register_default_workflow_tools, register_remote_mcp_tools, SupportTool, ToolRegistry,
};
use std::sync::Arc;

pub const DEFAULT_SUPPORT_SYSTEM_PROMPT: &str = concat!(
    "You are a technical support assistant for Mattermost-based services.\n",
    "Use concise, actionable, and safe guidance.\n",
    "Rule: before giving diagnostic procedures or operational guidance, use the ",
    "`instructions` tool to list and load relevant instructions.\n",
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

        Ok(handler)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        EngineerNotificationConfig, EngineerNotificationTarget, InstructionConfig, LlmConfig,
        SupportBotLimits, SupportRouteConfig, ToolConfig,
    };
    use crate::instructions::{InstructionDocument, InstructionManifest};
    use crate::llm::{ChatMessage, LlmRequest, LlmResponse};
    use async_trait::async_trait;
    use std::path::PathBuf;
    use std::time::Duration;

    struct StaticLlm;

    #[async_trait]
    impl LlmClient for StaticLlm {
        async fn complete(&self, _request: LlmRequest) -> crate::Result<LlmResponse> {
            Ok(LlmResponse {
                message: ChatMessage::assistant("ok"),
                tool_calls: Vec::new(),
            })
        }
    }

    fn test_config() -> SupportBotConfig {
        SupportBotConfig {
            system_prompt: DEFAULT_SUPPORT_SYSTEM_PROMPT.to_string(),
            llm: LlmConfig {
                base_url: "http://localhost:11434".to_string(),
                api_key: None,
                model: "gpt-test".to_string(),
                timeout: Duration::from_secs(5),
            },
            instructions: InstructionConfig {
                root_path: PathBuf::from("."),
                max_context_instructions: 5,
                max_instruction_bytes: 4096,
            },
            tools: ToolConfig::default(),
            limits: SupportBotLimits::default(),
            routes: SupportRouteConfig::default(),
            engineer_notifications: EngineerNotificationConfig {
                target: EngineerNotificationTarget::SameThread,
            },
        }
    }

    #[tokio::test]
    async fn builder_registers_instruction_repository_and_defaults() {
        let llm = Arc::new(StaticLlm);
        let repo = InstructionRepository::new(
            ".",
            InstructionManifest {
                documents: vec![InstructionDocument {
                    id: "diag".to_string(),
                    path: "diag.md".into(),
                    title: "Diagnostics".to_string(),
                    summary: "Summary".to_string(),
                    parent: None,
                    tags: Vec::new(),
                }],
            },
        );

        SupportBotBuilder::new("support", test_config(), llm)
            .with_instruction_repository(repo)
            .unwrap()
            .build()
            .await
            .unwrap();
    }

    #[test]
    fn default_system_prompt_requires_instruction_tool() {
        assert!(DEFAULT_SUPPORT_SYSTEM_PROMPT.contains("`instructions` tool"));
        assert!(DEFAULT_SUPPORT_SYSTEM_PROMPT.contains("не выдумывай"));
    }
}
