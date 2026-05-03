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
    WaitForUser { reason: Option<String> },
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
