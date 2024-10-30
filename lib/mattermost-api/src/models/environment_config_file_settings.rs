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
pub struct EnvironmentConfigFileSettings {
    #[serde(rename = "MaxFileSize", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub max_file_size: Option<bool>,
    #[serde(rename = "DriverName", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub driver_name: Option<bool>,
    #[serde(rename = "Directory", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub directory: Option<bool>,
    #[serde(rename = "EnablePublicLink", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub enable_public_link: Option<bool>,
    #[serde(rename = "PublicLinkSalt", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub public_link_salt: Option<bool>,
    #[serde(rename = "ThumbnailWidth", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub thumbnail_width: Option<bool>,
    #[serde(rename = "ThumbnailHeight", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub thumbnail_height: Option<bool>,
    #[serde(rename = "PreviewWidth", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub preview_width: Option<bool>,
    #[serde(rename = "PreviewHeight", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub preview_height: Option<bool>,
    #[serde(rename = "ProfileWidth", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub profile_width: Option<bool>,
    #[serde(rename = "ProfileHeight", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub profile_height: Option<bool>,
    #[serde(rename = "InitialFont", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub initial_font: Option<bool>,
    #[serde(rename = "AmazonS3AccessKeyId", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub amazon_s3_access_key_id: Option<bool>,
    #[serde(rename = "AmazonS3SecretAccessKey", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub amazon_s3_secret_access_key: Option<bool>,
    #[serde(rename = "AmazonS3Bucket", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub amazon_s3_bucket: Option<bool>,
    #[serde(rename = "AmazonS3Region", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub amazon_s3_region: Option<bool>,
    #[serde(rename = "AmazonS3Endpoint", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub amazon_s3_endpoint: Option<bool>,
    #[serde(rename = "AmazonS3SSL", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub amazon_s3_ssl: Option<bool>,
}

impl EnvironmentConfigFileSettings {
    pub fn new() -> EnvironmentConfigFileSettings {
        EnvironmentConfigFileSettings {
            max_file_size: None,
            driver_name: None,
            directory: None,
            enable_public_link: None,
            public_link_salt: None,
            thumbnail_width: None,
            thumbnail_height: None,
            preview_width: None,
            preview_height: None,
            profile_width: None,
            profile_height: None,
            initial_font: None,
            amazon_s3_access_key_id: None,
            amazon_s3_secret_access_key: None,
            amazon_s3_bucket: None,
            amazon_s3_region: None,
            amazon_s3_endpoint: None,
            amazon_s3_ssl: None,
        }
    }
}

