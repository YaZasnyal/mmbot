use crate::error::{Result, SupportBotError};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub kind: ToolKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolResult {
    pub call_id: String,
    pub content: serde_json::Value,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolExecutionOutcome {
    ToolResult(ToolResult),
    Action(SupportAction),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    ReadOnly,
    Notify,
    Workflow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SupportAction {
    SendUserMessage { message: String },
    NotifyEngineer { message: String },
    Noop { reason: Option<String> },
    FinishRequest { summary: Option<String> },
}

#[derive(Clone)]
pub struct ToolContext {
    pub thread: Option<Arc<thread_bot::Thread>>,
}

impl ToolContext {
    pub fn new(thread: Arc<thread_bot::Thread>) -> Self {
        Self {
            thread: Some(thread),
        }
    }

    pub fn without_thread() -> Self {
        Self { thread: None }
    }
}

#[async_trait]
pub trait SupportTool: Send + Sync {
    fn spec(&self) -> ToolSpec;

    async fn call(&self, ctx: ToolContext, call: ToolCall) -> Result<ToolExecutionOutcome>;
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn SupportTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T>(&mut self, tool: T) -> Result<()>
    where
        T: SupportTool + 'static,
    {
        let spec = tool.spec();
        if self.tools.contains_key(&spec.name) {
            return Err(SupportBotError::Config(format!(
                "duplicate tool name: {}",
                spec.name
            )));
        }
        self.tools.insert(spec.name, Arc::new(tool));
        Ok(())
    }

    pub fn specs(&self) -> Vec<ToolSpec> {
        let mut specs = self
            .tools
            .values()
            .map(|tool| tool.spec())
            .collect::<Vec<_>>();
        specs.sort_by(|left, right| left.name.cmp(&right.name));
        specs
    }

    pub async fn call(&self, ctx: ToolContext, call: ToolCall) -> Result<ToolExecutionOutcome> {
        let tool = self
            .tools
            .get(&call.name)
            .ok_or_else(|| SupportBotError::UnknownTool(call.name.clone()))?;

        tool.call(ctx, call).await
    }
}

pub fn register_default_workflow_tools(registry: &mut ToolRegistry) -> Result<()> {
    for spec in WORKFLOW_TOOL_SPECS {
        registry.register(WorkflowTool::new(spec))?;
    }
    Ok(())
}
pub use crate::remote_mcp::register_remote_mcp_tools;

struct WorkflowTool {
    spec: &'static WorkflowToolSpec,
}

impl WorkflowTool {
    fn new(spec: &'static WorkflowToolSpec) -> Self {
        Self { spec }
    }
}

struct WorkflowToolSpec {
    name: &'static str,
    description: &'static str,
    input_schema: fn() -> serde_json::Value,
}

const WORKFLOW_TOOL_SPECS: &[WorkflowToolSpec] = &[
    WorkflowToolSpec {
        name: "send_user_message",
        description: "Send a message to the user in the support thread.",
        input_schema: workflow_message_schema,
    },
    WorkflowToolSpec {
        name: "notify_engineer",
        description: "Send diagnostic context or a question to the engineer channel.",
        input_schema: workflow_message_schema,
    },
    WorkflowToolSpec {
        name: "noop",
        description: "Stop this turn without sending a message to the user.",
        input_schema: noop_schema,
    },
    WorkflowToolSpec {
        name: "finish_request",
        description: "Mark the support request as finished.",
        input_schema: finish_request_schema,
    },
];

#[async_trait]
impl SupportTool for WorkflowTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: self.spec.name.to_string(),
            description: self.spec.description.to_string(),
            input_schema: (self.spec.input_schema)(),
            kind: ToolKind::Workflow,
        }
    }

    async fn call(&self, _ctx: ToolContext, call: ToolCall) -> Result<ToolExecutionOutcome> {
        let action = match call.name.as_str() {
            "send_user_message" => {
                let args: WorkflowMessageArgs = parse_workflow_args(call.arguments)?;
                SupportAction::SendUserMessage {
                    message: non_empty_string(args.message, "message")?,
                }
            }
            "notify_engineer" => {
                let args: WorkflowMessageArgs = parse_workflow_args(call.arguments)?;
                SupportAction::NotifyEngineer {
                    message: non_empty_string(args.message, "message")?,
                }
            }
            "noop" => {
                let args: NoopArgs = parse_workflow_args(call.arguments)?;
                SupportAction::Noop {
                    reason: args.reason.and_then(non_empty_optional_string),
                }
            }
            "finish_request" => {
                let args: FinishRequestArgs = parse_workflow_args(call.arguments)?;
                SupportAction::FinishRequest {
                    summary: args.summary.and_then(non_empty_optional_string),
                }
            }
            _ => {
                return Err(SupportBotError::Tool(format!(
                    "unsupported workflow tool: {}",
                    call.name
                )));
            }
        };

        Ok(ToolExecutionOutcome::Action(action))
    }
}

fn workflow_message_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "message": { "type": "string" }
        },
        "required": ["message"],
        "additionalProperties": false
    })
}

fn noop_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "reason": { "type": "string" }
        },
        "additionalProperties": false
    })
}

fn finish_request_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "summary": { "type": "string" }
        },
        "additionalProperties": false
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowMessageArgs {
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NoopArgs {
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FinishRequestArgs {
    #[serde(default)]
    summary: Option<String>,
}

fn parse_workflow_args<T>(arguments: serde_json::Value) -> Result<T>
where
    T: DeserializeOwned,
{
    serde_json::from_value(arguments)
        .map_err(|error| SupportBotError::Tool(format!("invalid workflow tool arguments: {error}")))
}

fn non_empty_string(value: String, key: &str) -> Result<String> {
    if value.trim().is_empty() {
        return Err(SupportBotError::Tool(format!(
            "missing required string argument: {key}"
        )));
    }
    Ok(value)
}

fn non_empty_optional_string(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
#[path = "tests/tools.rs"]
mod tests;
