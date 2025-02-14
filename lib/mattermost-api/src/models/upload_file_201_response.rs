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
pub struct UploadFile201Response {
    /// A list of file metadata that has been stored in the database
    #[serde(rename = "file_infos", skip_serializing_if = "Option::is_none")]
    pub file_infos: Option<Vec<models::FileInfo>>,
    /// A list of the client_ids that were provided in the request
    #[serde(rename = "client_ids", skip_serializing_if = "Option::is_none")]
    pub client_ids: Option<Vec<String>>,
}

impl UploadFile201Response {
    pub fn new() -> UploadFile201Response {
        UploadFile201Response {
            file_infos: None,
            client_ids: None,
        }
    }
}
