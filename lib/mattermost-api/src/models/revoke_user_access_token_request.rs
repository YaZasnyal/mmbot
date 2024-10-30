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
pub struct RevokeUserAccessTokenRequest {
    /// The user access token GUID to revoke
    #[serde(rename = "token_id")]
    pub token_id: String,
}

impl RevokeUserAccessTokenRequest {
    pub fn new(token_id: String) -> RevokeUserAccessTokenRequest {
        RevokeUserAccessTokenRequest { token_id }
    }
}
