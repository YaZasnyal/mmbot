# \RootApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**acknowledge_notification**](RootApi.md#acknowledge_notification) | **POST** /api/v4/notifications/ack | Acknowledge receiving of a notification



## acknowledge_notification

> models::PushNotification acknowledge_notification()
Acknowledge receiving of a notification

__Minimum server version__: 3.10 ##### Permissions Must be logged in. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::PushNotification**](PushNotification.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

