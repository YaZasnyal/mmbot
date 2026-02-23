use serde::Deserialize;

use crate::nested_decoder::Nested;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// WebSocket connection established (contains bot's connection ID and user ID)
    Hello(HelloEvent),
    /// New post in a channel
    Posted(Box<PostedEvent>),
    /// Reaction added to a post
    ReactionAdded(ReactionAddedEvent),
    /// Reaction removed from a post
    ReactionRemoved(ReactionRemovedEvent),
    /// Unknown or unhandled event type
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HelloEvent {
    /// WebSocket connection ID
    pub connection_id: String,
    /// Mattermost server version
    pub server_version: String,
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
    pub reaction: Nested<mattermost_api::models::reaction::Reaction>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReactionRemovedEvent {
    pub reaction: Nested<mattermost_api::models::reaction::Reaction>,
}
