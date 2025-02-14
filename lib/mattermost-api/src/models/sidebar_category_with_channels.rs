/*
 * Mattermost API Reference
 *
 * There is also a work-in-progress [Postman API reference](https://documenter.getpostman.com/view/4508214/RW8FERUn).
 *
 * The version of the OpenAPI document: 4.0.0
 * Contact: feedback@mattermost.com
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// SidebarCategoryWithChannels : User's sidebar category with it's channels
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct SidebarCategoryWithChannels {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "user_id", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(rename = "team_id", skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    #[serde(rename = "display_name", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Type>,
    #[serde(rename = "channel_ids", skip_serializing_if = "Option::is_none")]
    pub channel_ids: Option<Vec<String>>,
}

impl SidebarCategoryWithChannels {
    /// User's sidebar category with it's channels
    pub fn new() -> SidebarCategoryWithChannels {
        SidebarCategoryWithChannels {
            id: None,
            user_id: None,
            team_id: None,
            display_name: None,
            r#type: None,
            channel_ids: None,
        }
    }
}
///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "channels")]
    Channels,
    #[serde(rename = "custom")]
    Custom,
    #[serde(rename = "direct_messages")]
    DirectMessages,
    #[serde(rename = "favorites")]
    Favorites,
}

impl Default for Type {
    fn default() -> Type {
        Self::Channels
    }
}
