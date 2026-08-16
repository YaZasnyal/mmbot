//! Layer 4 support-bot primitives.
//!
//! This crate is an experimental skeleton for Mattermost support bots built on
//! top of `thread-bot`.

pub mod acp;
pub mod admission;
pub mod builder;
pub mod config;
pub mod debug;
mod debug_export;
pub mod error;
pub mod handler;
mod metadata;
pub mod metrics;
pub mod notifier;
pub mod state;

#[cfg(test)]
mod testutil;

pub use acp::{AcpPrompt, AcpRuntime, AcpTurn, QwenAcpConfig, QwenAcpRuntime};
pub use admission::{
    FirstMessageTextAdmissionHook, SupportThreadAdmissionDecision, SupportThreadAdmissionHook,
};
pub use async_trait::async_trait;
pub use builder::SupportBotBuilder;
pub use config::{DebugCommandConfig, SupportBotConfig, SupportRouteConfig};
pub use debug::{DebugCommand, DebugCommandHandler, DebugCommandMatch, DebugResponse};
pub use error::{Result, SupportBotError};
pub use handler::SupportBotHandler;
pub use metrics::{SupportBotMetrics, SupportBotMetricsHandle};
pub use prometheus_client;
pub use state::{
    SupportRuntimeState, SupportRuntimeStatus, SupportThreadState, SupportThreadStatus,
};
