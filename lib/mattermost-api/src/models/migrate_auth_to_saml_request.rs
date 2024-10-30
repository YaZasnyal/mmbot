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
pub struct MigrateAuthToSamlRequest {
    /// The current authentication type for the matched users.
    #[serde(rename = "from")]
    pub from: String,
    /// Users map.
    #[serde(rename = "matches")]
    pub matches: serde_json::Value,
    #[serde(rename = "auto")]
    pub auto: bool,
}

impl MigrateAuthToSamlRequest {
    pub fn new(from: String, matches: serde_json::Value, auto: bool) -> MigrateAuthToSamlRequest {
        MigrateAuthToSamlRequest {
            from,
            matches,
            auto,
        }
    }
}
