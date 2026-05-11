//! Layer 4 support-bot primitives.
//!
//! This crate is an experimental skeleton for Mattermost support bots built on
//! top of `thread-bot`.

pub mod builder;
pub mod config;
mod conversation;
pub mod debug;
mod debug_export;
pub mod error;
pub mod handler;
pub mod instructions;
pub mod llm;
mod metadata;
pub mod metrics;
pub mod notifier;
mod output;
mod remote_mcp;
pub mod state;
pub mod tools;
mod user_thread_run;
mod workflow;

#[cfg(test)]
mod testutil;

pub use async_trait::async_trait;
pub use builder::{SupportBotBuilder, DEFAULT_SUPPORT_SYSTEM_PROMPT};
pub use config::{
    DebugCommandConfig, InstructionConfig, LlmConfig, RemoteMcpEndpoint, SupportBotConfig,
    SupportBotLimits, SupportRouteConfig, ToolConfig,
};
pub use debug::{DebugCommand, DebugCommandHandler, DebugCommandMatch, DebugResponse};
pub use error::{Result, SupportBotError};
pub use handler::SupportBotHandler;
pub use instructions::{
    InstructionDocument, InstructionLintIssue, InstructionLintIssueKind, InstructionRepository,
    LoadedInstruction,
};
pub use llm::{
    ChatMessage, ChatRole, LlmClient, LlmRequest, LlmResponse, OpenAiChatCompletionsClient,
};
pub use metrics::{SupportBotMetrics, SupportBotMetricsHandle};
pub use prometheus_client;
pub use state::{SupportThreadState, SupportThreadStatus};
pub use tools::{
    register_default_workflow_tools, register_remote_mcp_tools, SupportAction, SupportTool,
    ToolCall, ToolContext, ToolExecutionOutcome, ToolKind, ToolRegistry, ToolResult, ToolSpec,
};
