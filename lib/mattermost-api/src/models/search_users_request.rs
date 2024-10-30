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
pub struct SearchUsersRequest {
    /// The term to match against username, full name, nickname and email
    #[serde(rename = "term")]
    pub term: String,
    /// If provided, only search users on this team
    #[serde(rename = "team_id", skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    /// If provided, only search users not on this team
    #[serde(rename = "not_in_team_id", skip_serializing_if = "Option::is_none")]
    pub not_in_team_id: Option<String>,
    /// If provided, only search users in this channel
    #[serde(rename = "in_channel_id", skip_serializing_if = "Option::is_none")]
    pub in_channel_id: Option<String>,
    /// If provided, only search users not in this channel. Must specifiy `team_id` when using this option
    #[serde(rename = "not_in_channel_id", skip_serializing_if = "Option::is_none")]
    pub not_in_channel_id: Option<String>,
    /// If provided, only search users in this group. Must have `manage_system` permission.
    #[serde(rename = "in_group_id", skip_serializing_if = "Option::is_none")]
    pub in_group_id: Option<String>,
    /// When used with `not_in_channel_id` or `not_in_team_id`, returns only the users that are allowed to join the channel or team based on its group constrains.
    #[serde(
        rename = "group_constrained",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub group_constrained: Option<bool>,
    /// When `true`, include deactivated users in the results
    #[serde(
        rename = "allow_inactive",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub allow_inactive: Option<bool>,
    /// Set this to `true` if you would like to search for users that are not on a team. This option takes precendence over `team_id`, `in_channel_id`, and `not_in_channel_id`.
    #[serde(
        rename = "without_team",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub without_team: Option<bool>,
    /// The maximum number of users to return in the results  __Available as of server version 5.6. Defaults to `100` if not provided or on an earlier server version.__
    #[serde(rename = "limit", skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

impl SearchUsersRequest {
    pub fn new(term: String) -> SearchUsersRequest {
        SearchUsersRequest {
            term,
            team_id: None,
            not_in_team_id: None,
            in_channel_id: None,
            not_in_channel_id: None,
            in_group_id: None,
            group_constrained: None,
            allow_inactive: None,
            without_team: None,
            limit: None,
        }
    }
}
