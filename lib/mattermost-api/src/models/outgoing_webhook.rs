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
pub struct OutgoingWebhook {
    /// The unique identifier for this outgoing webhook
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The time in milliseconds a outgoing webhook was created
    #[serde(rename = "create_at", skip_serializing_if = "Option::is_none")]
    pub create_at: Option<i64>,
    /// The time in milliseconds a outgoing webhook was last updated
    #[serde(rename = "update_at", skip_serializing_if = "Option::is_none")]
    pub update_at: Option<i64>,
    /// The time in milliseconds a outgoing webhook was deleted
    #[serde(rename = "delete_at", skip_serializing_if = "Option::is_none")]
    pub delete_at: Option<i64>,
    /// The Id of the user who created the webhook
    #[serde(rename = "creator_id", skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<String>,
    /// The ID of the team that the webhook watchs
    #[serde(rename = "team_id", skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    /// The ID of a public channel that the webhook watchs
    #[serde(rename = "channel_id", skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// The description for this outgoing webhook
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The display name for this outgoing webhook
    #[serde(rename = "display_name", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// List of words for the webhook to trigger on
    #[serde(rename = "trigger_words", skip_serializing_if = "Option::is_none")]
    pub trigger_words: Option<Vec<String>>,
    /// When to trigger the webhook, `0` when a trigger word is present at all and `1` if the message starts with a trigger word
    #[serde(rename = "trigger_when", skip_serializing_if = "Option::is_none")]
    pub trigger_when: Option<i32>,
    /// The URLs to POST the payloads to when the webhook is triggered
    #[serde(rename = "callback_urls", skip_serializing_if = "Option::is_none")]
    pub callback_urls: Option<Vec<String>>,
    /// The format to POST the data in, either `application/json` or `application/x-www-form-urlencoded`
    #[serde(rename = "content_type", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

impl OutgoingWebhook {
    pub fn new() -> OutgoingWebhook {
        OutgoingWebhook {
            id: None,
            create_at: None,
            update_at: None,
            delete_at: None,
            creator_id: None,
            team_id: None,
            channel_id: None,
            description: None,
            display_name: None,
            trigger_words: None,
            trigger_when: None,
            callback_urls: None,
            content_type: None,
        }
    }
}
