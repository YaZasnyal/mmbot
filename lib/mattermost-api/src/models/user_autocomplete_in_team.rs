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
pub struct UserAutocompleteInTeam {
    /// A list of user objects in the team
    #[serde(rename = "in_team", skip_serializing_if = "Option::is_none")]
    pub in_team: Option<Vec<models::User>>,
}

impl UserAutocompleteInTeam {
    pub fn new() -> UserAutocompleteInTeam {
        UserAutocompleteInTeam { in_team: None }
    }
}
