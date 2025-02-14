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
pub struct SubmitPerformanceReportRequestCountersInner {
    /// The name of the counter
    #[serde(rename = "metric")]
    pub metric: String,
    /// The value to increment the counter by
    #[serde(rename = "value")]
    pub value: f64,
    /// The time that the counter was incremented
    #[serde(rename = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    /// Labels to be applied to this metric when recorded by the metrics backend
    #[serde(rename = "labels", skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
}

impl SubmitPerformanceReportRequestCountersInner {
    pub fn new(metric: String, value: f64) -> SubmitPerformanceReportRequestCountersInner {
        SubmitPerformanceReportRequestCountersInner {
            metric,
            value,
            timestamp: None,
            labels: None,
        }
    }
}
