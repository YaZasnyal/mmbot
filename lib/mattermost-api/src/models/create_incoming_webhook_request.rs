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
pub struct CreateIncomingWebhookRequest {
    /// The ID of a public channel or private group that receives the webhook payloads.
    #[serde(rename = "channel_id")]
    pub channel_id: String,
    /// The ID of the owner of the webhook if different than the requester. Required for [local mode](https://docs.mattermost.com/administration/mmctl-cli-tool.html#local-mode).
    #[serde(rename = "user_id", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// The display name for this incoming webhook
    #[serde(rename = "display_name", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The description for this incoming webhook
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The username this incoming webhook will post as.
    #[serde(rename = "username", skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// The profile picture this incoming webhook will use when posting.
    #[serde(rename = "icon_url", skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

impl CreateIncomingWebhookRequest {
    pub fn new(channel_id: String) -> CreateIncomingWebhookRequest {
        CreateIncomingWebhookRequest {
            channel_id,
            user_id: None,
            display_name: None,
            description: None,
            username: None,
            icon_url: None,
        }
    }
}
