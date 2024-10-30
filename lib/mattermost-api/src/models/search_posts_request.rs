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
pub struct SearchPostsRequest {
    /// The search terms as inputed by the user. To search for posts from a user include `from:someusername`, using a user's username. To search in a specific channel include `in:somechannel`, using the channel name (not the display name).
    #[serde(rename = "terms")]
    pub terms: String,
    /// Set to true if an Or search should be performed vs an And search.
    #[serde(rename = "is_or_search")]
    pub is_or_search: bool,
    /// Offset from UTC of user timezone for date searches.
    #[serde(rename = "time_zone_offset", skip_serializing_if = "Option::is_none")]
    pub time_zone_offset: Option<i32>,
    /// Set to true if deleted channels should be included in the search. (archived channels)
    #[serde(
        rename = "include_deleted_channels",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "bool_parser::deserialize_option_bool"
    )]
    pub include_deleted_channels: Option<bool>,
    /// The page to select. (Only works with Elasticsearch)
    #[serde(rename = "page", skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    /// The number of posts per page. (Only works with Elasticsearch)
    #[serde(rename = "per_page", skip_serializing_if = "Option::is_none")]
    pub per_page: Option<i32>,
}

impl SearchPostsRequest {
    pub fn new(terms: String, is_or_search: bool) -> SearchPostsRequest {
        SearchPostsRequest {
            terms,
            is_or_search,
            time_zone_offset: None,
            include_deleted_channels: None,
            page: None,
            per_page: None,
        }
    }
}
