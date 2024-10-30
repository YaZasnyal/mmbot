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
pub struct WebhookOnStatusUpdatePayload {
    /// A unique, 26 characters long, alphanumeric identifier for the playbook run.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The name of the playbook run.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The description of the playbook run.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// True if the playbook run is ongoing; false if the playbook run is ended.
    #[serde(rename = "is_active", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub is_active: Option<bool>,
    /// The identifier of the user that is commanding the playbook run.
    #[serde(rename = "owner_user_id", skip_serializing_if = "Option::is_none")]
    pub owner_user_id: Option<String>,
    /// The identifier of the team where the playbook run's channel is in.
    #[serde(rename = "team_id", skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    /// The identifier of the playbook run's channel.
    #[serde(rename = "channel_id", skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// The playbook run creation timestamp, formatted as the number of milliseconds since the Unix epoch.
    #[serde(rename = "create_at", skip_serializing_if = "Option::is_none")]
    pub create_at: Option<i64>,
    /// The playbook run finish timestamp, formatted as the number of milliseconds since the Unix epoch. It equals 0 if the playbook run is not finished.
    #[serde(rename = "end_at", skip_serializing_if = "Option::is_none")]
    pub end_at: Option<i64>,
    /// The playbook run deletion timestamp, formatted as the number of milliseconds since the Unix epoch. It equals 0 if the playbook run is not deleted.
    #[serde(rename = "delete_at", skip_serializing_if = "Option::is_none")]
    pub delete_at: Option<i64>,
    /// Zero-based index of the currently active stage.
    #[serde(rename = "active_stage", skip_serializing_if = "Option::is_none")]
    pub active_stage: Option<i32>,
    /// The title of the currently active stage.
    #[serde(rename = "active_stage_title", skip_serializing_if = "Option::is_none")]
    pub active_stage_title: Option<String>,
    /// If the playbook run was created from a post, this field contains the identifier of such post. If not, this field is empty.
    #[serde(rename = "post_id", skip_serializing_if = "Option::is_none")]
    pub post_id: Option<String>,
    /// The identifier of the playbook with from which this playbook run was created.
    #[serde(rename = "playbook_id", skip_serializing_if = "Option::is_none")]
    pub playbook_id: Option<String>,
    #[serde(rename = "checklists", skip_serializing_if = "Option::is_none")]
    pub checklists: Option<Vec<models::Checklist>>,
    /// Absolute URL to the playbook run's channel.
    #[serde(rename = "channel_url", skip_serializing_if = "Option::is_none")]
    pub channel_url: Option<String>,
    /// Absolute URL to the playbook run's details.
    #[serde(rename = "details_url", skip_serializing_if = "Option::is_none")]
    pub details_url: Option<String>,
}

impl WebhookOnStatusUpdatePayload {
    pub fn new() -> WebhookOnStatusUpdatePayload {
        WebhookOnStatusUpdatePayload {
            id: None,
            name: None,
            description: None,
            is_active: None,
            owner_user_id: None,
            team_id: None,
            channel_id: None,
            create_at: None,
            end_at: None,
            delete_at: None,
            active_stage: None,
            active_stage_title: None,
            post_id: None,
            playbook_id: None,
            checklists: None,
            channel_url: None,
            details_url: None,
        }
    }
}

