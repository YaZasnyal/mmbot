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
pub struct Emoji {
    /// The ID of the emoji
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The ID of the user that made the emoji
    #[serde(rename = "creator_id", skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<String>,
    /// The name of the emoji
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The time in milliseconds the emoji was made
    #[serde(rename = "create_at", skip_serializing_if = "Option::is_none")]
    pub create_at: Option<i64>,
    /// The time in milliseconds the emoji was last updated
    #[serde(rename = "update_at", skip_serializing_if = "Option::is_none")]
    pub update_at: Option<i64>,
    /// The time in milliseconds the emoji was deleted
    #[serde(rename = "delete_at", skip_serializing_if = "Option::is_none")]
    pub delete_at: Option<i64>,
}

impl Emoji {
    pub fn new() -> Emoji {
        Emoji {
            id: None,
            creator_id: None,
            name: None,
            create_at: None,
            update_at: None,
            delete_at: None,
        }
    }
}
