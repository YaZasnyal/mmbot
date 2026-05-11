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
        }
    }
}

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
