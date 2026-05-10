use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupportThreadState {
    pub version: u32,
    #[serde(default)]
    pub status: SupportThreadStatus,
    #[serde(default)]
    pub finished_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineerThreadRef {
    pub channel_id: String,
    pub root_post_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SupportThreadStatus {
    #[default]
    Active,
    Finished,
    Stopped,
}

impl Default for SupportThreadState {
    fn default() -> Self {
        Self {
            version: 1,
            status: SupportThreadStatus::Active,
            finished_summary: None,
        }
    }
}

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
