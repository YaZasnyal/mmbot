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
pub struct PlaybookList {
    /// The total number of playbooks in the list, regardless of the paging.
    #[serde(rename = "total_count", skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i32>,
    /// The total number of pages. This depends on the total number of playbooks in the database and the per_page parameter sent with the request.
    #[serde(rename = "page_count", skip_serializing_if = "Option::is_none")]
    pub page_count: Option<i32>,
    /// A boolean describing whether there are more pages after the currently returned.
    #[serde(rename = "has_more", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub has_more: Option<bool>,
    /// The playbooks in this page.
    #[serde(rename = "items", skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<models::Playbook>>,
}

impl PlaybookList {
    pub fn new() -> PlaybookList {
        PlaybookList {
            total_count: None,
            page_count: None,
            has_more: None,
            items: None,
        }
    }
}

