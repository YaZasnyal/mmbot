use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupportThreadState {
    pub version: u32,
    pub engineer_thread: Option<EngineerThreadRef>,
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
}

impl Default for SupportThreadState {
    fn default() -> Self {
        Self {
            version: 1,
            engineer_thread: None,
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
        let state = SupportThreadState {
            engineer_thread: Some(EngineerThreadRef {
                channel_id: "engineers".to_string(),
                root_post_id: "post-1".to_string(),
            }),
            ..Default::default()
        };

        let value = serde_json::to_value(&state).unwrap();
        let decoded: SupportThreadState = serde_json::from_value(value).unwrap();

        assert_eq!(decoded, state);
    }
}
