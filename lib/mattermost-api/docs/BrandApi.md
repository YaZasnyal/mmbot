# \BrandApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_brand_image**](BrandApi.md#delete_brand_image) | **DELETE** /api/v4/brand/image | Delete current brand image
[**get_brand_image**](BrandApi.md#get_brand_image) | **GET** /api/v4/brand/image | Get brand image
[**upload_brand_image**](BrandApi.md#upload_brand_image) | **POST** /api/v4/brand/image | Upload brand image



## delete_brand_image

> models::StatusOk delete_brand_image()
Delete current brand image

Deletes the previously uploaded brand image. Returns 404 if no brand image has been uploaded. ##### Permissions Must have `manage_system` permission. __Minimum server version: 5.6__ 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_brand_image

> String get_brand_image()
Get brand image

Get the previously uploaded brand image. Returns 404 if no brand image has been uploaded. ##### Permissions No permission required. 

### Parameters

This endpoint does not need any parameter.

### Return type

**String**

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upload_brand_image

> models::StatusOk upload_brand_image(image)
Upload brand image

Uploads a brand image. ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**image** | **std::path::PathBuf** | The image to be uploaded | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

