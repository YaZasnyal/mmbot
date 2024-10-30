# CreatePlaybookRunFromDialogRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | Option<**String**> |  | [optional]
**url** | Option<**String**> |  | [optional]
**callback_id** | Option<**String**> | Callback ID provided by the integration. | [optional]
**state** | Option<**String**> | Stringified JSON with the post_id and the client_id. | [optional]
**user_id** | Option<**String**> | ID of the user who submitted the dialog. | [optional]
**channel_id** | Option<**String**> | ID of the channel the user was in when submitting the dialog. | [optional]
**team_id** | Option<**String**> | ID of the team the user was on when submitting the dialog. | [optional]
**submission** | Option<[**models::CreatePlaybookRunFromDialogRequestSubmission**](createPlaybookRunFromDialog_request_submission.md)> |  | [optional]
**cancelled** | Option<**bool**> | If the dialog was cancelled. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


