# \BookmarksApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_channel_bookmark**](BookmarksApi.md#create_channel_bookmark) | **POST** /api/v4/channels/{channel_id}/bookmarks | Create channel bookmark
[**delete_channel_bookmark**](BookmarksApi.md#delete_channel_bookmark) | **DELETE** /api/v4/channels/{channel_id}/bookmarks/{bookmark_id} | Delete channel bookmark
[**list_channel_bookmarks_for_channel**](BookmarksApi.md#list_channel_bookmarks_for_channel) | **GET** /api/v4/channels/{channel_id}/bookmarks | Get channel bookmarks for Channel
[**update_channel_bookmark**](BookmarksApi.md#update_channel_bookmark) | **PATCH** /api/v4/channels/{channel_id}/bookmarks/{bookmark_id} | Update channel bookmark
[**update_channel_bookmark_sort_order**](BookmarksApi.md#update_channel_bookmark_sort_order) | **POST** /api/v4/channels/{channel_id}/bookmarks/{bookmark_id}/sort_order | Update channel bookmark's order



## create_channel_bookmark

> models::ChannelBookmarkWithFileInfo create_channel_bookmark(channel_id, create_channel_bookmark_request)
Create channel bookmark

Creates a new channel bookmark for this channel.  __Minimum server version__: 9.5  ##### Permissions Must have the `add_bookmark_public_channel` or `add_bookmark_private_channel` depending on the channel type. If the channel is a DM or GM, must be a non-guest member. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**channel_id** | **String** | Channel GUID | [required] |
**create_channel_bookmark_request** | [**CreateChannelBookmarkRequest**](CreateChannelBookmarkRequest.md) | Channel Bookmark object to be created | [required] |

### Return type

[**models::ChannelBookmarkWithFileInfo**](ChannelBookmarkWithFileInfo.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_channel_bookmark

> models::ChannelBookmarkWithFileInfo delete_channel_bookmark(channel_id, bookmark_id)
Delete channel bookmark

Archives a channel bookmark. This will set the `deleteAt` to the current timestamp in the database.  __Minimum server version__: 9.5  ##### Permissions Must have the `delete_bookmark_public_channel` or `delete_bookmark_private_channel` depending on the channel type. If the channel is a DM or GM, must be a non-guest member. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**channel_id** | **String** | Channel GUID | [required] |
**bookmark_id** | **String** | Bookmark GUID | [required] |

### Return type

[**models::ChannelBookmarkWithFileInfo**](ChannelBookmarkWithFileInfo.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_channel_bookmarks_for_channel

> Vec<models::ChannelBookmarkWithFileInfo> list_channel_bookmarks_for_channel(channel_id, bookmarks_since)
Get channel bookmarks for Channel

__Minimum server version__: 9.5 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**channel_id** | **String** | Channel GUID | [required] |
**bookmarks_since** | Option<**f64**> | Timestamp to filter the bookmarks with. If set, the endpoint returns bookmarks that have been added, updated or deleted since its value  |  |

### Return type

[**Vec<models::ChannelBookmarkWithFileInfo>**](ChannelBookmarkWithFileInfo.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_channel_bookmark

> models::UpdateChannelBookmarkResponse update_channel_bookmark(channel_id, bookmark_id, update_channel_bookmark_request)
Update channel bookmark

Partially update a channel bookmark by providing only the fields you want to update. Ommited fields will not be updated. The fields that can be updated are defined in the request body, all other provided fields will be ignored.  __Minimum server version__: 9.5  ##### Permissions Must have the `edit_bookmark_public_channel` or `edit_bookmark_private_channel` depending on the channel type. If the channel is a DM or GM, must be a non-guest member. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**channel_id** | **String** | Channel GUID | [required] |
**bookmark_id** | **String** | Bookmark GUID | [required] |
**update_channel_bookmark_request** | [**UpdateChannelBookmarkRequest**](UpdateChannelBookmarkRequest.md) | Channel Bookmark object to be updated | [required] |

### Return type

[**models::UpdateChannelBookmarkResponse**](UpdateChannelBookmarkResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_channel_bookmark_sort_order

> Vec<models::ChannelBookmarkWithFileInfo> update_channel_bookmark_sort_order(channel_id, bookmark_id, body)
Update channel bookmark's order

Updates the order of a channel bookmark, setting its new order from the parameters and updating the rest of the bookmarks of the channel to accomodate for this change.  __Minimum server version__: 9.5  ##### Permissions Must have the `order_bookmark_public_channel` or `order_bookmark_private_channel` depending on the channel type. If the channel is a DM or GM, must be a non-guest member. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**channel_id** | **String** | Channel GUID | [required] |
**bookmark_id** | **String** | Bookmark GUID | [required] |
**body** | Option<**f64**> |  |  |

### Return type

[**Vec<models::ChannelBookmarkWithFileInfo>**](ChannelBookmarkWithFileInfo.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

