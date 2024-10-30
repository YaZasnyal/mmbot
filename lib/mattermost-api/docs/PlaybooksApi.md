# \PlaybooksApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_playbook**](PlaybooksApi.md#create_playbook) | **POST** /plugins/playbooks/api/v0/playbooks | Create a playbook
[**delete_playbook**](PlaybooksApi.md#delete_playbook) | **DELETE** /plugins/playbooks/api/v0/playbooks/{id} | Delete a playbook
[**get_playbook**](PlaybooksApi.md#get_playbook) | **GET** /plugins/playbooks/api/v0/playbooks/{id} | Get a playbook
[**get_playbooks**](PlaybooksApi.md#get_playbooks) | **GET** /plugins/playbooks/api/v0/playbooks | List all playbooks
[**update_playbook**](PlaybooksApi.md#update_playbook) | **PUT** /plugins/playbooks/api/v0/playbooks/{id} | Update a playbook



## create_playbook

> models::CreatePlaybook201Response create_playbook(create_playbook_request)
Create a playbook

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_playbook_request** | Option<[**CreatePlaybookRequest**](CreatePlaybookRequest.md)> | Playbook |  |

### Return type

[**models::CreatePlaybook201Response**](createPlaybook_201_response.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_playbook

> delete_playbook(id)
Delete a playbook

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_playbook

> models::Playbook get_playbook(id)
Get a playbook

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook to retrieve. | [required] |

### Return type

[**models::Playbook**](Playbook.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_playbooks

> models::PlaybookList get_playbooks(team_id, page, per_page, sort, direction, with_archived)
List all playbooks

Retrieve a paged list of playbooks, filtered by team, and sorted by title, number of stages or number of steps.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | ID of the team to filter by. | [required] |
**page** | Option<**i32**> | Zero-based index of the page to request. |  |[default to 0]
**per_page** | Option<**i32**> | Number of playbooks to return per page. |  |[default to 1000]
**sort** | Option<**String**> | Field to sort the returned playbooks by title, number of stages or total number of steps. |  |[default to title]
**direction** | Option<**String**> | Direction (ascending or descending) followed by the sorting of the playbooks. |  |[default to asc]
**with_archived** | Option<**bool**> | Includes archived playbooks in the result. |  |[default to false]

### Return type

[**models::PlaybookList**](PlaybookList.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_playbook

> update_playbook(id, playbook)
Update a playbook

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook to update. | [required] |
**playbook** | Option<[**Playbook**](Playbook.md)> | Playbook payload |  |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

