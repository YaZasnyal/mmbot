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
pub struct RevokeSessionRequest {
    /// The session GUID to revoke.
    #[serde(rename = "session_id")]
    pub session_id: String,
}

impl RevokeSessionRequest {
    pub fn new(session_id: String) -> RevokeSessionRequest {
        RevokeSessionRequest {
            session_id,
        }
    }
}

