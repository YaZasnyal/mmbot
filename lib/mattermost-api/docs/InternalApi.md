# \InternalApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_playbook_run_from_dialog**](InternalApi.md#create_playbook_run_from_dialog) | **POST** /plugins/playbooks/api/v0/runs/dialog | Create a new playbook run from dialog
[**end_playbook_run_dialog**](InternalApi.md#end_playbook_run_dialog) | **POST** /plugins/playbooks/api/v0/runs/{id}/end | End a playbook run from dialog
[**get_checklist_autocomplete**](InternalApi.md#get_checklist_autocomplete) | **GET** /plugins/playbooks/api/v0/runs/checklist-autocomplete | Get autocomplete data for /playbook check
[**next_stage_dialog**](InternalApi.md#next_stage_dialog) | **POST** /plugins/playbooks/api/v0/runs/{id}/next-stage-dialog | Go to next stage from dialog



## create_playbook_run_from_dialog

> models::PlaybookRun create_playbook_run_from_dialog(create_playbook_run_from_dialog_request)
Create a new playbook run from dialog

This is an internal endpoint to create a playbook run from the submission of an interactive dialog, filled by a user in the webapp. See [Interactive Dialogs](https://docs.mattermost.com/developer/interactive-dialogs.html) for more information.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_playbook_run_from_dialog_request** | Option<[**CreatePlaybookRunFromDialogRequest**](CreatePlaybookRunFromDialogRequest.md)> | Dialog submission payload. |  |

### Return type

[**models::PlaybookRun**](PlaybookRun.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## end_playbook_run_dialog

> end_playbook_run_dialog(id)
End a playbook run from dialog

This is an internal endpoint to end a playbook run via a confirmation dialog, submitted by a user in the webapp.

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


## get_checklist_autocomplete

> Vec<models::GetChecklistAutocomplete200ResponseInner> get_checklist_autocomplete(channel_id)
Get autocomplete data for /playbook check

This is an internal endpoint used by the autocomplete system to retrieve the data needed to show the list of items that the user can check.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**channel_id** | **String** | ID of the channel the user is in. | [required] |

### Return type

[**Vec<models::GetChecklistAutocomplete200ResponseInner>**](getChecklistAutocomplete_200_response_inner.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## next_stage_dialog

> next_stage_dialog(id, next_stage_dialog_request)
Go to next stage from dialog

This is an internal endpoint to go to the next stage via a confirmation dialog, submitted by a user in the webapp.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The PlaybookRun ID | [required] |
**next_stage_dialog_request** | Option<[**NextStageDialogRequest**](NextStageDialogRequest.md)> | Dialog submission payload. |  |

### Return type

 (empty response body)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

