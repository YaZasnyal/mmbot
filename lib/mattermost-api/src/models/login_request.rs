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
pub struct LoginRequest {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "login_id", skip_serializing_if = "Option::is_none")]
    pub login_id: Option<String>,
    #[serde(rename = "token", skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(rename = "device_id", skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(
        rename = "ldap_only",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub ldap_only: Option<bool>,
    /// The password used for email authentication.
    #[serde(rename = "password", skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

impl LoginRequest {
    pub fn new() -> LoginRequest {
        LoginRequest {
            id: None,
            login_id: None,
            token: None,
            device_id: None,
            ldap_only: None,
            password: None,
        }
    }
}
