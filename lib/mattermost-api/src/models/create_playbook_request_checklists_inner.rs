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
pub struct CreatePlaybookRequestChecklistsInner {
    /// The title of the checklist.
    #[serde(rename = "title")]
    pub title: String,
    /// The list of tasks to do.
    #[serde(rename = "items")]
    pub items: Vec<models::CreatePlaybookRequestChecklistsInnerItemsInner>,
}

impl CreatePlaybookRequestChecklistsInner {
    pub fn new(
        title: String,
        items: Vec<models::CreatePlaybookRequestChecklistsInnerItemsInner>,
    ) -> CreatePlaybookRequestChecklistsInner {
        CreatePlaybookRequestChecklistsInner { title, items }
    }
}
