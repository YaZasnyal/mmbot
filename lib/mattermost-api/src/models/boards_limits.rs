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
pub struct BoardsLimits {
    #[serde(
        rename = "cards",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub cards: Option<Option<i32>>,
    #[serde(
        rename = "views",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub views: Option<Option<i32>>,
}

impl BoardsLimits {
    pub fn new() -> BoardsLimits {
        BoardsLimits {
            cards: None,
            views: None,
        }
    }
}
