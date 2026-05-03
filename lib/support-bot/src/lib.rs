//! Layer 4 support-bot primitives.
//!
//! This crate is an experimental skeleton for Mattermost support bots built on
//! top of `thread-bot`.

pub mod config;
pub mod debug;
pub mod error;
pub mod handler;
pub mod instructions;
pub mod llm;
pub mod notifier;
mod remote_mcp;
pub mod state;
pub mod tools;

#[cfg(test)]
mod testutil;

pub use async_trait::async_trait;
pub use config::{
    DebugCommandConfig, EngineerNotificationConfig, EngineerNotificationTarget, InstructionConfig,
    LlmConfig, RemoteMcpEndpoint, SupportBotConfig, SupportBotLimits, SupportRouteConfig,
    ToolConfig,
};
pub use debug::{DebugCommand, DebugCommandHandler, DebugCommandMatch, DebugResponse};
pub use error::{Result, SupportBotError};
pub use handler::SupportBotHandler;
pub use instructions::{
    InstructionDocument, InstructionManifest, InstructionRepository, LoadedInstruction,
};
pub use llm::{
    ChatMessage, ChatRole, LlmClient, LlmRequest, LlmResponse, OpenAiChatCompletionsClient,
};
pub use notifier::MattermostSupportNotifier;
pub use state::{EngineerThreadRef, SupportHistoryItem, SupportThreadState};
pub use tools::{
    register_default_workflow_tools, register_remote_mcp_tools, SupportAction, SupportTool,
    ToolCall, ToolContext, ToolExecutionOutcome, ToolKind, ToolRegistry, ToolResult, ToolSpec,
};
