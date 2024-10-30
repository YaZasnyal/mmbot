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
pub struct Command {
    /// The ID of the slash command
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The token which is used to verify the source of the payload
    #[serde(rename = "token", skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// The time in milliseconds the command was created
    #[serde(rename = "create_at", skip_serializing_if = "Option::is_none")]
    pub create_at: Option<i32>,
    /// The time in milliseconds the command was last updated
    #[serde(rename = "update_at", skip_serializing_if = "Option::is_none")]
    pub update_at: Option<i64>,
    /// The time in milliseconds the command was deleted, 0 if never deleted
    #[serde(rename = "delete_at", skip_serializing_if = "Option::is_none")]
    pub delete_at: Option<i64>,
    /// The user id for the commands creator
    #[serde(rename = "creator_id", skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<String>,
    /// The team id for which this command is configured
    #[serde(rename = "team_id", skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    /// The string that triggers this command
    #[serde(rename = "trigger", skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    /// Is the trigger done with HTTP Get ('G') or HTTP Post ('P')
    #[serde(rename = "method", skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// What is the username for the response post
    #[serde(rename = "username", skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// The url to find the icon for this users avatar
    #[serde(rename = "icon_url", skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Use auto complete for this command
    #[serde(rename = "auto_complete", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub auto_complete: Option<bool>,
    /// The description for this command shown when selecting the command
    #[serde(rename = "auto_complete_desc", skip_serializing_if = "Option::is_none")]
    pub auto_complete_desc: Option<String>,
    /// The hint for this command
    #[serde(rename = "auto_complete_hint", skip_serializing_if = "Option::is_none")]
    pub auto_complete_hint: Option<String>,
    /// Display name for the command
    #[serde(rename = "display_name", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Description for this command
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The URL that is triggered
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl Command {
    pub fn new() -> Command {
        Command {
            id: None,
            token: None,
            create_at: None,
            update_at: None,
            delete_at: None,
            creator_id: None,
            team_id: None,
            trigger: None,
            method: None,
            username: None,
            icon_url: None,
            auto_complete: None,
            auto_complete_desc: None,
            auto_complete_hint: None,
            display_name: None,
            description: None,
            url: None,
        }
    }
}

