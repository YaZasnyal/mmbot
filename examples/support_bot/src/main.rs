use anyhow::{Context, Result};
use mattermost_api::apis::configuration::Configuration;
use mattermost_bot::{Bot, MattermostBotMetrics, tokio_graceful};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use support_bot::{
    DEFAULT_SUPPORT_SYSTEM_PROMPT, FirstMessageTextAdmissionHook, InstructionConfig,
    InstructionRepository, LlmConfig, OpenAiChatCompletionsClient, SupportBotBuilder,
    SupportBotConfig, SupportBotLimits, SupportBotMetrics, SupportRouteConfig, ToolConfig,
};
use thread_bot::{PgThreadStore, ThreadBotMetrics, ThreadBotPlugin, ThreadStore};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = load_support_config()?;
    let instruction_repo = load_instruction_repository(&config.instructions)?;

    let llm = Arc::new(OpenAiChatCompletionsClient::new(config.llm.clone())?);

    let bot_name = "support_bot";
    let mut _metrics_registry = support_bot::prometheus_client::registry::Registry::default();
    let mattermost_metrics = MattermostBotMetrics::register(&mut _metrics_registry);
    let thread_metrics = ThreadBotMetrics::register(&mut _metrics_registry);
    let support_metrics = SupportBotMetrics::register(&mut _metrics_registry);

    let mut builder = SupportBotBuilder::new(bot_name, config.clone(), llm)
        .with_instruction_repository(instruction_repo)?
        .with_metrics(support_metrics.for_bot(bot_name));
    let admission_required_texts = read_csv_env("SUPPORT_ADMISSION_REQUIRED_TEXTS");
    if !admission_required_texts.is_empty() {
        builder = builder.with_admission_hook(Arc::new(FirstMessageTextAdmissionHook::new(
            admission_required_texts,
        )));
    }
    let handler = builder.build().await?;

    let store: Arc<dyn ThreadStore> = Arc::new(
        PgThreadStore::new(
            thread_bot::sqlx::postgres::PgPoolOptions::new()
                .max_connections(read_env("THREAD_BOT_DB_MAX_CONNECTIONS", "5")?.parse()?)
                .connect(&read_env(
                    "THREAD_BOT_DATABASE_URL",
                    "postgres://test:test@localhost:5433/thread_bot_test",
                )?)
                .await?,
        )
        .await?,
    );

    let plugin = ThreadBotPlugin::new(handler, store)
        .with_metrics(thread_metrics.for_bot(bot_name, bot_name));

    let mm_config = Configuration {
        base_path: read_env("MM_BASE_PATH", "http://localhost:8065")?,
        bearer_access_token: Some(read_required_env("MM_BEARER_TOKEN")?),
        ..Default::default()
    };

    let mut bot = Bot::with_config(mm_config)?
        .with_metrics(mattermost_metrics.for_bot(bot_name))
        .with_plugin(plugin);

    tracing::info!("Starting support-bot example...");

    let shutdown = tokio_graceful::Shutdown::builder()
        .with_signal(tokio::signal::ctrl_c())
        .build();

    bot.run(shutdown.guard()).await;
    Ok(())
}

fn load_support_config() -> Result<SupportBotConfig> {
    let instructions_root = read_env(
        "SUPPORT_INSTRUCTIONS_ROOT",
        "examples/support_bot/instructions",
    )?;
    let llm_timeout_secs: u64 = read_env("SUPPORT_LLM_TIMEOUT_SECS", "45")?.parse()?;
    let max_tool_rounds: usize = read_env("SUPPORT_MAX_TOOL_ROUNDS", "4")?.parse()?;
    let max_tool_calls_per_round: usize =
        read_env("SUPPORT_MAX_TOOL_CALLS_PER_ROUND", "8")?.parse()?;
    let max_tool_result_bytes: usize =
        read_env("SUPPORT_MAX_TOOL_RESULT_BYTES", "16384")?.parse()?;

    let max_context_instructions: usize =
        read_env("SUPPORT_MAX_CONTEXT_INSTRUCTIONS", "5")?.parse()?;
    let max_instruction_bytes: usize =
        read_env("SUPPORT_MAX_INSTRUCTION_BYTES", "16384")?.parse()?;

    let engineer_channel_id = read_required_env("SUPPORT_ENGINEER_CHANNEL_ID")?;

    let remote_mcp_endpoints = load_remote_mcp_endpoints()?;

    Ok(SupportBotConfig {
        system_prompt: load_system_prompt()?,
        llm: LlmConfig {
            base_url: read_env("SUPPORT_LLM_BASE_URL", "http://localhost:11434")?,
            api_key: std::env::var("SUPPORT_LLM_API_KEY").ok(),
            model: read_env("SUPPORT_LLM_MODEL", "gpt-4o-mini")?,
            timeout: Duration::from_secs(llm_timeout_secs),
        },
        instructions: InstructionConfig {
            root_path: PathBuf::from(instructions_root),
            max_context_instructions,
            max_instruction_bytes,
        },
        tools: ToolConfig {
            remote_mcp_endpoints,
        },
        limits: SupportBotLimits {
            max_tool_rounds,
            max_tool_calls_per_round,
            max_tool_result_bytes,
        },
        routes: SupportRouteConfig {
            user_channel_ids: read_csv_env("SUPPORT_USER_CHANNEL_IDS"),
            engineer_channel_id,
            ..SupportRouteConfig::default()
        },
    })
}

fn load_instruction_repository(config: &InstructionConfig) -> Result<InstructionRepository> {
    for issue in InstructionRepository::lint(&config.root_path)? {
        let path = issue
            .path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string());
        let id = issue.id.as_deref().unwrap_or("-");
        tracing::error!(
            issue_kind = ?issue.kind,
            path = %path,
            id = %id,
            message = %issue.message,
            "support-bot: instruction repository lint issue"
        );
    }

    Ok(InstructionRepository::new(config.root_path.clone())?
        .with_max_instruction_bytes(config.max_instruction_bytes)
        .with_max_load_documents(config.max_context_instructions))
}

fn load_remote_mcp_endpoints() -> Result<Vec<support_bot::RemoteMcpEndpoint>> {
    let mut endpoints = Vec::new();
    for name in read_csv_env("SUPPORT_REMOTE_MCP_NAMES") {
        let key = name.to_ascii_uppercase().replace('-', "_");
        let url = std::env::var(format!("SUPPORT_REMOTE_MCP_{}_URL", key)).with_context(|| {
            format!(
                "missing SUPPORT_REMOTE_MCP_{}_URL for MCP endpoint '{}'",
                key, name
            )
        })?;

        let auth_header = std::env::var(format!("SUPPORT_REMOTE_MCP_{}_AUTH_HEADER", key)).ok();
        let timeout_secs: u64 =
            read_env(&format!("SUPPORT_REMOTE_MCP_{}_TIMEOUT_SECS", key), "30")?.parse()?;

        endpoints.push(support_bot::RemoteMcpEndpoint {
            name,
            url,
            auth_header,
            timeout: Duration::from_secs(timeout_secs),
        });
    }

    Ok(endpoints)
}

fn load_system_prompt() -> Result<String> {
    if let Ok(path) = std::env::var("SUPPORT_SYSTEM_PROMPT_FILE")
        && !path.trim().is_empty()
    {
        return std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read SUPPORT_SYSTEM_PROMPT_FILE: {path}"));
    }

    Ok(std::env::var("SUPPORT_SYSTEM_PROMPT")
        .unwrap_or_else(|_| DEFAULT_SUPPORT_SYSTEM_PROMPT.to_string()))
}

fn read_required_env(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("missing required env var: {name}"))
}

fn read_env(name: &str, default: &str) -> Result<String> {
    Ok(std::env::var(name).unwrap_or_else(|_| default.to_string()))
}

fn read_csv_env(name: &str) -> Vec<String> {
    std::env::var(name)
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
