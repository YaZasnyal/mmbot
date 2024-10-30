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
pub struct FileInfo {
    /// The unique identifier for this file
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The ID of the user that uploaded this file
    #[serde(rename = "user_id", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// If this file is attached to a post, the ID of that post
    #[serde(rename = "post_id", skip_serializing_if = "Option::is_none")]
    pub post_id: Option<String>,
    /// The time in milliseconds a file was created
    #[serde(rename = "create_at", skip_serializing_if = "Option::is_none")]
    pub create_at: Option<i64>,
    /// The time in milliseconds a file was last updated
    #[serde(rename = "update_at", skip_serializing_if = "Option::is_none")]
    pub update_at: Option<i64>,
    /// The time in milliseconds a file was deleted
    #[serde(rename = "delete_at", skip_serializing_if = "Option::is_none")]
    pub delete_at: Option<i64>,
    /// The name of the file
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The extension at the end of the file name
    #[serde(rename = "extension", skip_serializing_if = "Option::is_none")]
    pub extension: Option<String>,
    /// The size of the file in bytes
    #[serde(rename = "size", skip_serializing_if = "Option::is_none")]
    pub size: Option<i32>,
    /// The MIME type of the file
    #[serde(rename = "mime_type", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// If this file is an image, the width of the file
    #[serde(rename = "width", skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// If this file is an image, the height of the file
    #[serde(rename = "height", skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// If this file is an image, whether or not it has a preview-sized version
    #[serde(
        rename = "has_preview_image",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub has_preview_image: Option<bool>,
}

impl FileInfo {
    pub fn new() -> FileInfo {
        FileInfo {
            id: None,
            user_id: None,
            post_id: None,
            create_at: None,
            update_at: None,
            delete_at: None,
            name: None,
            extension: None,
            size: None,
            mime_type: None,
            width: None,
            height: None,
            has_preview_image: None,
        }
    }
}
