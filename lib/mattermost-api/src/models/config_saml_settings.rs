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
pub struct ConfigSamlSettings {
    #[serde(
        rename = "Enable",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub enable: Option<bool>,
    #[serde(
        rename = "Verify",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub verify: Option<bool>,
    #[serde(
        rename = "Encrypt",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub encrypt: Option<bool>,
    #[serde(rename = "IdpUrl", skip_serializing_if = "Option::is_none")]
    pub idp_url: Option<String>,
    #[serde(rename = "IdpDescriptorUrl", skip_serializing_if = "Option::is_none")]
    pub idp_descriptor_url: Option<String>,
    #[serde(
        rename = "AssertionConsumerServiceURL",
        skip_serializing_if = "Option::is_none"
    )]
    pub assertion_consumer_service_url: Option<String>,
    #[serde(rename = "IdpCertificateFile", skip_serializing_if = "Option::is_none")]
    pub idp_certificate_file: Option<String>,
    #[serde(
        rename = "PublicCertificateFile",
        skip_serializing_if = "Option::is_none"
    )]
    pub public_certificate_file: Option<String>,
    #[serde(rename = "PrivateKeyFile", skip_serializing_if = "Option::is_none")]
    pub private_key_file: Option<String>,
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
    #[serde(rename = "LocaleAttribute", skip_serializing_if = "Option::is_none")]
    pub locale_attribute: Option<String>,
    #[serde(rename = "PositionAttribute", skip_serializing_if = "Option::is_none")]
    pub position_attribute: Option<String>,
    #[serde(rename = "LoginButtonText", skip_serializing_if = "Option::is_none")]
    pub login_button_text: Option<String>,
}

impl ConfigSamlSettings {
    pub fn new() -> ConfigSamlSettings {
        ConfigSamlSettings {
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
