# \RemoteClustersApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**accept_remote_cluster_invite**](RemoteClustersApi.md#accept_remote_cluster_invite) | **POST** /api/v4/remotecluster/accept_invite | Accept a remote cluster invite code.
[**create_remote_cluster**](RemoteClustersApi.md#create_remote_cluster) | **POST** /api/v4/remotecluster | Create a new remote cluster.
[**delete_remote_cluster**](RemoteClustersApi.md#delete_remote_cluster) | **DELETE** /api/v4/remotecluster/{remote_id} | Delete a remote cluster.
[**get_remote_cluster**](RemoteClustersApi.md#get_remote_cluster) | **GET** /api/v4/remotecluster/{remote_id} | Get a remote cluster.
[**get_remote_clusters**](RemoteClustersApi.md#get_remote_clusters) | **GET** /api/v4/remotecluster | Get a list of remote clusters.
[**patch_remote_cluster**](RemoteClustersApi.md#patch_remote_cluster) | **PATCH** /api/v4/remotecluster/{remote_id} | Patch a remote cluster.



## accept_remote_cluster_invite

> models::RemoteCluster accept_remote_cluster_invite(accept_remote_cluster_invite_request)
Accept a remote cluster invite code.

Accepts a remote cluster invite code.  ##### Permissions `manage_secure_connections` 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**accept_remote_cluster_invite_request** | Option<[**AcceptRemoteClusterInviteRequest**](AcceptRemoteClusterInviteRequest.md)> |  |  |

### Return type

[**models::RemoteCluster**](RemoteCluster.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_remote_cluster

> models::CreateRemoteCluster201Response create_remote_cluster(create_remote_cluster_request)
Create a new remote cluster.

Create a new remote cluster and generate an invite code.  ##### Permissions `manage_secure_connections` 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_remote_cluster_request** | Option<[**CreateRemoteClusterRequest**](CreateRemoteClusterRequest.md)> |  |  |

### Return type

[**models::CreateRemoteCluster201Response**](CreateRemoteCluster_201_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_remote_cluster

> delete_remote_cluster(remote_id)
Delete a remote cluster.

Deletes a Remote Cluster.  ##### Permissions `manage_secure_connections` 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**remote_id** | **String** | Remote Cluster GUID | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_remote_cluster

> models::RemoteCluster get_remote_cluster(remote_id)
Get a remote cluster.

Get the Remote Cluster details from the provided id string.  ##### Permissions `manage_secure_connections` 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**remote_id** | **String** | Remote Cluster GUID | [required] |

### Return type

[**models::RemoteCluster**](RemoteCluster.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_remote_clusters

> Vec<models::RemoteCluster> get_remote_clusters(page, per_page, exclude_offline, in_channel, not_in_channel, only_confirmed, only_plugins, exclude_plugins, include_deleted)
Get a list of remote clusters.

Get a list of remote clusters.  ##### Permissions `manage_secure_connections` 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | The page to select |  |
**per_page** | Option<**i32**> | The number of remote clusters per page |  |
**exclude_offline** | Option<**bool**> | Exclude offline remote clusters |  |
**in_channel** | Option<**String**> | Select remote clusters in channel |  |
**not_in_channel** | Option<**String**> | Select remote clusters not in this channel |  |
**only_confirmed** | Option<**bool**> | Select only remote clusters already confirmed |  |
**only_plugins** | Option<**bool**> | Select only remote clusters that belong to a plugin |  |
**exclude_plugins** | Option<**bool**> | Select only remote clusters that don't belong to a plugin |  |
**include_deleted** | Option<**bool**> | Include those remote clusters that have been deleted |  |

### Return type

[**Vec<models::RemoteCluster>**](RemoteCluster.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## patch_remote_cluster

> models::RemoteCluster patch_remote_cluster(remote_id, patch_remote_cluster_request)
Patch a remote cluster.

Partially update a Remote Cluster by providing only the fields you want to update. Ommited fields will not be updated.  ##### Permissions `manage_secure_connections` 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**remote_id** | **String** | Remote Cluster GUID | [required] |
**patch_remote_cluster_request** | Option<[**PatchRemoteClusterRequest**](PatchRemoteClusterRequest.md)> |  |  |

### Return type

[**models::RemoteCluster**](RemoteCluster.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

