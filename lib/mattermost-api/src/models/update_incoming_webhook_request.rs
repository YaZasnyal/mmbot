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
pub struct UpdateIncomingWebhookRequest {
    /// Incoming webhook GUID
    #[serde(rename = "id")]
    pub id: String,
    /// The ID of a public channel or private group that receives the webhook payloads.
    #[serde(rename = "channel_id")]
    pub channel_id: String,
    /// The display name for this incoming webhook
    #[serde(rename = "display_name")]
    pub display_name: String,
    /// The description for this incoming webhook
    #[serde(rename = "description")]
    pub description: String,
    /// The username this incoming webhook will post as.
    #[serde(rename = "username", skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// The profile picture this incoming webhook will use when posting.
    #[serde(rename = "icon_url", skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

impl UpdateIncomingWebhookRequest {
    pub fn new(
        id: String,
        channel_id: String,
        display_name: String,
        description: String,
    ) -> UpdateIncomingWebhookRequest {
        UpdateIncomingWebhookRequest {
            id,
            channel_id,
            display_name,
            description,
            username: None,
            icon_url: None,
        }
    }
}
