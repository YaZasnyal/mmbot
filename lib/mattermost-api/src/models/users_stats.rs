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
pub struct UsersStats {
    #[serde(rename = "total_users_count", skip_serializing_if = "Option::is_none")]
    pub total_users_count: Option<i32>,
}

impl UsersStats {
    pub fn new() -> UsersStats {
        UsersStats {
            total_users_count: None,
        }
    }
}
