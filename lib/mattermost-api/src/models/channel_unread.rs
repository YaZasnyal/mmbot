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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelUnread {
    #[serde(rename = "team_id", skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    #[serde(rename = "channel_id", skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(rename = "msg_count", skip_serializing_if = "Option::is_none")]
    pub msg_count: Option<i32>,
    #[serde(rename = "mention_count", skip_serializing_if = "Option::is_none")]
    pub mention_count: Option<i32>,
}

impl ChannelUnread {
    pub fn new() -> ChannelUnread {
        ChannelUnread {
            team_id: None,
            channel_id: None,
            msg_count: None,
            mention_count: None,
        }
    }
}

