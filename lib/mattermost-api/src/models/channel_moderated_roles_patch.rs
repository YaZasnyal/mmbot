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
pub struct ChannelModeratedRolesPatch {
    #[serde(
        rename = "guests",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub guests: Option<bool>,
    #[serde(
        rename = "members",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub members: Option<bool>,
}

impl ChannelModeratedRolesPatch {
    pub fn new() -> ChannelModeratedRolesPatch {
        ChannelModeratedRolesPatch {
            guests: None,
            members: None,
        }
    }
}
