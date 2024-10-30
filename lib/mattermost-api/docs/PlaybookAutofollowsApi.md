# \PlaybookAutofollowsApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_auto_follows**](PlaybookAutofollowsApi.md#get_auto_follows) | **GET** /plugins/playbooks/api/v0/playbooks/{id}/autofollows | Get the list of followers' user IDs of a playbook



## get_auto_follows

> models::PlaybookAutofollows get_auto_follows(id)
Get the list of followers' user IDs of a playbook

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook to retrieve followers from. | [required] |

### Return type

[**models::PlaybookAutofollows**](PlaybookAutofollows.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

