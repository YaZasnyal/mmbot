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
pub struct GetChecklistAutocomplete200ResponseInner {
    /// A string containing a pair of integers separated by a space. The first integer is the index of the checklist; the second is the index of the item within the checklist.
    #[serde(rename = "item")]
    pub item: String,
    /// The title of the corresponding item.
    #[serde(rename = "hint")]
    pub hint: String,
    /// Always the value \"Check/uncheck this item\".
    #[serde(rename = "helptext")]
    pub helptext: String,
}

impl GetChecklistAutocomplete200ResponseInner {
    pub fn new(item: String, hint: String, helptext: String) -> GetChecklistAutocomplete200ResponseInner {
        GetChecklistAutocomplete200ResponseInner {
            item,
            hint,
            helptext,
        }
    }
}

