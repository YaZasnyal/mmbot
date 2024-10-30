# \ReportsApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**start_batch_users_export**](ReportsApi.md#start_batch_users_export) | **POST** /api/v4/reports/users/export | Starts a job to export the users to a report file.



## start_batch_users_export

> Vec<models::UserReport> start_batch_users_export(date_range)
Starts a job to export the users to a report file.

Starts a job to export the users to a report file.  Must be a system admin to invoke this API. ##### Permissions Requires `sysconsole_read_user_management_users`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**date_range** | Option<**String**> | The date range of the post statistics to display. Must be one of (\"last30days\", \"previousmonth\", \"last6months\", \"alltime\"). Will default to 'alltime' if the input is not valid. |  |[default to alltime]

### Return type

[**Vec<models::UserReport>**](UserReport.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

