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
pub struct UserReport {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The time in milliseconds a user was created
    #[serde(rename = "create_at", skip_serializing_if = "Option::is_none")]
    pub create_at: Option<i64>,
    #[serde(rename = "username", skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(rename = "email", skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Calculated display name based on user
    #[serde(rename = "display_name", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Last time the user was logged in
    #[serde(rename = "last_login_at", skip_serializing_if = "Option::is_none")]
    pub last_login_at: Option<i64>,
    /// Last time the user's status was updated
    #[serde(rename = "last_status_at", skip_serializing_if = "Option::is_none")]
    pub last_status_at: Option<i64>,
    /// Last time the user made a post within the given date range
    #[serde(rename = "last_post_date", skip_serializing_if = "Option::is_none")]
    pub last_post_date: Option<i64>,
    /// Total number of days a user posted within the given date range
    #[serde(rename = "days_active", skip_serializing_if = "Option::is_none")]
    pub days_active: Option<i32>,
    /// Total number of posts made by a user within the given date range
    #[serde(rename = "total_posts", skip_serializing_if = "Option::is_none")]
    pub total_posts: Option<i32>,
}

impl UserReport {
    pub fn new() -> UserReport {
        UserReport {
            id: None,
            create_at: None,
            username: None,
            email: None,
            display_name: None,
            last_login_at: None,
            last_status_at: None,
            last_post_date: None,
            days_active: None,
            total_posts: None,
        }
    }
}

