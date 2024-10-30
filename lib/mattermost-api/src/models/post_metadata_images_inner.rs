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
pub struct PostMetadataImagesInner {
    #[serde(rename = "height", skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    #[serde(rename = "width", skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
}

impl PostMetadataImagesInner {
    pub fn new() -> PostMetadataImagesInner {
        PostMetadataImagesInner {
            height: None,
            width: None,
        }
    }
}

