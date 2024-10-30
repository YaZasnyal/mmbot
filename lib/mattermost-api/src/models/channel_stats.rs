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
pub struct ChannelStats {
    #[serde(rename = "channel_id", skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(rename = "member_count", skip_serializing_if = "Option::is_none")]
    pub member_count: Option<i32>,
}

impl ChannelStats {
    pub fn new() -> ChannelStats {
        ChannelStats {
            channel_id: None,
            member_count: None,
        }
    }
}

