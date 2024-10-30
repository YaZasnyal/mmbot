# \SharedChannelsApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_all_shared_channels**](SharedChannelsApi.md#get_all_shared_channels) | **GET** /api/v4/sharedchannels/{team_id} | Get all shared channels for team.
[**get_remote_cluster_info**](SharedChannelsApi.md#get_remote_cluster_info) | **GET** /api/v4/sharedchannels/remote_info/{remote_id} | Get remote cluster info by ID for user.
[**get_shared_channel_remotes_by_remote_cluster**](SharedChannelsApi.md#get_shared_channel_remotes_by_remote_cluster) | **GET** /api/v4/remotecluster/{remote_id}/sharedchannelremotes | Get shared channel remotes by remote cluster.
[**invite_remote_cluster_to_channel**](SharedChannelsApi.md#invite_remote_cluster_to_channel) | **POST** /api/v4/remotecluster/{remote_id}/channels/{channel_id}/invite | Invites a remote cluster to a channel.
[**uninvite_remote_cluster_to_channel**](SharedChannelsApi.md#uninvite_remote_cluster_to_channel) | **POST** /api/v4/remotecluster/{remote_id}/channels/{channel_id}/uninvite | Uninvites a remote cluster to a channel.



## get_all_shared_channels

> Vec<models::SharedChannel> get_all_shared_channels(team_id, page, per_page)
Get all shared channels for team.

Get all shared channels for a team.  __Minimum server version__: 5.50  ##### Permissions Must be authenticated. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team Id | [required] |
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of sharedchannels per page. |  |[default to 0]

### Return type

[**Vec<models::SharedChannel>**](SharedChannel.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_remote_cluster_info

> models::RemoteClusterInfo get_remote_cluster_info(remote_id)
Get remote cluster info by ID for user.

Get remote cluster info based on remoteId.  __Minimum server version__: 5.50  ##### Permissions Must be authenticated and user must belong to at least one channel shared with the remote cluster. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**remote_id** | **String** | Remote Cluster GUID | [required] |

### Return type

[**models::RemoteClusterInfo**](RemoteClusterInfo.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_shared_channel_remotes_by_remote_cluster

> Vec<models::SharedChannelRemote> get_shared_channel_remotes_by_remote_cluster(remote_id, include_unconfirmed, exclude_confirmed, exclude_home, exclude_remote, include_deleted, page, per_page)
Get shared channel remotes by remote cluster.

Get a list of the channels shared with a given remote cluster and their status.  ##### Permissions `manage_secure_connections` 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**remote_id** | **String** | The remote cluster GUID | [required] |
**include_unconfirmed** | Option<**bool**> | Include those Shared channel remotes that are unconfirmed |  |
**exclude_confirmed** | Option<**bool**> | Show only those Shared channel remotes that are not confirmed yet |  |
**exclude_home** | Option<**bool**> | Show only those Shared channel remotes that were shared with this server |  |
**exclude_remote** | Option<**bool**> | Show only those Shared channel remotes that were shared from this server |  |
**include_deleted** | Option<**bool**> | Include those Shared channel remotes that have been deleted |  |
**page** | Option<**i32**> | The page to select |  |
**per_page** | Option<**i32**> | The number of shared channels per page |  |

### Return type

[**Vec<models::SharedChannelRemote>**](SharedChannelRemote.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## invite_remote_cluster_to_channel

> models::StatusOk invite_remote_cluster_to_channel(remote_id, channel_id)
Invites a remote cluster to a channel.

Invites a remote cluster to a channel, sharing the channel if needed. If the remote cluster was already invited to the channel, calling this endpoint will have no effect.  ##### Permissions `manage_shared_channels` 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**remote_id** | **String** | The remote cluster GUID | [required] |
**channel_id** | **String** | The channel GUID to invite the remote cluster to | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## uninvite_remote_cluster_to_channel

> models::StatusOk uninvite_remote_cluster_to_channel(remote_id, channel_id)
Uninvites a remote cluster to a channel.

Stops sharing a channel with a remote cluster. If the channel was not shared with the remote, calling this endpoint will have no effect.  ##### Permissions `manage_shared_channels` 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**remote_id** | **String** | The remote cluster GUID | [required] |
**channel_id** | **String** | The channel GUID to uninvite the remote cluster to | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

