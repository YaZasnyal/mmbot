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
pub struct EnvironmentConfigSamlSettings {
    #[serde(rename = "Enable", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub enable: Option<bool>,
    #[serde(rename = "Verify", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub verify: Option<bool>,
    #[serde(rename = "Encrypt", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub encrypt: Option<bool>,
    #[serde(rename = "IdpUrl", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub idp_url: Option<bool>,
    #[serde(rename = "IdpDescriptorUrl", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub idp_descriptor_url: Option<bool>,
    #[serde(rename = "AssertionConsumerServiceURL", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub assertion_consumer_service_url: Option<bool>,
    #[serde(rename = "IdpCertificateFile", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub idp_certificate_file: Option<bool>,
    #[serde(rename = "PublicCertificateFile", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub public_certificate_file: Option<bool>,
    #[serde(rename = "PrivateKeyFile", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub private_key_file: Option<bool>,
    #[serde(rename = "FirstNameAttribute", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub first_name_attribute: Option<bool>,
    #[serde(rename = "LastNameAttribute", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub last_name_attribute: Option<bool>,
    #[serde(rename = "EmailAttribute", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub email_attribute: Option<bool>,
    #[serde(rename = "UsernameAttribute", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub username_attribute: Option<bool>,
    #[serde(rename = "NicknameAttribute", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub nickname_attribute: Option<bool>,
    #[serde(rename = "LocaleAttribute", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub locale_attribute: Option<bool>,
    #[serde(rename = "PositionAttribute", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub position_attribute: Option<bool>,
    #[serde(rename = "LoginButtonText", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub login_button_text: Option<bool>,
}

impl EnvironmentConfigSamlSettings {
    pub fn new() -> EnvironmentConfigSamlSettings {
        EnvironmentConfigSamlSettings {
            enable: None,
            verify: None,
            encrypt: None,
            idp_url: None,
            idp_descriptor_url: None,
            assertion_consumer_service_url: None,
            idp_certificate_file: None,
            public_certificate_file: None,
            private_key_file: None,
            first_name_attribute: None,
            last_name_attribute: None,
            email_attribute: None,
            username_attribute: None,
            nickname_attribute: None,
            locale_attribute: None,
            position_attribute: None,
            login_button_text: None,
        }
    }
}

