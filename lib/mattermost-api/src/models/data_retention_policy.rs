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
pub struct DataRetentionPolicy {
    /// The display name for this retention policy.
    #[serde(rename = "display_name", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The number of days a message will be retained before being deleted by this policy. If this value is less than 0, the policy has infinite retention (i.e. messages are never deleted). 
    #[serde(rename = "post_duration", skip_serializing_if = "Option::is_none")]
    pub post_duration: Option<i32>,
    /// The ID of this retention policy.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl DataRetentionPolicy {
    pub fn new() -> DataRetentionPolicy {
        DataRetentionPolicy {
            display_name: None,
            post_duration: None,
            id: None,
        }
    }
}

