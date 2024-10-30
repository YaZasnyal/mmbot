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
pub struct Scheme {
    /// The unique identifier of the scheme.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The human readable name for the scheme.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// A human readable description of the scheme.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The time at which the scheme was created.
    #[serde(rename = "create_at", skip_serializing_if = "Option::is_none")]
    pub create_at: Option<i64>,
    /// The time at which the scheme was last updated.
    #[serde(rename = "update_at", skip_serializing_if = "Option::is_none")]
    pub update_at: Option<i64>,
    /// The time at which the scheme was deleted.
    #[serde(rename = "delete_at", skip_serializing_if = "Option::is_none")]
    pub delete_at: Option<i64>,
    /// The scope to which this scheme can be applied, either \"team\" or \"channel\".
    #[serde(rename = "scope", skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// The id of the default team admin role for this scheme.
    #[serde(
        rename = "default_team_admin_role",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_team_admin_role: Option<String>,
    /// The id of the default team user role for this scheme.
    #[serde(
        rename = "default_team_user_role",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_team_user_role: Option<String>,
    /// The id of the default channel admin role for this scheme.
    #[serde(
        rename = "default_channel_admin_role",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_channel_admin_role: Option<String>,
    /// The id of the default channel user role for this scheme.
    #[serde(
        rename = "default_channel_user_role",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_channel_user_role: Option<String>,
}

impl Scheme {
    pub fn new() -> Scheme {
        Scheme {
            id: None,
            name: None,
            description: None,
            create_at: None,
            update_at: None,
            delete_at: None,
            scope: None,
            default_team_admin_role: None,
            default_team_user_role: None,
            default_channel_admin_role: None,
            default_channel_user_role: None,
        }
    }
}
