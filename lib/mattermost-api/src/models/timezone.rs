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
pub struct Timezone {
    /// Set to \"true\" to use the browser/system timezone, \"false\" to set manually. Defaults to \"true\".
    #[serde(
        rename = "useAutomaticTimezone",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub use_automatic_timezone: Option<bool>,
    /// Value when setting manually the timezone, i.e. \"Europe/Berlin\".
    #[serde(rename = "manualTimezone", skip_serializing_if = "Option::is_none")]
    pub manual_timezone: Option<String>,
    /// This value is set automatically when the \"useAutomaticTimezone\" is set to \"true\".
    #[serde(rename = "automaticTimezone", skip_serializing_if = "Option::is_none")]
    pub automatic_timezone: Option<String>,
}

impl Timezone {
    pub fn new() -> Timezone {
        Timezone {
            use_automatic_timezone: None,
            manual_timezone: None,
            automatic_timezone: None,
        }
    }
}
