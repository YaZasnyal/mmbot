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
pub struct PatchGroupSyncableForTeamRequest {
    #[serde(rename = "auto_add", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub auto_add: Option<bool>,
}

impl PatchGroupSyncableForTeamRequest {
    pub fn new() -> PatchGroupSyncableForTeamRequest {
        PatchGroupSyncableForTeamRequest {
            auto_add: None,
        }
    }
}

