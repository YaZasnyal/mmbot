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
pub struct EnvironmentConfigSqlSettings {
    #[serde(rename = "DriverName", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub driver_name: Option<bool>,
    #[serde(rename = "DataSource", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub data_source: Option<bool>,
    #[serde(rename = "DataSourceReplicas", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub data_source_replicas: Option<bool>,
    #[serde(rename = "MaxIdleConns", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub max_idle_conns: Option<bool>,
    #[serde(rename = "MaxOpenConns", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub max_open_conns: Option<bool>,
    #[serde(rename = "Trace", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub trace: Option<bool>,
    #[serde(rename = "AtRestEncryptKey", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub at_rest_encrypt_key: Option<bool>,
}

impl EnvironmentConfigSqlSettings {
    pub fn new() -> EnvironmentConfigSqlSettings {
        EnvironmentConfigSqlSettings {
            driver_name: None,
            data_source: None,
            data_source_replicas: None,
            max_idle_conns: None,
            max_open_conns: None,
            trace: None,
            at_rest_encrypt_key: None,
        }
    }
}

