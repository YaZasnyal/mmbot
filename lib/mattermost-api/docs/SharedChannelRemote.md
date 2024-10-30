# SharedChannelRemote

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | The id of the shared channel remote | [optional]
**channel_id** | Option<**String**> | The id of the channel | [optional]
**creator_id** | Option<**String**> | Id of the user that invited the remote to share the channel | [optional]
**create_at** | Option<**i32**> | Time in milliseconds that the remote was invited to the channel | [optional]
**update_at** | Option<**i32**> | Time in milliseconds that the shared channel remote record was last updated | [optional]
**delete_at** | Option<**i32**> | Time in milliseconds that the shared chanenl remote record was deleted | [optional]
**is_invite_accepted** | Option<**bool**> | Indicates if the invite has been accepted by the remote | [optional]
**is_invite_confirmed** | Option<**bool**> | Indicates if the invite has been confirmed by the remote | [optional]
**remote_id** | Option<**String**> | Id of the remote cluster that the channel is shared with | [optional]
**last_post_update_at** | Option<**i32**> | Time in milliseconds of the last post in the channel that was synchronized with the remote update_at | [optional]
**last_post_id** | Option<**String**> | Id of the last post in the channel that was synchronized with the remote | [optional]
**last_post_create_at** | Option<**String**> | Time in milliseconds of the last post in the channel that was synchronized with the remote create_at | [optional]
**last_post_create_id** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


