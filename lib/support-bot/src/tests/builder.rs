use super::*;
use crate::config::{
    EngineerNotificationConfig, InstructionConfig, LlmConfig, SupportBotLimits,
    SupportRouteConfig, ToolConfig,
};
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
            channel_id: "engineers".to_string(),
        },
    }
}

#[tokio::test]
async fn builder_registers_instruction_repository_and_defaults() {
    let temp_dir = std::env::temp_dir().join(format!(
        "support-bot-builder-instructions-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(
        temp_dir.join("index.md"),
        "---\ntitle: Diagnostics\n---\n# Diagnostics",
    )
    .unwrap();

    let llm = Arc::new(StaticLlm);
    let repo = InstructionRepository::new(&temp_dir).unwrap();

    SupportBotBuilder::new("support", test_config(), llm)
        .with_instruction_repository(repo)
        .unwrap()
        .build()
        .await
        .unwrap();

    std::fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn default_system_prompt_requires_instruction_tool() {
    assert!(DEFAULT_SUPPORT_SYSTEM_PROMPT.contains("`instructions` tool"));
    assert!(DEFAULT_SUPPORT_SYSTEM_PROMPT.contains("не выдумывай"));
}
