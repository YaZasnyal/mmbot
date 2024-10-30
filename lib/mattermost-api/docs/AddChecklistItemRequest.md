# AddChecklistItemRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**title** | **String** | The title of the checklist item. | 
**state** | Option<**String**> | The state of the checklist item. An empty string means that the item is not done. | [optional]
**state_modified** | Option<**i64**> | The timestamp for the latest modification of the item's state, formatted as the number of milliseconds since the Unix epoch. It equals 0 if the item was never modified. | [optional]
**assignee_id** | Option<**String**> | The identifier of the user that has been assigned to complete this item. If the item has no assignee, this is an empty string. | [optional]
**assignee_modified** | Option<**i64**> | The timestamp for the latest modification of the item's assignee, formatted as the number of milliseconds since the Unix epoch. It equals 0 if the item never got an assignee. | [optional]
**command** | Option<**String**> | The slash command associated with this item. If the item has no slash command associated, this is an empty string | [optional]
**command_last_run** | Option<**i64**> | The timestamp for the latest execution of the item's command, formatted as the number of milliseconds since the Unix epoch. It equals 0 if the command was never executed. | [optional]
**description** | Option<**String**> | A detailed description of the checklist item, formatted with Markdown. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


