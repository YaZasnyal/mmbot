# \PermissionsApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_ancillary_permissions_post**](PermissionsApi.md#get_ancillary_permissions_post) | **POST** /api/v4/permissions/ancillary | Return all system console subsection ancillary permissions



## get_ancillary_permissions_post

> Vec<String> get_ancillary_permissions_post(request_body)
Return all system console subsection ancillary permissions

Returns all the ancillary permissions for the corresponding system console subsection permissions appended to the requested permission subsections. __Minimum server version__: 9.10 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_body** | [**Vec<String>**](String.md) | List of subsection permissions | [required] |

### Return type

**Vec<String>**

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

