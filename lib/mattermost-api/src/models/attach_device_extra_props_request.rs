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
pub struct AttachDeviceExtraPropsRequest {
    /// Mobile device id. For Android prefix the id with `android:` and Apple with `apple:`
    #[serde(rename = "device_id", skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    /// Whether the mobile device has notifications disabled. Accepted values are \"true\" or \"false\".
    #[serde(
        rename = "deviceNotificationDisabled",
        skip_serializing_if = "Option::is_none"
    )]
    pub device_notification_disabled: Option<String>,
    /// Mobile app version. The version must be parseable as a semver.
    #[serde(rename = "mobileVersion", skip_serializing_if = "Option::is_none")]
    pub mobile_version: Option<String>,
}

impl AttachDeviceExtraPropsRequest {
    pub fn new() -> AttachDeviceExtraPropsRequest {
        AttachDeviceExtraPropsRequest {
            device_id: None,
            device_notification_disabled: None,
            mobile_version: None,
        }
    }
}
