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
pub struct UserAutocompleteInChannel {
    /// A list of user objects in the channel
    #[serde(rename = "in_channel", skip_serializing_if = "Option::is_none")]
    pub in_channel: Option<Vec<models::User>>,
    /// A list of user objects not in the channel
    #[serde(rename = "out_of_channel", skip_serializing_if = "Option::is_none")]
    pub out_of_channel: Option<Vec<models::User>>,
}

impl UserAutocompleteInChannel {
    pub fn new() -> UserAutocompleteInChannel {
        UserAutocompleteInChannel {
            in_channel: None,
            out_of_channel: None,
        }
    }
}
