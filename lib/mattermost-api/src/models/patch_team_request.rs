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
pub struct PatchTeamRequest {
    #[serde(rename = "display_name", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "company_name", skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
    #[serde(rename = "invite_id", skip_serializing_if = "Option::is_none")]
    pub invite_id: Option<String>,
    #[serde(rename = "allow_open_invite", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub allow_open_invite: Option<bool>,
}

impl PatchTeamRequest {
    pub fn new() -> PatchTeamRequest {
        PatchTeamRequest {
            display_name: None,
            description: None,
            company_name: None,
            invite_id: None,
            allow_open_invite: None,
        }
    }
}

