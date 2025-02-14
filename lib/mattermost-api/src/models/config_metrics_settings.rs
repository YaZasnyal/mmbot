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
pub struct ConfigMetricsSettings {
    #[serde(
        rename = "Enable",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub enable: Option<bool>,
    #[serde(rename = "BlockProfileRate", skip_serializing_if = "Option::is_none")]
    pub block_profile_rate: Option<i32>,
    #[serde(rename = "ListenAddress", skip_serializing_if = "Option::is_none")]
    pub listen_address: Option<String>,
}

impl ConfigMetricsSettings {
    pub fn new() -> ConfigMetricsSettings {
        ConfigMetricsSettings {
            enable: None,
            block_profile_rate: None,
            listen_address: None,
        }
    }
}
