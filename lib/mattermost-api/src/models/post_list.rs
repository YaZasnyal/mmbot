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
pub struct PostList {
    #[serde(rename = "order", skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
    #[serde(rename = "posts", skip_serializing_if = "Option::is_none")]
    pub posts: Option<std::collections::HashMap<String, models::Post>>,
    /// The ID of next post. Not omitted when empty or not relevant.
    #[serde(rename = "next_post_id", skip_serializing_if = "Option::is_none")]
    pub next_post_id: Option<String>,
    /// The ID of previous post. Not omitted when empty or not relevant.
    #[serde(rename = "prev_post_id", skip_serializing_if = "Option::is_none")]
    pub prev_post_id: Option<String>,
    /// Whether there are more items after this page.
    #[serde(rename = "has_next", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub has_next: Option<bool>,
}

impl PostList {
    pub fn new() -> PostList {
        PostList {
            order: None,
            posts: None,
            next_post_id: None,
            prev_post_id: None,
            has_next: None,
        }
    }
}

