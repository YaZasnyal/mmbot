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
mod tests {
    use super::*;

    #[test]
    fn support_thread_state_round_trips_as_json() {
        let state = SupportThreadState::default();

        let value = serde_json::to_value(&state).unwrap();
        let decoded: SupportThreadState = serde_json::from_value(value).unwrap();

        assert_eq!(decoded, state);
    }

    #[test]
    fn support_thread_state_ignores_legacy_engineer_thread_field() {
        let decoded: SupportThreadState = serde_json::from_value(serde_json::json!({
            "version": 1,
            "engineer_thread": {
                "channel_id": "engineers",
                "root_post_id": "post-1"
            },
            "status": "active"
        }))
        .unwrap();

        assert_eq!(decoded, SupportThreadState::default());
    }
}
