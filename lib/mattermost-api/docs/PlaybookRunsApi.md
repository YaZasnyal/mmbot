# \PlaybookRunsApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_checklist_item**](PlaybookRunsApi.md#add_checklist_item) | **POST** /plugins/playbooks/api/v0/runs/{id}/checklists/{checklist}/add | Add an item to a playbook run's checklist
[**change_owner**](PlaybookRunsApi.md#change_owner) | **POST** /plugins/playbooks/api/v0/runs/{id}/owner | Update playbook run owner
[**create_playbook_run_from_post**](PlaybookRunsApi.md#create_playbook_run_from_post) | **POST** /plugins/playbooks/api/v0/runs | Create a new playbook run
[**end_playbook_run**](PlaybookRunsApi.md#end_playbook_run) | **PUT** /plugins/playbooks/api/v0/runs/{id}/end | End a playbook run
[**finish**](PlaybookRunsApi.md#finish) | **PUT** /plugins/playbooks/api/v0/runs/{id}/finish | Finish a playbook
[**get_channels**](PlaybookRunsApi.md#get_channels) | **GET** /plugins/playbooks/api/v0/runs/channels | Get playbook run channels
[**get_owners**](PlaybookRunsApi.md#get_owners) | **GET** /plugins/playbooks/api/v0/runs/owners | Get all owners
[**get_playbook_run**](PlaybookRunsApi.md#get_playbook_run) | **GET** /plugins/playbooks/api/v0/runs/{id} | Get a playbook run
[**get_playbook_run_by_channel_id**](PlaybookRunsApi.md#get_playbook_run_by_channel_id) | **GET** /plugins/playbooks/api/v0/runs/channel/{channel_id} | Find playbook run by channel ID
[**get_playbook_run_metadata**](PlaybookRunsApi.md#get_playbook_run_metadata) | **GET** /plugins/playbooks/api/v0/runs/{id}/metadata | Get playbook run metadata
[**item_delete**](PlaybookRunsApi.md#item_delete) | **DELETE** /plugins/playbooks/api/v0/runs/{id}/checklists/{checklist}/item/{item} | Delete an item of a playbook run's checklist
[**item_rename**](PlaybookRunsApi.md#item_rename) | **PUT** /plugins/playbooks/api/v0/runs/{id}/checklists/{checklist}/item/{item} | Update an item of a playbook run's checklist
[**item_run**](PlaybookRunsApi.md#item_run) | **PUT** /plugins/playbooks/api/v0/runs/{id}/checklists/{checklist}/item/{item}/run | Run an item's slash command
[**item_set_assignee**](PlaybookRunsApi.md#item_set_assignee) | **PUT** /plugins/playbooks/api/v0/runs/{id}/checklists/{checklist}/item/{item}/assignee | Update the assignee of an item
[**item_set_state**](PlaybookRunsApi.md#item_set_state) | **PUT** /plugins/playbooks/api/v0/runs/{id}/checklists/{checklist}/item/{item}/state | Update the state of an item
[**list_playbook_runs**](PlaybookRunsApi.md#list_playbook_runs) | **GET** /plugins/playbooks/api/v0/runs | List all playbook runs
[**reoder_checklist_item**](PlaybookRunsApi.md#reoder_checklist_item) | **PUT** /plugins/playbooks/api/v0/runs/{id}/checklists/{checklist}/reorder | Reorder an item in a playbook run's checklist
[**restart_playbook_run**](PlaybookRunsApi.md#restart_playbook_run) | **PUT** /plugins/playbooks/api/v0/runs/{id}/restart | Restart a playbook run
[**status**](PlaybookRunsApi.md#status) | **POST** /plugins/playbooks/api/v0/runs/{id}/status | Update a playbook run's status
[**update_playbook_run**](PlaybookRunsApi.md#update_playbook_run) | **PATCH** /plugins/playbooks/api/v0/runs/{id} | Update a playbook run



## add_checklist_item

> add_checklist_item(id, checklist, add_checklist_item_request)
Add an item to a playbook run's checklist

The most common pattern to add a new item is to only send its title as the request payload. By default, it is an open item, with no assignee and no slash command.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run whose checklist will be modified. | [required] |
**checklist** | **i32** | Zero-based index of the checklist to modify. | [required] |
**add_checklist_item_request** | Option<[**AddChecklistItemRequest**](AddChecklistItemRequest.md)> | Checklist item payload. |  |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## change_owner

> change_owner(id, change_owner_request)
Update playbook run owner

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run whose owner will be changed. | [required] |
**change_owner_request** | Option<[**ChangeOwnerRequest**](ChangeOwnerRequest.md)> | Payload to change the playbook run's owner. |  |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_playbook_run_from_post

> models::PlaybookRun create_playbook_run_from_post(create_playbook_run_from_post_request)
Create a new playbook run

Create a new playbook run in a team, using a playbook as template, with a specific name and a specific owner.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_playbook_run_from_post_request** | Option<[**CreatePlaybookRunFromPostRequest**](CreatePlaybookRunFromPostRequest.md)> | Playbook run payload. |  |

### Return type

[**models::PlaybookRun**](PlaybookRun.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## end_playbook_run

> end_playbook_run(id)
End a playbook run

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run to end. | [required] |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## finish

> finish(id)
Finish a playbook

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run to finish. | [required] |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_channels

> Vec<String> get_channels(team_id, sort, direction, status, owner_user_id, search_term, participant_id)
Get playbook run channels

Get all channels associated with a playbook run, filtered by team, status, owner, name and/or members, and sorted by ID, name, status, creation date, end date, team, or owner ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | ID of the team to filter by. | [required] |
**sort** | Option<**String**> | Field to sort the returned channels by, according to their playbook run. |  |[default to create_at]
**direction** | Option<**String**> | Direction (ascending or descending) followed by the sorting of the playbook runs associated to the channels. |  |[default to desc]
**status** | Option<**String**> | The returned list will contain only the channels whose playbook run has this status. |  |[default to all]
**owner_user_id** | Option<**String**> | The returned list will contain only the channels whose playbook run is commanded by this user. |  |
**search_term** | Option<**String**> | The returned list will contain only the channels associated to a playbook run whose name contains the search term. |  |
**participant_id** | Option<**String**> | The returned list will contain only the channels associated to a playbook run for which the given user is a participant. |  |

### Return type

**Vec<String>**

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_owners

> Vec<models::OwnerInfo> get_owners(team_id)
Get all owners

Get the owners of all playbook runs, filtered by team.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | ID of the team to filter by. | [required] |

### Return type

[**Vec<models::OwnerInfo>**](OwnerInfo.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_playbook_run

> models::PlaybookRun get_playbook_run(id)
Get a playbook run

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run to retrieve. | [required] |

### Return type

[**models::PlaybookRun**](PlaybookRun.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_playbook_run_by_channel_id

> models::PlaybookRun get_playbook_run_by_channel_id(channel_id)
Find playbook run by channel ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**channel_id** | **String** | ID of the channel associated to the playbook run to retrieve. | [required] |

### Return type

[**models::PlaybookRun**](PlaybookRun.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_playbook_run_metadata

> models::PlaybookRunMetadata get_playbook_run_metadata(id)
Get playbook run metadata

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run whose metadata will be retrieved. | [required] |

### Return type

[**models::PlaybookRunMetadata**](PlaybookRunMetadata.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## item_delete

> item_delete(id, checklist, item)
Delete an item of a playbook run's checklist

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run whose checklist will be modified. | [required] |
**checklist** | **i32** | Zero-based index of the checklist to modify. | [required] |
**item** | **i32** | Zero-based index of the item to modify. | [required] |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## item_rename

> item_rename(id, checklist, item, item_rename_request)
Update an item of a playbook run's checklist

Update the title and the slash command of an item in one of the playbook run's checklists.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run whose checklist will be modified. | [required] |
**checklist** | **i32** | Zero-based index of the checklist to modify. | [required] |
**item** | **i32** | Zero-based index of the item to modify. | [required] |
**item_rename_request** | Option<[**ItemRenameRequest**](ItemRenameRequest.md)> | Update checklist item payload. |  |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## item_run

> models::TriggerIdReturn item_run(id, checklist, item)
Run an item's slash command

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run whose item will be executed. | [required] |
**checklist** | **i32** | Zero-based index of the checklist whose item will be executed. | [required] |
**item** | **i32** | Zero-based index of the item whose slash command will be executed. | [required] |

### Return type

[**models::TriggerIdReturn**](TriggerIdReturn.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## item_set_assignee

> item_set_assignee(id, checklist, item, item_set_assignee_request)
Update the assignee of an item

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run whose item will get a new assignee. | [required] |
**checklist** | **i32** | Zero-based index of the checklist whose item will get a new assignee. | [required] |
**item** | **i32** | Zero-based index of the item that will get a new assignee. | [required] |
**item_set_assignee_request** | Option<[**ItemSetAssigneeRequest**](ItemSetAssigneeRequest.md)> | User ID of the new assignee. |  |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## item_set_state

> item_set_state(id, checklist, item, item_set_state_request)
Update the state of an item

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run whose checklist will be modified. | [required] |
**checklist** | **i32** | Zero-based index of the checklist to modify. | [required] |
**item** | **i32** | Zero-based index of the item to modify. | [required] |
**item_set_state_request** | Option<[**ItemSetStateRequest**](ItemSetStateRequest.md)> | Update checklist item's state payload. |  |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_playbook_runs

> models::PlaybookRunList list_playbook_runs(team_id, page, per_page, sort, direction, statuses, owner_user_id, participant_id, search_term)
List all playbook runs

Retrieve a paged list of playbook runs, filtered by team, status, owner, name and/or members, and sorted by ID, name, status, creation date, end date, team or owner ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | ID of the team to filter by. | [required] |
**page** | Option<**i32**> | Zero-based index of the page to request. |  |[default to 0]
**per_page** | Option<**i32**> | Number of playbook runs to return per page. |  |[default to 1000]
**sort** | Option<**String**> | Field to sort the returned playbook runs by. |  |[default to create_at]
**direction** | Option<**String**> | Direction (ascending or descending) followed by the sorting of the playbook runs. |  |[default to desc]
**statuses** | Option<[**Vec<String>**](String.md)> | The returned list will contain only the playbook runs with the specified statuses. |  |[default to ["InProgress"]]
**owner_user_id** | Option<**String**> | The returned list will contain only the playbook runs commanded by this user. Specify \"me\" for current user. |  |
**participant_id** | Option<**String**> | The returned list will contain only the playbook runs for which the given user is a participant. Specify \"me\" for current user. |  |
**search_term** | Option<**String**> | The returned list will contain only the playbook runs whose name contains the search term. |  |

### Return type

[**models::PlaybookRunList**](PlaybookRunList.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reoder_checklist_item

> reoder_checklist_item(id, checklist, reoder_checklist_item_request)
Reorder an item in a playbook run's checklist

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run whose checklist will be modified. | [required] |
**checklist** | **i32** | Zero-based index of the checklist to modify. | [required] |
**reoder_checklist_item_request** | Option<[**ReoderChecklistItemRequest**](ReoderChecklistItemRequest.md)> | Reorder checklist item payload. |  |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## restart_playbook_run

> restart_playbook_run(id)
Restart a playbook run

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run to restart. | [required] |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## status

> status(id, status_request)
Update a playbook run's status

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run to update. | [required] |
**status_request** | Option<[**StatusRequest**](StatusRequest.md)> | Payload to change the playbook run's status update message. |  |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_playbook_run

> update_playbook_run(id, update_playbook_run_request)
Update a playbook run

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run to retrieve. | [required] |
**update_playbook_run_request** | Option<[**UpdatePlaybookRunRequest**](UpdatePlaybookRunRequest.md)> | Playbook run update payload. |  |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

