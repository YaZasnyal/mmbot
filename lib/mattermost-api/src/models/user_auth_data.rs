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
pub struct UserAuthData {
    /// Service-specific authentication data
    #[serde(rename = "auth_data")]
    pub auth_data: String,
    /// The authentication service such as \"email\", \"gitlab\", or \"ldap\"
    #[serde(rename = "auth_service")]
    pub auth_service: String,
}

impl UserAuthData {
    pub fn new(auth_data: String, auth_service: String) -> UserAuthData {
        UserAuthData {
            auth_data,
            auth_service,
        }
    }
}

