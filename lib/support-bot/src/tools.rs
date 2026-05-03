use crate::error::{Result, SupportBotError};
use async_trait::async_trait;
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
    registry.register(WorkflowTool::send_user_message())?;
    registry.register(WorkflowTool::notify_engineer())?;
    registry.register(WorkflowTool::finish_request())?;
    Ok(())
}
pub use crate::remote_mcp::register_remote_mcp_tools;

struct WorkflowTool {
    name: &'static str,
    description: &'static str,
    kind: WorkflowToolKind,
}

impl WorkflowTool {
    fn send_user_message() -> Self {
        Self {
            name: "send_user_message",
            description: "Send a message to the user in the support thread.",
            kind: WorkflowToolKind::SendUserMessage,
        }
    }

    fn notify_engineer() -> Self {
        Self {
            name: "notify_engineer",
            description: "Send diagnostic context or a question to the engineer channel.",
            kind: WorkflowToolKind::NotifyEngineer,
        }
    }

    fn finish_request() -> Self {
        Self {
            name: "finish_request",
            description: "Mark the support request as finished.",
            kind: WorkflowToolKind::FinishRequest,
        }
    }
}

#[derive(Clone, Copy)]
enum WorkflowToolKind {
    SendUserMessage,
    NotifyEngineer,
    FinishRequest,
}

#[async_trait]
impl SupportTool for WorkflowTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: self.name.to_string(),
            description: self.description.to_string(),
            input_schema: match self.kind {
                WorkflowToolKind::SendUserMessage | WorkflowToolKind::NotifyEngineer => {
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "message": { "type": "string" }
                        },
                        "required": ["message"],
                        "additionalProperties": false
                    })
                }
                WorkflowToolKind::FinishRequest => serde_json::json!({
                    "type": "object",
                    "properties": {
                        "summary": { "type": "string" }
                    },
                    "additionalProperties": false
                }),
            },
            kind: ToolKind::Workflow,
        }
    }

    async fn call(&self, _ctx: ToolContext, call: ToolCall) -> Result<ToolExecutionOutcome> {
        let action = match self.kind {
            WorkflowToolKind::SendUserMessage => SupportAction::SendUserMessage {
                message: required_string(&call.arguments, "message")?,
            },
            WorkflowToolKind::NotifyEngineer => SupportAction::NotifyEngineer {
                message: required_string(&call.arguments, "message")?,
            },
            WorkflowToolKind::FinishRequest => SupportAction::FinishRequest {
                summary: optional_string(&call.arguments, "summary"),
            },
        };

        Ok(ToolExecutionOutcome::Action(action))
    }
}

fn required_string(arguments: &serde_json::Value, key: &str) -> Result<String> {
    arguments
        .get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| SupportBotError::Tool(format!("missing required string argument: {key}")))
}

fn optional_string(arguments: &serde_json::Value, key: &str) -> Option<String> {
    arguments
        .get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_default_workflow_tools() {
        let mut registry = ToolRegistry::new();
        register_default_workflow_tools(&mut registry).unwrap();

        let names = registry
            .specs()
            .into_iter()
            .map(|spec| spec.name)
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec!["finish_request", "notify_engineer", "send_user_message"]
        );
    }

    #[tokio::test]
    async fn workflow_tool_returns_support_action() {
        let tool = WorkflowTool::send_user_message();
        let outcome = tool
            .call(
                ToolContext::without_thread(),
                ToolCall {
                    id: "call-1".to_string(),
                    name: "send_user_message".to_string(),
                    arguments: serde_json::json!({ "message": "hello" }),
                },
            )
            .await
            .unwrap();

        assert_eq!(
            outcome,
            ToolExecutionOutcome::Action(SupportAction::SendUserMessage {
                message: "hello".to_string()
            })
        );
    }
}
