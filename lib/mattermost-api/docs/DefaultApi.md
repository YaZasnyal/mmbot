# \DefaultApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**submit_performance_report**](DefaultApi.md#submit_performance_report) | **POST** /api/v4/client_perf | Report client performance metrics



## submit_performance_report

> models::StatusOk submit_performance_report(submit_performance_report_request)
Report client performance metrics

Uploads client performance measurements to the server as part of the Client Performance Monitoring feature. __Minimum server version__: 9.9.0 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**submit_performance_report_request** | Option<[**SubmitPerformanceReportRequest**](SubmitPerformanceReportRequest.md)> |  |  |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

