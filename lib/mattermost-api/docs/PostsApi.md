# \PostsApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_post**](PostsApi.md#create_post) | **POST** /api/v4/posts | Create a post
[**create_post_ephemeral**](PostsApi.md#create_post_ephemeral) | **POST** /api/v4/posts/ephemeral | Create a ephemeral post
[**delete_acknowledgement_for_post**](PostsApi.md#delete_acknowledgement_for_post) | **DELETE** /api/v4/users/{user_id}/posts/{post_id}/ack | Delete a post acknowledgement
[**delete_post**](PostsApi.md#delete_post) | **DELETE** /api/v4/posts/{post_id} | Delete a post
[**do_post_action**](PostsApi.md#do_post_action) | **POST** /api/v4/posts/{post_id}/actions/{action_id} | Perform a post action
[**get_file_infos_for_post**](PostsApi.md#get_file_infos_for_post) | **GET** /api/v4/posts/{post_id}/files/info | Get file info for post
[**get_flagged_posts_for_user**](PostsApi.md#get_flagged_posts_for_user) | **GET** /api/v4/users/{user_id}/posts/flagged | Get a list of flagged posts
[**get_post**](PostsApi.md#get_post) | **GET** /api/v4/posts/{post_id} | Get a post
[**get_post_thread**](PostsApi.md#get_post_thread) | **GET** /api/v4/posts/{post_id}/thread | Get a thread
[**get_posts_around_last_unread**](PostsApi.md#get_posts_around_last_unread) | **GET** /api/v4/users/{user_id}/channels/{channel_id}/posts/unread | Get posts around oldest unread
[**get_posts_by_ids**](PostsApi.md#get_posts_by_ids) | **POST** /api/v4/posts/ids | Get posts by a list of ids
[**get_posts_for_channel**](PostsApi.md#get_posts_for_channel) | **GET** /api/v4/channels/{channel_id}/posts | Get posts for a channel
[**move_thread**](PostsApi.md#move_thread) | **POST** /api/v4/posts/{post_id}/move | Move a post (and any posts within that post's thread)
[**patch_post**](PostsApi.md#patch_post) | **PUT** /api/v4/posts/{post_id}/patch | Patch a post
[**pin_post**](PostsApi.md#pin_post) | **POST** /api/v4/posts/{post_id}/pin | Pin a post to the channel
[**save_acknowledgement_for_post**](PostsApi.md#save_acknowledgement_for_post) | **POST** /api/v4/users/{user_id}/posts/{post_id}/ack | Acknowledge a post
[**search_posts**](PostsApi.md#search_posts) | **POST** /api/v4/teams/{team_id}/posts/search | Search for team posts
[**set_post_reminder**](PostsApi.md#set_post_reminder) | **POST** /api/v4/users/{user_id}/posts/{post_id}/reminder | Set a post reminder
[**set_post_unread**](PostsApi.md#set_post_unread) | **POST** /api/v4/users/{user_id}/posts/{post_id}/set_unread | Mark as unread from a post.
[**unpin_post**](PostsApi.md#unpin_post) | **POST** /api/v4/posts/{post_id}/unpin | Unpin a post to the channel
[**update_post**](PostsApi.md#update_post) | **PUT** /api/v4/posts/{post_id} | Update a post



## create_post

> models::Post create_post(create_post_request, set_online)
Create a post

Create a new post in a channel. To create the post as a comment on another post, provide `root_id`. ##### Permissions Must have `create_post` permission for the channel the post is being created in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_post_request** | [**CreatePostRequest**](CreatePostRequest.md) | Post object to create | [required] |
**set_online** | Option<**bool**> | Whether to set the user status as online or not. |  |

### Return type

[**models::Post**](Post.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_post_ephemeral

> models::Post create_post_ephemeral(create_post_ephemeral_request)
Create a ephemeral post

Create a new ephemeral post in a channel. ##### Permissions Must have `create_post_ephemeral` permission (currently only given to system admin) 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_post_ephemeral_request** | [**CreatePostEphemeralRequest**](CreatePostEphemeralRequest.md) | Ephemeral Post object to send | [required] |

### Return type

[**models::Post**](Post.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_acknowledgement_for_post

> models::StatusOk delete_acknowledgement_for_post(user_id, post_id)
Delete a post acknowledgement

Delete an acknowledgement form a post that you had previously acknowledged. ##### Permissions Must have `read_channel` permission for the channel the post is in.<br/> Must be logged in as the user or have `edit_other_users` permission.<br/> The post must have been acknowledged in the previous 5 minutes.  __Minimum server version__: 7.7 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**post_id** | **String** | Post GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_post

> models::StatusOk delete_post(post_id)
Delete a post

Soft deletes a post, by marking the post as deleted in the database. Soft deleted posts will not be returned in post queries. ##### Permissions Must be logged in as the user or have `delete_others_posts` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_id** | **String** | ID of the post to delete | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## do_post_action

> models::StatusOk do_post_action(post_id, action_id)
Perform a post action

Perform a post action, which allows users to interact with integrations through posts. ##### Permissions Must be authenticated and have the `read_channel` permission to the channel the post is in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_id** | **String** | Post GUID | [required] |
**action_id** | **String** | Action GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_file_infos_for_post

> Vec<models::FileInfo> get_file_infos_for_post(post_id, include_deleted)
Get file info for post

Gets a list of file information objects for the files attached to a post. ##### Permissions Must have `read_channel` permission for the channel the post is in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_id** | **String** | ID of the post | [required] |
**include_deleted** | Option<**bool**> | Defines if result should include deleted posts, must have 'manage_system' (admin) permission. |  |[default to false]

### Return type

[**Vec<models::FileInfo>**](FileInfo.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_flagged_posts_for_user

> Vec<models::PostList> get_flagged_posts_for_user(user_id, team_id, channel_id, page, per_page)
Get a list of flagged posts

Get a page of flagged posts of a user provided user id string. Selects from a channel, team, or all flagged posts by a user. Will only return posts from channels in which the user is member. ##### Permissions Must be user or have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | ID of the user | [required] |
**team_id** | Option<**String**> | Team ID |  |
**channel_id** | Option<**String**> | Channel ID |  |
**page** | Option<**i32**> | The page to select |  |[default to 0]
**per_page** | Option<**i32**> | The number of posts per page |  |[default to 60]

### Return type

[**Vec<models::PostList>**](PostList.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_post

> models::Post get_post(post_id, include_deleted)
Get a post

Get a single post. ##### Permissions Must have `read_channel` permission for the channel the post is in or if the channel is public, have the `read_public_channels` permission for the team. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_id** | **String** | ID of the post to get | [required] |
**include_deleted** | Option<**bool**> | Defines if result should include deleted posts, must have 'manage_system' (admin) permission. |  |[default to false]

### Return type

[**models::Post**](Post.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_post_thread

> models::PostList get_post_thread(post_id, per_page, from_post, from_create_at, direction, skip_fetch_threads, collapsed_threads, collapsed_threads_extended)
Get a thread

Get a post and the rest of the posts in the same thread. ##### Permissions Must have `read_channel` permission for the channel the post is in or if the channel is public, have the `read_public_channels` permission for the team. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_id** | **String** | ID of a post in the thread | [required] |
**per_page** | Option<**i32**> | The number of posts per page |  |[default to 0]
**from_post** | Option<**String**> | The post_id to return the next page of posts from |  |[default to ]
**from_create_at** | Option<**i32**> | The create_at timestamp to return the next page of posts from |  |[default to 0]
**direction** | Option<**String**> | The direction to return the posts. Either up or down. |  |[default to ]
**skip_fetch_threads** | Option<**bool**> | Whether to skip fetching threads or not |  |[default to false]
**collapsed_threads** | Option<**bool**> | Whether the client uses CRT or not |  |[default to false]
**collapsed_threads_extended** | Option<**bool**> | Whether to return the associated users as part of the response or not |  |[default to false]

### Return type

[**models::PostList**](PostList.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_posts_around_last_unread

> models::PostList get_posts_around_last_unread(user_id, channel_id, limit_before, limit_after, skip_fetch_threads, collapsed_threads, collapsed_threads_extended)
Get posts around oldest unread

Get the oldest unread post in the channel for the given user as well as the posts around it. The returned list is sorted in descending order (most recent post first). ##### Permissions Must be logged in as the user or have `edit_other_users` permission, and must have `read_channel` permission for the channel. __Minimum server version__: 5.14 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | ID of the user | [required] |
**channel_id** | **String** | The channel ID to get the posts for | [required] |
**limit_before** | Option<**i32**> | Number of posts before the oldest unread posts. Maximum is 200 posts if limit is set greater than that. |  |[default to 60]
**limit_after** | Option<**i32**> | Number of posts after and including the oldest unread post. Maximum is 200 posts if limit is set greater than that. |  |[default to 60]
**skip_fetch_threads** | Option<**bool**> | Whether to skip fetching threads or not |  |[default to false]
**collapsed_threads** | Option<**bool**> | Whether the client uses CRT or not |  |[default to false]
**collapsed_threads_extended** | Option<**bool**> | Whether to return the associated users as part of the response or not |  |[default to false]

### Return type

[**models::PostList**](PostList.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_posts_by_ids

> Vec<models::Post> get_posts_by_ids(request_body)
Get posts by a list of ids

Fetch a list of posts based on the provided postIDs ##### Permissions Must have `read_channel` permission for the channel the post is in or if the channel is public, have the `read_public_channels` permission for the team. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_body** | [**Vec<String>**](String.md) | List of post ids | [required] |

### Return type

[**Vec<models::Post>**](Post.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_posts_for_channel

> models::PostList get_posts_for_channel(channel_id, page, per_page, since, before, after, include_deleted)
Get posts for a channel

Get a page of posts in a channel. Use the query parameters to modify the behaviour of this endpoint. The parameter `since` must not be used with any of `before`, `after`, `page`, and `per_page` parameters. If `since` is used, it will always return all posts modified since that time, ordered by their create time limited till 1000. A caveat with this parameter is that there is no guarantee that the returned posts will be consecutive. It is left to the clients to maintain state and fill any missing holes in the post order. ##### Permissions Must have `read_channel` permission for the channel. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**channel_id** | **String** | The channel ID to get the posts for | [required] |
**page** | Option<**i32**> | The page to select |  |[default to 0]
**per_page** | Option<**i32**> | The number of posts per page |  |[default to 60]
**since** | Option<**i32**> | Provide a non-zero value in Unix time milliseconds to select posts modified after that time |  |
**before** | Option<**String**> | A post id to select the posts that came before this one |  |
**after** | Option<**String**> | A post id to select the posts that came after this one |  |
**include_deleted** | Option<**bool**> | Whether to include deleted posts or not. Must have system admin permissions. |  |[default to false]

### Return type

[**models::PostList**](PostList.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## move_thread

> models::StatusOk move_thread(post_id, move_thread_request)
Move a post (and any posts within that post's thread)

Move a post/thread to another channel. THIS IS A BETA FEATURE. The API is subject to change without notice. ##### Permissions Must have `read_channel` permission for the channel the post is in. Must have `write_post` permission for the channel the post is being moved to.  __Minimum server version__: 9.3 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_id** | **String** | The identifier of the post to move | [required] |
**move_thread_request** | [**MoveThreadRequest**](MoveThreadRequest.md) | The channel identifier of where the post/thread is to be moved | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## patch_post

> models::Post patch_post(post_id, patch_post_request)
Patch a post

Partially update a post by providing only the fields you want to update. Omitted fields will not be updated. The fields that can be updated are defined in the request body, all other provided fields will be ignored. ##### Permissions Must have the `edit_post` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_id** | **String** | Post GUID | [required] |
**patch_post_request** | [**PatchPostRequest**](PatchPostRequest.md) | Post object that is to be updated | [required] |

### Return type

[**models::Post**](Post.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## pin_post

> models::StatusOk pin_post(post_id)
Pin a post to the channel

Pin a post to a channel it is in based from the provided post id string. ##### Permissions Must be authenticated and have the `read_channel` permission to the channel the post is in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_id** | **String** | Post GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## save_acknowledgement_for_post

> models::PostAcknowledgement save_acknowledgement_for_post(user_id, post_id)
Acknowledge a post

Acknowledge a post that has a request for acknowledgements. ##### Permissions Must have `read_channel` permission for the channel the post is in.<br/> Must be logged in as the user or have `edit_other_users` permission.  __Minimum server version__: 7.7 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**post_id** | **String** | Post GUID | [required] |

### Return type

[**models::PostAcknowledgement**](PostAcknowledgement.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_posts

> models::PostListWithSearchMatches search_posts(team_id, search_posts_request)
Search for team posts

Search posts in the team and from the provided terms string. ##### Permissions Must be authenticated and have the `view_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**search_posts_request** | [**SearchPostsRequest**](SearchPostsRequest.md) | The search terms and logic to use in the search. | [required] |

### Return type

[**models::PostListWithSearchMatches**](PostListWithSearchMatches.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_post_reminder

> models::StatusOk set_post_reminder(user_id, post_id, set_post_reminder_request)
Set a post reminder

Set a reminder for the user for the post. ##### Permissions Must have `read_channel` permission for the channel the post is in.  __Minimum server version__: 7.2 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**post_id** | **String** | Post GUID | [required] |
**set_post_reminder_request** | [**SetPostReminderRequest**](SetPostReminderRequest.md) | Target time for the reminder | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_post_unread

> models::ChannelUnreadAt set_post_unread(user_id, post_id)
Mark as unread from a post.

Mark a channel as being unread from a given post. ##### Permissions Must have `read_channel` permission for the channel the post is in or if the channel is public, have the `read_public_channels` permission for the team. Must have `edit_other_users` permission if the user is not the one marking the post for himself.  __Minimum server version__: 5.18 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**post_id** | **String** | Post GUID | [required] |

### Return type

[**models::ChannelUnreadAt**](ChannelUnreadAt.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unpin_post

> models::StatusOk unpin_post(post_id)
Unpin a post to the channel

Unpin a post to a channel it is in based from the provided post id string. ##### Permissions Must be authenticated and have the `read_channel` permission to the channel the post is in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_id** | **String** | Post GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_post

> models::Post update_post(post_id, update_post_request)
Update a post

Update a post. Only the fields listed below are updatable, omitted fields will be treated as blank. ##### Permissions Must have `edit_post` permission for the channel the post is in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_id** | **String** | ID of the post to update | [required] |
**update_post_request** | [**UpdatePostRequest**](UpdatePostRequest.md) | Post object that is to be updated | [required] |

### Return type

[**models::Post**](Post.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

