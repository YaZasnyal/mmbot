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
pub struct SamlCertificateStatus {
    /// Status is good when `true`
    #[serde(
        rename = "idp_certificate_file",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub idp_certificate_file: Option<bool>,
    /// Status is good when `true`
    #[serde(
        rename = "public_certificate_file",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub public_certificate_file: Option<bool>,
    /// Status is good when `true`
    #[serde(
        rename = "private_key_file",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub private_key_file: Option<bool>,
}

impl SamlCertificateStatus {
    pub fn new() -> SamlCertificateStatus {
        SamlCertificateStatus {
            idp_certificate_file: None,
            public_certificate_file: None,
            private_key_file: None,
        }
    }
}
