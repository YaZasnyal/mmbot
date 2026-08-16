#[derive(Debug, Clone)]
pub struct SupportBotConfig {
    pub routes: SupportRouteConfig,
}

#[derive(Debug, Clone, Default)]
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
