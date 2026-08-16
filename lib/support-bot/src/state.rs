use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupportThreadState {
    pub version: u32,
    #[serde(default)]
    pub status: SupportThreadStatus,
    #[serde(default)]
    pub finished_summary: Option<String>,
    #[serde(default)]
    pub ignored_reason: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "support_runtime"
    )]
    pub runtime: Option<SupportRuntimeState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupportRuntimeState {
    pub version: u32,
    pub acp_session_id: String,
    pub last_inbound_post_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_outbound_post_id: Option<String>,
    pub status: SupportRuntimeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SupportRuntimeStatus {
    Active,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SupportThreadStatus {
    #[default]
    Active,
    Ignored,
    Finished,
    Stopped,
}

impl Default for SupportThreadState {
    fn default() -> Self {
        Self {
            version: 1,
            status: SupportThreadStatus::Active,
            finished_summary: None,
            ignored_reason: None,
            runtime: None,
        }
    }
}

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
