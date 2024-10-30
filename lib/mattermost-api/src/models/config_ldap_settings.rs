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
pub struct ConfigLdapSettings {
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub enable: Option<bool>,
    #[serde(rename = "LdapServer", skip_serializing_if = "Option::is_none")]
    pub ldap_server: Option<String>,
    #[serde(rename = "LdapPort", skip_serializing_if = "Option::is_none")]
    pub ldap_port: Option<i32>,
    #[serde(rename = "ConnectionSecurity", skip_serializing_if = "Option::is_none")]
    pub connection_security: Option<String>,
    #[serde(rename = "BaseDN", skip_serializing_if = "Option::is_none")]
    pub base_dn: Option<String>,
    #[serde(rename = "BindUsername", skip_serializing_if = "Option::is_none")]
    pub bind_username: Option<String>,
    #[serde(rename = "BindPassword", skip_serializing_if = "Option::is_none")]
    pub bind_password: Option<String>,
    #[serde(rename = "UserFilter", skip_serializing_if = "Option::is_none")]
    pub user_filter: Option<String>,
    #[serde(rename = "FirstNameAttribute", skip_serializing_if = "Option::is_none")]
    pub first_name_attribute: Option<String>,
    #[serde(rename = "LastNameAttribute", skip_serializing_if = "Option::is_none")]
    pub last_name_attribute: Option<String>,
    #[serde(rename = "EmailAttribute", skip_serializing_if = "Option::is_none")]
    pub email_attribute: Option<String>,
    #[serde(rename = "UsernameAttribute", skip_serializing_if = "Option::is_none")]
    pub username_attribute: Option<String>,
    #[serde(rename = "NicknameAttribute", skip_serializing_if = "Option::is_none")]
    pub nickname_attribute: Option<String>,
    #[serde(rename = "IdAttribute", skip_serializing_if = "Option::is_none")]
    pub id_attribute: Option<String>,
    #[serde(rename = "PositionAttribute", skip_serializing_if = "Option::is_none")]
    pub position_attribute: Option<String>,
    #[serde(rename = "SyncIntervalMinutes", skip_serializing_if = "Option::is_none")]
    pub sync_interval_minutes: Option<i32>,
    #[serde(rename = "SkipCertificateVerification", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub skip_certificate_verification: Option<bool>,
    #[serde(rename = "QueryTimeout", skip_serializing_if = "Option::is_none")]
    pub query_timeout: Option<i32>,
    #[serde(rename = "MaxPageSize", skip_serializing_if = "Option::is_none")]
    pub max_page_size: Option<i32>,
    #[serde(rename = "LoginFieldName", skip_serializing_if = "Option::is_none")]
    pub login_field_name: Option<String>,
}

impl ConfigLdapSettings {
    pub fn new() -> ConfigLdapSettings {
        ConfigLdapSettings {
            enable: None,
            ldap_server: None,
            ldap_port: None,
            connection_security: None,
            base_dn: None,
            bind_username: None,
            bind_password: None,
            user_filter: None,
            first_name_attribute: None,
            last_name_attribute: None,
            email_attribute: None,
            username_attribute: None,
            nickname_attribute: None,
            id_attribute: None,
            position_attribute: None,
            sync_interval_minutes: None,
            skip_certificate_verification: None,
            query_timeout: None,
            max_page_size: None,
            login_field_name: None,
        }
    }
}

