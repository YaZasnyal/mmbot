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
pub struct UserTermsOfService {
    /// The unique identifier of the user who performed this terms of service action.
    #[serde(rename = "user_id", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// The unique identifier of the terms of service the action was performed on.
    #[serde(rename = "terms_of_service_id", skip_serializing_if = "Option::is_none")]
    pub terms_of_service_id: Option<String>,
    /// The time in milliseconds that this action was performed.
    #[serde(rename = "create_at", skip_serializing_if = "Option::is_none")]
    pub create_at: Option<i64>,
}

impl UserTermsOfService {
    pub fn new() -> UserTermsOfService {
        UserTermsOfService {
            user_id: None,
            terms_of_service_id: None,
            create_at: None,
        }
    }
}

