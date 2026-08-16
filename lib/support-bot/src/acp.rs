use crate::error::{Result, SupportBotError};
use agent_client_protocol::schema::v1::{
    ContentBlock, ContentChunk, InitializeRequest, LoadSessionRequest, NewSessionRequest,
    PromptRequest, RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
    SessionNotification, SessionUpdate,
};
use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Agent, ConnectionTo};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

const QWEN_SUPPORT_SYSTEM_PROMPT: &str = r#"You are a support runtime. Follow the local /support skill for every prompt.
Never use ask_user_question or request interactive input. To ask for clarification, return the question in message and end the turn.
Return only one JSON object: {"message":"user-facing Markdown","action":"none|ignore|finish","reason":null|string}. Use ignore when the thread is not a support request and finish when the support conversation is complete."#;

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
    pub action: AcpTurnAction,
    pub reason: Option<String>,
    pub stop_reason: String,
    pub session_recovered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AcpTurnAction {
    #[default]
    None,
    Ignore,
    Finish,
}

impl AcpTurnAction {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Ignore => "ignore",
            Self::Finish => "finish",
        }
    }
}

#[derive(Deserialize)]
struct AcpResponse {
    message: String,
    #[serde(default)]
    action: String,
    #[serde(default)]
    reason: Option<String>,
}

impl AcpResponse {
    fn action(&self) -> AcpTurnAction {
        match self.action.as_str() {
            "ignore" => AcpTurnAction::Ignore,
            "finish" => AcpTurnAction::Finish,
            _ => AcpTurnAction::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpSessionTrace {
    pub events: Vec<AcpSessionEvent>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpSessionEvent {
    pub timestamp: String,
    pub kind: String,
    pub title: String,
    pub body: String,
}

#[async_trait]
pub trait AcpRuntime: Send + Sync {
    async fn prompt(&self, prompt: AcpPrompt) -> Result<AcpTurn>;

    async fn session_trace(&self, _session_id: &str) -> Result<Option<AcpSessionTrace>> {
        Ok(None)
    }
}

#[derive(Clone)]
pub struct QwenAcpRuntime {
    commands: mpsc::Sender<RuntimeCommand>,
    projects_dir: PathBuf,
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
        let projects_dir = qwen_runtime_dir()?.join("projects");
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
        Ok(Self {
            commands,
            projects_dir,
        })
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

    async fn session_trace(&self, session_id: &str) -> Result<Option<AcpSessionTrace>> {
        if session_id.is_empty()
            || !session_id
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        {
            return Err(SupportBotError::Acp("invalid Qwen session ID".to_string()));
        }
        let Some(path) = find_session_file(&self.projects_dir, session_id)? else {
            return Ok(None);
        };
        match std::fs::read_to_string(path) {
            Ok(jsonl) => Ok(Some(parse_session_trace(&jsonl))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(SupportBotError::internal(error)),
        }
    }
}

fn find_session_file(projects_dir: &Path, session_id: &str) -> Result<Option<PathBuf>> {
    let projects = match std::fs::read_dir(projects_dir) {
        Ok(projects) => projects,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(SupportBotError::internal(error)),
    };
    // ponytail: linear project scan; index sessions only if debug exports become frequent.
    for project in projects.flatten() {
        let path = project
            .path()
            .join("chats")
            .join(format!("{session_id}.jsonl"));
        if path.is_file() {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn qwen_runtime_dir() -> Result<PathBuf> {
    let configured = std::env::var_os("QWEN_RUNTIME_DIR")
        .or_else(|| std::env::var_os("QWEN_HOME"))
        .map(PathBuf::from);
    let Some(path) = configured else {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(".qwen"))
            .ok_or_else(|| SupportBotError::Acp("cannot locate Qwen runtime directory".into()));
    };
    if path.is_absolute() {
        return Ok(path);
    }
    if path == Path::new("~") || path.starts_with("~/") {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| SupportBotError::Acp("cannot expand Qwen runtime directory".into()))?;
        return Ok(home.join(path.strip_prefix("~").unwrap_or(Path::new(""))));
    }
    Ok(std::env::current_dir()
        .map_err(SupportBotError::internal)?
        .join(path))
}

fn parse_session_trace(jsonl: &str) -> AcpSessionTrace {
    let mut trace = AcpSessionTrace {
        events: Vec::new(),
        warnings: Vec::new(),
    };
    for (index, line) in jsonl
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
    {
        let record: serde_json::Value = match serde_json::from_str(line) {
            Ok(record) => record,
            Err(error) => {
                trace
                    .warnings
                    .push(format!("line {} could not be decoded: {error}", index + 1));
                continue;
            }
        };
        append_session_events(&mut trace.events, &record);
    }
    trace
}

fn append_session_events(events: &mut Vec<AcpSessionEvent>, record: &serde_json::Value) {
    let timestamp = record["timestamp"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let record_type = record["type"].as_str().unwrap_or("session");
    if let Some(parts) = record["message"]["parts"].as_array() {
        for part in parts {
            let (kind, title, body) = if let Some(text) = part["text"].as_str() {
                let thought = part["thought"].as_bool().unwrap_or(false);
                (
                    if thought { "reasoning" } else { record_type },
                    if thought {
                        "Agent reasoning"
                    } else {
                        record_type
                    },
                    text.to_string(),
                )
            } else if let Some(call) = part.get("functionCall") {
                (
                    "tool_call",
                    call["name"].as_str().unwrap_or("Tool call"),
                    pretty_redacted(&call["args"]),
                )
            } else if let Some(response) = part.get("functionResponse") {
                (
                    "tool_result",
                    response["name"].as_str().unwrap_or("Tool result"),
                    pretty_redacted(&response["response"]),
                )
            } else {
                continue;
            };
            events.push(AcpSessionEvent {
                timestamp: timestamp.clone(),
                kind: kind.to_string(),
                title: title.to_string(),
                body,
            });
        }
    } else if let Some(payload) = record.get("systemPayload") {
        events.push(AcpSessionEvent {
            timestamp,
            kind: "system".to_string(),
            title: record["subtype"]
                .as_str()
                .unwrap_or("System event")
                .to_string(),
            body: pretty_redacted(payload),
        });
    }
}

fn pretty_redacted(value: &serde_json::Value) -> String {
    let mut value = value.clone();
    redact_secrets(&mut value);
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "<unavailable>".to_string())
}

fn redact_secrets(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(object) => {
            for (key, value) in object {
                let key = key.to_ascii_lowercase();
                if [
                    "password",
                    "passwd",
                    "secret",
                    "authorization",
                    "cookie",
                    "api_key",
                    "apikey",
                    "access_token",
                    "refresh_token",
                    "bearer_token",
                    "credential",
                    "credentials",
                ]
                .contains(&key.as_str())
                {
                    *value = serde_json::Value::String("[REDACTED]".to_string());
                } else {
                    redact_secrets(value);
                }
            }
        }
        serde_json::Value::Array(values) => values.iter_mut().for_each(redact_secrets),
        _ => {}
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

    let parsed = parse_response(&text);
    let action = parsed.action();
    Ok(AcpTurn {
        session_id,
        response: parsed.message,
        action,
        reason: parsed.reason,
        stop_reason: format!("{:?}", response.stop_reason).to_ascii_lowercase(),
        session_recovered,
    })
}

fn parse_response(text: &str) -> AcpResponse {
    serde_json::from_str(text).unwrap_or_else(|_| AcpResponse {
        message: text.to_string(),
        action: String::new(),
        reason: None,
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
