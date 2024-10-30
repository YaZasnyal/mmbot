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

/// LdapGroup : A LDAP group
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct LdapGroup {
    #[serde(rename = "has_syncables", skip_serializing_if = "Option::is_none", default, deserialize_with = "bool_parser::deserialize_option_bool")]
    pub has_syncables: Option<bool>,
    #[serde(rename = "mattermost_group_id", skip_serializing_if = "Option::is_none")]
    pub mattermost_group_id: Option<String>,
    #[serde(rename = "primary_key", skip_serializing_if = "Option::is_none")]
    pub primary_key: Option<String>,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl LdapGroup {
    /// A LDAP group
    pub fn new() -> LdapGroup {
        LdapGroup {
            has_syncables: None,
            mattermost_group_id: None,
            primary_key: None,
            name: None,
        }
    }
}

