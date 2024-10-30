# \ReactionsApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_reaction**](ReactionsApi.md#delete_reaction) | **DELETE** /api/v4/users/{user_id}/posts/{post_id}/reactions/{emoji_name} | Remove a reaction from a post
[**get_bulk_reactions**](ReactionsApi.md#get_bulk_reactions) | **POST** /api/v4/posts/ids/reactions | Bulk get the reaction for posts
[**get_reactions**](ReactionsApi.md#get_reactions) | **GET** /api/v4/posts/{post_id}/reactions | Get a list of reactions to a post
[**save_reaction**](ReactionsApi.md#save_reaction) | **POST** /api/v4/reactions | Create a reaction



## delete_reaction

> models::StatusOk delete_reaction(user_id, post_id, emoji_name)
Remove a reaction from a post

Deletes a reaction made by a user from the given post. ##### Permissions Must be user or have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | ID of the user | [required] |
**post_id** | **String** | ID of the post | [required] |
**emoji_name** | **String** | emoji name | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_bulk_reactions

> std::collections::HashMap<String, Vec<models::Reaction>> get_bulk_reactions(request_body)
Bulk get the reaction for posts

Get a list of reactions made by all users to a given post. ##### Permissions Must have `read_channel` permission for the channel the post is in.  __Minimum server version__: 5.8 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_body** | [**Vec<String>**](String.md) | Array of post IDs | [required] |

### Return type

[**std::collections::HashMap<String, Vec<models::Reaction>>**](Vec.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_reactions

> Vec<models::Reaction> get_reactions(post_id)
Get a list of reactions to a post

Get a list of reactions made by all users to a given post. ##### Permissions Must have `read_channel` permission for the channel the post is in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_id** | **String** | ID of a post | [required] |

### Return type

[**Vec<models::Reaction>**](Reaction.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## save_reaction

> models::Reaction save_reaction(reaction)
Create a reaction

Create a reaction. ##### Permissions Must have `read_channel` permission for the channel the post is in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**reaction** | [**Reaction**](Reaction.md) | The user's reaction with its post_id, user_id, and emoji_name fields set | [required] |

### Return type

[**models::Reaction**](Reaction.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

