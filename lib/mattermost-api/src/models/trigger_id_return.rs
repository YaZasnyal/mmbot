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
pub struct TriggerIdReturn {
    /// The trigger_id returned by the slash command.
    #[serde(rename = "trigger_id")]
    pub trigger_id: String,
}

impl TriggerIdReturn {
    pub fn new(trigger_id: String) -> TriggerIdReturn {
        TriggerIdReturn {
            trigger_id,
        }
    }
}

