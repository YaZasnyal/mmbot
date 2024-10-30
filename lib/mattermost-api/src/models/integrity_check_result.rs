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

/// IntegrityCheckResult : an object with the result of the integrity check.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntegrityCheckResult {
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<Box<models::RelationalIntegrityCheckData>>,
    /// a string value set in case of error.
    #[serde(rename = "err", skip_serializing_if = "Option::is_none")]
    pub err: Option<String>,
}

impl IntegrityCheckResult {
    /// an object with the result of the integrity check.
    pub fn new() -> IntegrityCheckResult {
        IntegrityCheckResult {
            data: None,
            err: None,
        }
    }
}
