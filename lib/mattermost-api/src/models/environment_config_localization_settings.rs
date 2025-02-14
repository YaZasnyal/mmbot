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
pub struct EnvironmentConfigLocalizationSettings {
    #[serde(
        rename = "DefaultServerLocale",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub default_server_locale: Option<bool>,
    #[serde(
        rename = "DefaultClientLocale",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub default_client_locale: Option<bool>,
    #[serde(
        rename = "AvailableLocales",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub available_locales: Option<bool>,
}

impl EnvironmentConfigLocalizationSettings {
    pub fn new() -> EnvironmentConfigLocalizationSettings {
        EnvironmentConfigLocalizationSettings {
            default_server_locale: None,
            default_client_locale: None,
            available_locales: None,
        }
    }
}
