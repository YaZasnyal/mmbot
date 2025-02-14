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
pub struct ReoderChecklistItemRequest {
    /// Zero-based index of the item to reorder.
    #[serde(rename = "item_num")]
    pub item_num: i32,
    /// Zero-based index of the new place to move the item to.
    #[serde(rename = "new_location")]
    pub new_location: i32,
}

impl ReoderChecklistItemRequest {
    pub fn new(item_num: i32, new_location: i32) -> ReoderChecklistItemRequest {
        ReoderChecklistItemRequest {
            item_num,
            new_location,
        }
    }
}
