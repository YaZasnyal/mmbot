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
pub struct DataRetentionPolicyCreate {
    /// The display name for this retention policy.
    #[serde(rename = "display_name")]
    pub display_name: String,
    /// The number of days a message will be retained before being deleted by this policy. If this value is less than 0, the policy has infinite retention (i.e. messages are never deleted).
    #[serde(rename = "post_duration")]
    pub post_duration: i32,
    /// The IDs of the teams to which this policy should be applied.
    #[serde(rename = "team_ids", skip_serializing_if = "Option::is_none")]
    pub team_ids: Option<Vec<String>>,
    /// The IDs of the channels to which this policy should be applied.
    #[serde(rename = "channel_ids", skip_serializing_if = "Option::is_none")]
    pub channel_ids: Option<Vec<String>>,
}

impl DataRetentionPolicyCreate {
    pub fn new(display_name: String, post_duration: i32) -> DataRetentionPolicyCreate {
        DataRetentionPolicyCreate {
            display_name,
            post_duration,
            team_ids: None,
            channel_ids: None,
        }
    }
}
