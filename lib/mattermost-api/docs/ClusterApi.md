# \ClusterApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_cluster_status**](ClusterApi.md#get_cluster_status) | **GET** /api/v4/cluster/status | Get cluster status



## get_cluster_status

> Vec<Vec<models::ClusterInfoInner>> get_cluster_status()
Get cluster status

Get a list of all healthy nodes, including local information and status of each one. If a node is not present, it means it is not healthy. ##### Permissions Must have `manage_system` permission. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<Vec<models::ClusterInfoInner>>**](Vec.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

