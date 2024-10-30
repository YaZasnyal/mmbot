# SubmitPerformanceReportRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**version** | **String** | An identifier for the schema of the data being submitted which currently must be \"0.1.0\" | 
**client_id** | Option<**String**> | Not currently used | [optional]
**labels** | Option<**Vec<String>**> | Labels to be applied to all metrics when recorded by the metrics backend | [optional]
**start** | **i64** | The time in milliseconds of the first metric in this report | 
**end** | **i64** | The time in milliseconds of the last metric in this report | 
**counters** | Option<[**Vec<models::SubmitPerformanceReportRequestCountersInner>**](SubmitPerformanceReport_request_counters_inner.md)> | An array of counter metrics to be reported | [optional]
**histograms** | Option<[**Vec<models::SubmitPerformanceReportRequestHistogramsInner>**](SubmitPerformanceReport_request_histograms_inner.md)> | An array of histogram measurements to be reported | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


