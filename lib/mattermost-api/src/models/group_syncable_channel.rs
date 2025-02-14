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
pub struct GroupSyncableChannel {
    #[serde(rename = "channel_id", skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(rename = "group_id", skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(
        rename = "auto_add",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub auto_add: Option<bool>,
    #[serde(rename = "create_at", skip_serializing_if = "Option::is_none")]
    pub create_at: Option<i64>,
    #[serde(rename = "delete_at", skip_serializing_if = "Option::is_none")]
    pub delete_at: Option<i64>,
    #[serde(rename = "update_at", skip_serializing_if = "Option::is_none")]
    pub update_at: Option<i64>,
}

impl GroupSyncableChannel {
    pub fn new() -> GroupSyncableChannel {
        GroupSyncableChannel {
            channel_id: None,
            group_id: None,
            auto_add: None,
            create_at: None,
            delete_at: None,
            update_at: None,
        }
    }
}
