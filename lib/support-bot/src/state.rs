use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupportThreadState {
    pub version: u32,
    pub selected_instruction_ids: Vec<String>,
    pub compact_history: Vec<SupportHistoryItem>,
    pub last_action: Option<String>,
    pub waiting_for_user: bool,
    pub waiting_for_engineer: bool,
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
            last_action: None,
            waiting_for_user: false,
            waiting_for_engineer: false,
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
            last_action: Some("notify_engineer".to_string()),
            waiting_for_engineer: true,
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
