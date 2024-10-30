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
pub struct CreateUserRequest {
    #[serde(rename = "email")]
    pub email: String,
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "first_name", skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(rename = "last_name", skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(rename = "nickname", skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(rename = "position", skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    #[serde(rename = "timezone", skip_serializing_if = "Option::is_none")]
    pub timezone: Option<Box<models::Timezone>>,
    /// Service-specific authentication data, such as email address.
    #[serde(rename = "auth_data", skip_serializing_if = "Option::is_none")]
    pub auth_data: Option<String>,
    /// The authentication service, one of \"email\", \"gitlab\", \"ldap\", \"saml\", \"office365\", \"google\", and \"\".
    #[serde(rename = "auth_service", skip_serializing_if = "Option::is_none")]
    pub auth_service: Option<String>,
    /// The password used for email authentication.
    #[serde(rename = "password", skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(rename = "locale", skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(rename = "props", skip_serializing_if = "Option::is_none")]
    pub props: Option<serde_json::Value>,
    #[serde(rename = "notify_props", skip_serializing_if = "Option::is_none")]
    pub notify_props: Option<Box<models::UserNotifyProps>>,
}

impl CreateUserRequest {
    pub fn new(email: String, username: String) -> CreateUserRequest {
        CreateUserRequest {
            email,
            username,
            first_name: None,
            last_name: None,
            nickname: None,
            position: None,
            timezone: None,
            auth_data: None,
            auth_service: None,
            password: None,
            locale: None,
            props: None,
            notify_props: None,
        }
    }
}

