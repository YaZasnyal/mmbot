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
pub struct CreatePlaybookRunFromPostRequest {
    /// The name of the playbook run.
    #[serde(rename = "name")]
    pub name: String,
    /// The description of the playbook run.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The identifier of the user who is commanding the playbook run.
    #[serde(rename = "owner_user_id")]
    pub owner_user_id: String,
    /// The identifier of the team where the playbook run's channel is in.
    #[serde(rename = "team_id")]
    pub team_id: String,
    /// If the playbook run was created from a post, this field contains the identifier of such post. If not, this field is empty.
    #[serde(rename = "post_id", skip_serializing_if = "Option::is_none")]
    pub post_id: Option<String>,
    /// The identifier of the playbook with from which this playbook run was created.
    #[serde(rename = "playbook_id")]
    pub playbook_id: String,
}

impl CreatePlaybookRunFromPostRequest {
    pub fn new(name: String, owner_user_id: String, team_id: String, playbook_id: String) -> CreatePlaybookRunFromPostRequest {
        CreatePlaybookRunFromPostRequest {
            name,
            description: None,
            owner_user_id,
            team_id,
            post_id: None,
            playbook_id,
        }
    }
}

