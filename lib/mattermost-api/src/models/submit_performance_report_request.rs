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
pub struct SubmitPerformanceReportRequest {
    /// An identifier for the schema of the data being submitted which currently must be \"0.1.0\"
    #[serde(rename = "version")]
    pub version: String,
    /// Not currently used
    #[serde(rename = "client_id", skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// Labels to be applied to all metrics when recorded by the metrics backend
    #[serde(rename = "labels", skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    /// The time in milliseconds of the first metric in this report
    #[serde(rename = "start")]
    pub start: i64,
    /// The time in milliseconds of the last metric in this report
    #[serde(rename = "end")]
    pub end: i64,
    /// An array of counter metrics to be reported
    #[serde(rename = "counters", skip_serializing_if = "Option::is_none")]
    pub counters: Option<Vec<models::SubmitPerformanceReportRequestCountersInner>>,
    /// An array of histogram measurements to be reported
    #[serde(rename = "histograms", skip_serializing_if = "Option::is_none")]
    pub histograms: Option<Vec<models::SubmitPerformanceReportRequestHistogramsInner>>,
}

impl SubmitPerformanceReportRequest {
    pub fn new(version: String, start: i64, end: i64) -> SubmitPerformanceReportRequest {
        SubmitPerformanceReportRequest {
            version,
            client_id: None,
            labels: None,
            start,
            end,
            counters: None,
            histograms: None,
        }
    }
}
