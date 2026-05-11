use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SupportBotConfig {
    pub system_prompt: String,
    pub llm: LlmConfig,
    pub tools: ToolConfig,
    pub limits: SupportBotLimits,
    pub routes: SupportRouteConfig,
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct ToolConfig {
    pub remote_mcp_endpoints: Vec<RemoteMcpEndpoint>,
}

#[derive(Debug, Clone)]
pub struct RemoteMcpEndpoint {
    pub name: String,
    pub url: String,
    pub auth_header: Option<String>,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct SupportBotLimits {
    pub max_tool_rounds: usize,
    pub max_tool_calls_per_round: usize,
    pub max_tool_result_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct SupportRouteConfig {
    pub user_channel_ids: Vec<String>,
    pub engineer_channel_id: String,
    pub debug_commands: DebugCommandConfig,
}

#[derive(Debug, Clone)]
pub struct DebugCommandConfig {
    pub enabled: bool,
    pub prefixes: Vec<String>,
}

impl Default for DebugCommandConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefixes: vec!["/support".to_string(), "!support".to_string()],
        }
    }
}

impl Default for SupportRouteConfig {
    fn default() -> Self {
        Self {
            user_channel_ids: Vec::new(),
            engineer_channel_id: String::new(),
            debug_commands: DebugCommandConfig::default(),
        }
    }
}

impl Default for SupportBotLimits {
    fn default() -> Self {
        Self {
            max_tool_rounds: 4,
            max_tool_calls_per_round: 8,
            max_tool_result_bytes: 16 * 1024,
        }
    }
}
