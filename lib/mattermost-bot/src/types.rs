use serde::Deserialize;
use std::sync::Arc;

use crate::nested_decoder::Nested;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Posted(PostedEvent),
    ReactionAdded(ReactionAddedEvent),
    ReactionRemoved(ReactionRemovedEvent),
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostedEvent {
    pub channel_display_name: String,
    pub channel_name: String,
    pub channel_type: ChannelType,
    pub post: Nested<mattermost_api::models::post::Post>,
    pub sender_name: String,
    pub set_online: bool,
    pub team_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ChannelType {
    #[serde(rename = "O")]
    Open,
    #[serde(rename = "P")]
    Private,
    /// Direct messages
    #[serde(rename = "D")]
    Direct,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReactionAddedEvent {
    reaction: Nested<mattermost_api::models::reaction::Reaction>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReactionRemovedEvent {
    reaction: Nested<mattermost_api::models::reaction::Reaction>,
}
