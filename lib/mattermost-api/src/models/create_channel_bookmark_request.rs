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
pub struct CreateChannelBookmarkRequest {
    /// The ID of the file associated with the channel bookmark. Required for bookmarks of type 'file'
    #[serde(rename = "file_id", skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// The name of the channel bookmark
    #[serde(rename = "display_name")]
    pub display_name: String,
    /// The URL associated with the channel bookmark. Required for bookmarks of type 'link'
    #[serde(rename = "link_url", skip_serializing_if = "Option::is_none")]
    pub link_url: Option<String>,
    /// The URL of the image associated with the channel bookmark. Optional, only applies for bookmarks of type 'link'
    #[serde(rename = "image_url", skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// The emoji of the channel bookmark
    #[serde(rename = "emoji", skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    /// * `link` for channel bookmarks that reference a link. `link_url` is requied * `file` for channel bookmarks that reference a file. `file_id` is required
    #[serde(rename = "type")]
    pub r#type: Type,
}

impl CreateChannelBookmarkRequest {
    pub fn new(display_name: String, r#type: Type) -> CreateChannelBookmarkRequest {
        CreateChannelBookmarkRequest {
            file_id: None,
            display_name,
            link_url: None,
            image_url: None,
            emoji: None,
            r#type,
        }
    }
}
/// * `link` for channel bookmarks that reference a link. `link_url` is requied * `file` for channel bookmarks that reference a file. `file_id` is required
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "link")]
    Link,
    #[serde(rename = "file")]
    File,
}

impl Default for Type {
    fn default() -> Type {
        Self::Link
    }
}
