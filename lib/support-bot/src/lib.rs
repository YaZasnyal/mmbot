//! Layer 4 support-bot primitives.
//!
//! This crate is an experimental skeleton for Mattermost support bots built on
//! top of `thread-bot`.

pub mod config;
pub mod debug;
pub mod error;
pub mod instructions;
pub mod llm;
pub mod notifier;
pub mod state;
pub mod tools;

pub use async_trait::async_trait;
pub use config::{
    DebugCommandConfig, EngineerNotificationConfig, EngineerNotificationTarget, InstructionConfig,
    LlmConfig, RemoteMcpEndpoint, SupportBotConfig, SupportBotLimits, SupportRouteConfig,
    ToolConfig,
};
pub use debug::{DebugCommand, DebugCommandHandler, DebugCommandMatch, DebugResponse};
pub use error::{Result, SupportBotError};
pub use instructions::{
    InstructionDocument, InstructionManifest, InstructionRepository, LoadedInstruction,
};
pub use llm::{
    ChatMessage, ChatRole, LlmClient, LlmRequest, LlmResponse, OpenAiChatCompletionsClient,
};
pub use notifier::SupportNotifier;
pub use state::{EngineerThreadRef, SupportHistoryItem, SupportThreadState};
pub use tools::{
    SupportAction, SupportTool, ToolCall, ToolContext, ToolExecutionOutcome, ToolKind,
    ToolRegistry, ToolResult, ToolSpec,
};
