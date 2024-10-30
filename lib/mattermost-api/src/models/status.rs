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
pub struct Status {
    #[serde(rename = "user_id", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(rename = "manual", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub manual: Option<bool>,
    #[serde(rename = "last_activity_at", skip_serializing_if = "Option::is_none")]
    pub last_activity_at: Option<i64>,
}

impl Status {
    pub fn new() -> Status {
        Status {
            user_id: None,
            status: None,
            manual: None,
            last_activity_at: None,
        }
    }
}

