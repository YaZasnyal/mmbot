# \IpApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**apply_ip_filters**](IpApi.md#apply_ip_filters) | **POST** /api/v4/ip_filtering | Get all IP filters
[**get_ip_filters**](IpApi.md#get_ip_filters) | **GET** /api/v4/ip_filtering | Get all IP filters
[**my_ip**](IpApi.md#my_ip) | **GET** /api/v4/ip_filtering/my_ip | Get all IP filters



## apply_ip_filters

> Vec<models::AllowedIpRange> apply_ip_filters(allowed_ip_range)
Get all IP filters

Adjust IP Filters applied to the workspace __Minimum server version__: 9.1 __Note:__ This is intended for internal use and only applicable to Cloud workspaces 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**allowed_ip_range** | [**Vec<models::AllowedIpRange>**](AllowedIPRange.md) | IP Filters to apply | [required] |

### Return type

[**Vec<models::AllowedIpRange>**](AllowedIPRange.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_ip_filters

> Vec<models::AllowedIpRange> get_ip_filters()
Get all IP filters

Retrieve a list of IP filters applied to the workspace __Minimum server version__: 9.1 __Note:__ This is intended for internal use and only applicable to Cloud workspaces 

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::AllowedIpRange>**](AllowedIPRange.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## my_ip

> models::MyIp200Response my_ip()
Get all IP filters

Retrieve your current IP address as seen by the workspace __Minimum server version__: 9.1 __Note:__ This is intended for internal use and only applicable to Cloud workspaces 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::MyIp200Response**](MyIP_200_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

