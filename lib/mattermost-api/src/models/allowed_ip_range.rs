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
pub struct AllowedIpRange {
    /// An IP address range in CIDR notation
    #[serde(rename = "CIDRBlock", skip_serializing_if = "Option::is_none")]
    pub cidr_block: Option<String>,
    /// A description for the CIDRBlock
    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl AllowedIpRange {
    pub fn new() -> AllowedIpRange {
        AllowedIpRange {
            cidr_block: None,
            description: None,
        }
    }
}

