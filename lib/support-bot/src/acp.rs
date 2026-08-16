use crate::error::{Result, SupportBotError};
use agent_client_protocol::schema::v1::{
    ContentBlock, ContentChunk, InitializeRequest, LoadSessionRequest, NewSessionRequest,
    PromptRequest, RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
    SessionNotification, SessionUpdate,
};
use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Agent, ConnectionTo};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

const QWEN_SUPPORT_SYSTEM_PROMPT: &str =
    "You are a support runtime. Follow the local /support skill for every prompt. Return only the final user-facing response.";

#[derive(Debug, Clone)]
pub struct QwenAcpConfig {
    pub executable: PathBuf,
    pub cwd: PathBuf,
    pub model: Option<String>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpPrompt {
    pub thread_id: String,
    pub post_id: String,
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpTurn {
    pub session_id: String,
    pub response: String,
    pub stop_reason: String,
    pub session_recovered: bool,
}

#[async_trait]
pub trait AcpRuntime: Send + Sync {
    async fn prompt(&self, prompt: AcpPrompt) -> Result<AcpTurn>;
}

#[derive(Clone)]
pub struct QwenAcpRuntime {
    commands: mpsc::Sender<RuntimeCommand>,
}

struct RuntimeCommand {
    prompt: AcpPrompt,
    response: oneshot::Sender<Result<AcpTurn>>,
}

impl QwenAcpRuntime {
    pub async fn start(mut config: QwenAcpConfig) -> Result<Self> {
        if !config.cwd.is_absolute() {
            config.cwd = std::env::current_dir()
                .map_err(SupportBotError::internal)?
                .join(&config.cwd);
        }
        let (commands, receiver) = mpsc::channel(16);
        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            if let Err(error) = run_qwen(config, receiver, ready_tx).await {
                tracing::error!(error = %error, "support-bot: Qwen ACP runtime stopped");
            }
        });

        ready_rx
            .await
            .map_err(|_| SupportBotError::Acp("Qwen ACP initialization failed".to_string()))??;
        Ok(Self { commands })
    }
}

#[async_trait]
impl AcpRuntime for QwenAcpRuntime {
    async fn prompt(&self, prompt: AcpPrompt) -> Result<AcpTurn> {
        let (response, receiver) = oneshot::channel();
        self.commands
            .send(RuntimeCommand { prompt, response })
            .await
            .map_err(|_| SupportBotError::Acp("Qwen ACP runtime is unavailable".to_string()))?;
        receiver
            .await
            .map_err(|_| SupportBotError::Acp("Qwen ACP runtime stopped".to_string()))?
    }
}

async fn run_qwen(
    config: QwenAcpConfig,
    mut commands: mpsc::Receiver<RuntimeCommand>,
    ready: oneshot::Sender<Result<()>>,
) -> Result<()> {
    let mut process = AcpAgentConfig::new(&config.executable).args([
        "--acp",
        "--system-prompt",
        QWEN_SUPPORT_SYSTEM_PROMPT,
    ]);
    if let Some(model) = &config.model {
        process = process.args(["--model", model]);
    }
    let agent = AcpAgent::new(process);
    let output = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    let notification_output = Arc::clone(&output);

    let connection_result = agent_client_protocol::Client
        .builder()
        .name("mmbot-support")
        .on_receive_notification(
            async move |notification: SessionNotification, _connection| {
                if let SessionUpdate::AgentMessageChunk(ContentChunk {
                    content: ContentBlock::Text(text),
                    ..
                }) = notification.update
                {
                    if let Ok(mut output) = notification_output.lock() {
                        output
                            .entry(notification.session_id.to_string())
                            .or_default()
                            .push_str(&text.text);
                    }
                }
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
        )
        .on_receive_request(
            async move |_: RequestPermissionRequest, responder, _connection| {
                responder.respond(RequestPermissionResponse::new(
                    RequestPermissionOutcome::Cancelled,
                ))
            },
            agent_client_protocol::on_receive_request!(),
        )
        .connect_with(agent, |connection: ConnectionTo<Agent>| async move {
            let initialized = connection
                .send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await
                .map_err(acp_error);
            let load_session = match initialized {
                Ok(response) => {
                    let load_session = response.agent_capabilities.load_session;
                    let _ = ready.send(Ok(()));
                    load_session
                }
                Err(error) => {
                    let _ = ready.send(Err(error));
                    return Ok(());
                }
            };
            let mut live_sessions = HashSet::new();

            while let Some(command) = commands.recv().await {
                let result = tokio::time::timeout(
                    config.timeout,
                    run_prompt(
                        &connection,
                        &config.cwd,
                        &output,
                        &mut live_sessions,
                        load_session,
                        command.prompt,
                    ),
                )
                .await
                .map_err(|_| SupportBotError::Acp("Qwen ACP prompt timed out".to_string()))
                .and_then(|result| result);
                let _ = command.response.send(result);
            }
            Ok(())
        })
        .await;

    connection_result.map_err(acp_error)
}

async fn run_prompt(
    connection: &ConnectionTo<Agent>,
    cwd: &PathBuf,
    output: &Arc<Mutex<HashMap<String, String>>>,
    live_sessions: &mut HashSet<String>,
    load_session: bool,
    prompt: AcpPrompt,
) -> Result<AcpTurn> {
    let requested_session = prompt.session_id.clone().filter(|id| !id.trim().is_empty());
    let mut session_recovered = false;
    let session_id = if let Some(session_id) = requested_session {
        if live_sessions.contains(&session_id) {
            session_id
        } else if load_session
            && connection
                .send_request(LoadSessionRequest::new(session_id.clone(), cwd))
                .block_task()
                .await
                .is_ok()
        {
            live_sessions.insert(session_id.clone());
            session_id
        } else {
            session_recovered = true;
            new_session(connection, cwd, live_sessions).await?
        }
    } else {
        new_session(connection, cwd, live_sessions).await?
    };

    output
        .lock()
        .map_err(|_| SupportBotError::Acp("ACP output buffer is poisoned".to_string()))?
        .insert(session_id.clone(), String::new());

    let response = connection
        .send_request(PromptRequest::new(
            session_id.clone(),
            vec![format_prompt(&prompt).into()],
        ))
        .block_task()
        .await
        .map_err(acp_error)?;
    // Qwen may resolve prompt() just before its final notification callback runs.
    let mut text = String::new();
    for _ in 0..20 {
        text = output
            .lock()
            .map_err(|_| SupportBotError::Acp("ACP output buffer is poisoned".to_string()))?
            .get(&session_id)
            .cloned()
            .unwrap_or_default();
        if !text.trim().is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    output
        .lock()
        .map_err(|_| SupportBotError::Acp("ACP output buffer is poisoned".to_string()))?
        .remove(&session_id);
    if text.trim().is_empty() {
        return Err(SupportBotError::Acp(
            "Qwen ACP returned an empty final response".to_string(),
        ));
    }

    Ok(AcpTurn {
        session_id,
        response: text,
        stop_reason: format!("{:?}", response.stop_reason).to_ascii_lowercase(),
        session_recovered,
    })
}

async fn new_session(
    connection: &ConnectionTo<Agent>,
    cwd: &PathBuf,
    live_sessions: &mut HashSet<String>,
) -> Result<String> {
    let response = connection
        .send_request(NewSessionRequest::new(cwd))
        .block_task()
        .await
        .map_err(acp_error)?;
    let session_id = response.session_id.to_string();
    live_sessions.insert(session_id.clone());
    Ok(session_id)
}

pub(crate) fn format_prompt(prompt: &AcpPrompt) -> String {
    format!(
        "/support\n\nMattermost thread: {}\nMattermost post: {}\nUser message:\n{}",
        prompt.thread_id, prompt.post_id, prompt.message
    )
}

fn acp_error(error: agent_client_protocol::Error) -> SupportBotError {
    SupportBotError::Acp(error.to_string())
}

#[cfg(test)]
#[path = "tests/acp.rs"]
mod tests;
