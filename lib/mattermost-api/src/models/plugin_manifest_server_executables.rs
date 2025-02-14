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

/// PluginManifestServerExecutables : Paths to executable binaries, specifying multiple entry points for different platforms when bundled together in a single plugin.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginManifestServerExecutables {
    #[serde(rename = "linux-amd64", skip_serializing_if = "Option::is_none")]
    pub linux_amd64: Option<String>,
    #[serde(rename = "darwin-amd64", skip_serializing_if = "Option::is_none")]
    pub darwin_amd64: Option<String>,
    #[serde(rename = "windows-amd64", skip_serializing_if = "Option::is_none")]
    pub windows_amd64: Option<String>,
}

impl PluginManifestServerExecutables {
    /// Paths to executable binaries, specifying multiple entry points for different platforms when bundled together in a single plugin.
    pub fn new() -> PluginManifestServerExecutables {
        PluginManifestServerExecutables {
            linux_amd64: None,
            darwin_amd64: None,
            windows_amd64: None,
        }
    }
}
