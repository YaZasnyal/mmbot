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
pub struct CreatePlaybook201Response {
    #[serde(rename = "id")]
    pub id: String,
}

impl CreatePlaybook201Response {
    pub fn new(id: String) -> CreatePlaybook201Response {
        CreatePlaybook201Response { id }
    }
}
