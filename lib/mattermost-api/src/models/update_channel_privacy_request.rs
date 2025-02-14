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
pub struct UpdateChannelPrivacyRequest {
    /// Channel privacy setting: 'O' for a public channel, 'P' for a private channel
    #[serde(rename = "privacy")]
    pub privacy: String,
}

impl UpdateChannelPrivacyRequest {
    pub fn new(privacy: String) -> UpdateChannelPrivacyRequest {
        UpdateChannelPrivacyRequest { privacy }
    }
}
