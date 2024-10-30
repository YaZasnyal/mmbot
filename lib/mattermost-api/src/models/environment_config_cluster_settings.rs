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
pub struct EnvironmentConfigClusterSettings {
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub enable: Option<bool>,
    #[serde(rename = "InterNodeListenAddress", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub inter_node_listen_address: Option<bool>,
    #[serde(rename = "InterNodeUrls", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub inter_node_urls: Option<bool>,
}

impl EnvironmentConfigClusterSettings {
    pub fn new() -> EnvironmentConfigClusterSettings {
        EnvironmentConfigClusterSettings {
            enable: None,
            inter_node_listen_address: None,
            inter_node_urls: None,
        }
    }
}

