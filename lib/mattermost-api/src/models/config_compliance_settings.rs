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
pub struct ConfigComplianceSettings {
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub enable: Option<bool>,
    #[serde(rename = "Directory", skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
    #[serde(rename = "EnableDaily", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub enable_daily: Option<bool>,
}

impl ConfigComplianceSettings {
    pub fn new() -> ConfigComplianceSettings {
        ConfigComplianceSettings {
            enable: None,
            directory: None,
            enable_daily: None,
        }
    }
}

