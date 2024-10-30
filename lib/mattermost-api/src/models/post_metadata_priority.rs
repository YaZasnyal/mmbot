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

/// PostMetadataPriority : Post priority set for this post. This field will be null if no priority metadata has been set. 
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PostMetadataPriority {
    /// The priority label of a post, could be either empty, important, or urgent.
    #[serde(rename = "priority", skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    /// Whether the post author has requested for acknowledgements or not.
    #[serde(rename = "requested_ack", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub requested_ack: Option<bool>,
}

impl PostMetadataPriority {
    /// Post priority set for this post. This field will be null if no priority metadata has been set. 
    pub fn new() -> PostMetadataPriority {
        PostMetadataPriority {
            priority: None,
            requested_ack: None,
        }
    }
}

