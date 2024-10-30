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
pub struct PluginManifest {
    /// Globally unique identifier that represents the plugin.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Name of the plugin.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description of what the plugin is and does.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Version number of the plugin.
    #[serde(rename = "version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The minimum Mattermost server version required for the plugin.  Available as server version 5.6.
    #[serde(rename = "min_server_version", skip_serializing_if = "Option::is_none")]
    pub min_server_version: Option<String>,
    #[serde(rename = "backend", skip_serializing_if = "Option::is_none")]
    pub backend: Option<Box<models::PluginManifestBackend>>,
    #[serde(rename = "server", skip_serializing_if = "Option::is_none")]
    pub server: Option<Box<models::PluginManifestServer>>,
    #[serde(rename = "webapp", skip_serializing_if = "Option::is_none")]
    pub webapp: Option<Box<models::PluginManifestWebapp>>,
    /// Settings schema used to define the System Console UI for the plugin.
    #[serde(rename = "settings_schema", skip_serializing_if = "Option::is_none")]
    pub settings_schema: Option<serde_json::Value>,
}

impl PluginManifest {
    pub fn new() -> PluginManifest {
        PluginManifest {
            id: None,
            name: None,
            description: None,
            version: None,
            min_server_version: None,
            backend: None,
            server: None,
            webapp: None,
            settings_schema: None,
        }
    }
}
