# \EmojiApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**autocomplete_emoji**](EmojiApi.md#autocomplete_emoji) | **GET** /api/v4/emoji/autocomplete | Autocomplete custom emoji
[**create_emoji**](EmojiApi.md#create_emoji) | **POST** /api/v4/emoji | Create a custom emoji
[**delete_emoji**](EmojiApi.md#delete_emoji) | **DELETE** /api/v4/emoji/{emoji_id} | Delete a custom emoji
[**get_emoji**](EmojiApi.md#get_emoji) | **GET** /api/v4/emoji/{emoji_id} | Get a custom emoji
[**get_emoji_by_name**](EmojiApi.md#get_emoji_by_name) | **GET** /api/v4/emoji/name/{emoji_name} | Get a custom emoji by name
[**get_emoji_image**](EmojiApi.md#get_emoji_image) | **GET** /api/v4/emoji/{emoji_id}/image | Get custom emoji image
[**get_emoji_list**](EmojiApi.md#get_emoji_list) | **GET** /api/v4/emoji | Get a list of custom emoji
[**get_emojis_by_names**](EmojiApi.md#get_emojis_by_names) | **POST** /api/v4/emoji/names | Get custom emojis by name
[**search_emoji**](EmojiApi.md#search_emoji) | **POST** /api/v4/emoji/search | Search custom emoji



## autocomplete_emoji

> models::Emoji autocomplete_emoji(name)
Autocomplete custom emoji

Get a list of custom emoji with names starting with or matching the provided name. Returns a maximum of 100 results. ##### Permissions Must be authenticated.  __Minimum server version__: 4.7 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | The emoji name to search. | [required] |

### Return type

[**models::Emoji**](Emoji.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_emoji

> models::Emoji create_emoji(image, emoji)
Create a custom emoji

Create a custom emoji for the team. ##### Permissions Must be authenticated. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**image** | **std::path::PathBuf** | A file to be uploaded | [required] |
**emoji** | **String** | A JSON object containing a `name` field with the name of the emoji and a `creator_id` field with the id of the authenticated user. | [required] |

### Return type

[**models::Emoji**](Emoji.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_emoji

> models::Emoji delete_emoji(emoji_id)
Delete a custom emoji

Delete a custom emoji. ##### Permissions Must have the `manage_team` or `manage_system` permissions or be the user who created the emoji. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**emoji_id** | **String** | Emoji GUID | [required] |

### Return type

[**models::Emoji**](Emoji.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_emoji

> models::Emoji get_emoji(emoji_id)
Get a custom emoji

Get some metadata for a custom emoji. ##### Permissions Must be authenticated. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**emoji_id** | **String** | Emoji GUID | [required] |

### Return type

[**models::Emoji**](Emoji.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_emoji_by_name

> models::Emoji get_emoji_by_name(emoji_name)
Get a custom emoji by name

Get some metadata for a custom emoji using its name. ##### Permissions Must be authenticated.  __Minimum server version__: 4.7 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**emoji_name** | **String** | Emoji name | [required] |

### Return type

[**models::Emoji**](Emoji.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_emoji_image

> get_emoji_image(emoji_id)
Get custom emoji image

Get the image for a custom emoji. ##### Permissions Must be authenticated. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**emoji_id** | **String** | Emoji GUID | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_emoji_list

> models::Emoji get_emoji_list(page, per_page, sort)
Get a list of custom emoji

Get a page of metadata for custom emoji on the system. Since server version 4.7, sort using the `sort` query parameter. ##### Permissions Must be authenticated. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of emojis per page. |  |[default to 60]
**sort** | Option<**String**> | Either blank for no sorting or \"name\" to sort by emoji names. Minimum server version for sorting is 4.7. |  |[default to ]

### Return type

[**models::Emoji**](Emoji.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_emojis_by_names

> Vec<models::User> get_emojis_by_names(request_body)
Get custom emojis by name

Get a list of custom emoji based on a provided list of emoji names. A maximum of 200 results are returned. ##### Permissions Must be authenticated. __Minimum server version__: 9.2 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_body** | [**Vec<String>**](String.md) | List of emoji names | [required] |

### Return type

[**Vec<models::User>**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_emoji

> Vec<models::Emoji> search_emoji(search_emoji_request)
Search custom emoji

Search for custom emoji by name based on search criteria provided in the request body. A maximum of 200 results are returned. ##### Permissions Must be authenticated.  __Minimum server version__: 4.7 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**search_emoji_request** | [**SearchEmojiRequest**](SearchEmojiRequest.md) | Search criteria | [required] |

### Return type

[**Vec<models::Emoji>**](Emoji.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

