use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupportThreadState {
    pub version: u32,
    pub selected_instruction_ids: Vec<String>,
    pub compact_history: Vec<SupportHistoryItem>,
    pub engineer_thread: Option<EngineerThreadRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineerThreadRef {
    pub channel_id: String,
    pub root_post_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupportHistoryItem {
    pub kind: String,
    pub summary: String,
}

impl Default for SupportThreadState {
    fn default() -> Self {
        Self {
            version: 1,
            selected_instruction_ids: Vec::new(),
            compact_history: Vec::new(),
            engineer_thread: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn support_thread_state_round_trips_as_json() {
        let state = SupportThreadState {
            selected_instruction_ids: vec!["diagnostics.slow_requests".to_string()],
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
