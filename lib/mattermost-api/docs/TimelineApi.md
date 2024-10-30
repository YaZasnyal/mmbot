# \TimelineApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**remove_timeline_event**](TimelineApi.md#remove_timeline_event) | **DELETE** /plugins/playbooks/api/v0/runs/{id}/timeline/{event_id}/ | Remove a timeline event from the playbook run



## remove_timeline_event

> remove_timeline_event(id, event_id)
Remove a timeline event from the playbook run

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | ID of the playbook run whose timeline event will be modified. | [required] |
**event_id** | **String** | ID of the timeline event to be deleted | [required] |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

