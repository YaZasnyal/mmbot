use anyhow::{Context, Result};
use mattermost_api::apis::configuration::Configuration;
use mattermost_bot::{Bot, MattermostBotMetrics, tokio_graceful};
use std::sync::Arc;
use std::time::Duration;
use support_bot::{
    FirstMessageTextAdmissionHook, QwenAcpConfig, QwenAcpRuntime, SupportBotBuilder,
    SupportBotConfig, SupportBotMetrics, SupportRouteConfig,
};
use thread_bot::{PgThreadStore, ThreadBotMetrics, ThreadBotPlugin, ThreadStore};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = load_support_config()?;
    let runtime = Arc::new(
        QwenAcpRuntime::start(QwenAcpConfig {
            executable: read_env("SUPPORT_QWEN_EXECUTABLE", "qwen")?.into(),
            cwd: read_env("SUPPORT_QWEN_CWD", ".")?.into(),
            model: std::env::var("SUPPORT_QWEN_MODEL")
                .ok()
                .filter(|model| !model.trim().is_empty()),
            timeout: Duration::from_secs(read_env("SUPPORT_QWEN_TIMEOUT_SECS", "120")?.parse()?),
        })
        .await?,
    );

    let bot_name = "support_bot";
    let mut _metrics_registry = support_bot::prometheus_client::registry::Registry::default();
    let mattermost_metrics = MattermostBotMetrics::register(&mut _metrics_registry);
    let thread_metrics = ThreadBotMetrics::register(&mut _metrics_registry);
    let support_metrics = SupportBotMetrics::register(&mut _metrics_registry);

    let mut builder = SupportBotBuilder::new(bot_name, config.clone(), runtime)
        .with_metrics(support_metrics.for_bot(bot_name));
    let admission_required_texts = read_csv_env("SUPPORT_ADMISSION_REQUIRED_TEXTS");
    if !admission_required_texts.is_empty() {
        builder = builder.with_admission_hook(Arc::new(FirstMessageTextAdmissionHook::new(
            admission_required_texts,
        )));
    }
    let handler = builder.build();

    let store: Arc<dyn ThreadStore> = Arc::new(
        PgThreadStore::connect(
            thread_bot::sqlx::postgres::PgPoolOptions::new()
                .max_connections(read_env("THREAD_BOT_DB_MAX_CONNECTIONS", "5")?.parse()?)
                .acquire_timeout(Duration::from_secs(
                    read_env("THREAD_BOT_DB_ACQUIRE_TIMEOUT_SECS", "5")?.parse()?,
                )),
            database_connect_options()?,
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
    let engineer_channel_id = read_required_env("SUPPORT_ENGINEER_CHANNEL_ID")?;

    Ok(SupportBotConfig {
        routes: SupportRouteConfig {
            user_channel_ids: read_csv_env("SUPPORT_USER_CHANNEL_IDS"),
            engineer_channel_id,
            ..SupportRouteConfig::default()
        },
    })
}

fn read_required_env(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("missing required env var: {name}"))
}

fn read_env(name: &str, default: &str) -> Result<String> {
    Ok(std::env::var(name).unwrap_or_else(|_| default.to_string()))
}

fn database_connect_options() -> Result<Vec<thread_bot::sqlx::postgres::PgConnectOptions>> {
    let options = read_env(
        "THREAD_BOT_DATABASE_URL",
        "postgres://test:test@localhost:5433/thread_bot_test",
    )?
    .parse::<thread_bot::sqlx::postgres::PgConnectOptions>()?;
    let hosts = read_csv_env("THREAD_BOT_DATABASE_HOSTS");

    Ok(if hosts.is_empty() {
        vec![options]
    } else {
        hosts
            .into_iter()
            .map(|host| options.clone().host(&host))
            .collect()
    })
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
