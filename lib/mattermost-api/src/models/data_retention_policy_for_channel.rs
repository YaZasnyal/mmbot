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
pub struct DataRetentionPolicyForChannel {
    /// The channel ID.
    #[serde(rename = "channel_id", skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// The number of days a message will be retained before being deleted by this policy.
    #[serde(rename = "post_duration", skip_serializing_if = "Option::is_none")]
    pub post_duration: Option<i32>,
}

impl DataRetentionPolicyForChannel {
    pub fn new() -> DataRetentionPolicyForChannel {
        DataRetentionPolicyForChannel {
            channel_id: None,
            post_duration: None,
        }
    }
}

