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
pub struct PostMetadataEmbedsInner {
    /// The type of content that is embedded in this point.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Type>,
    /// The URL of the embedded content, if one exists.
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Any additional information about the embedded content. Only used at this time to store OpenGraph metadata. This field will be null for non-OpenGraph embeds. 
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl PostMetadataEmbedsInner {
    pub fn new() -> PostMetadataEmbedsInner {
        PostMetadataEmbedsInner {
            r#type: None,
            url: None,
            data: None,
        }
    }
}
/// The type of content that is embedded in this point.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "message_attachment")]
    MessageAttachment,
    #[serde(rename = "opengraph")]
    Opengraph,
    #[serde(rename = "link")]
    Link,
}

impl Default for Type {
    fn default() -> Type {
        Self::Image
    }
}

