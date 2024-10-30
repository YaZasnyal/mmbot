# CreatePlaybookRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**title** | **String** | The title of the playbook. | 
**description** | Option<**String**> | The description of the playbook. | [optional]
**team_id** | **String** | The identifier of the team where the playbook is in. | 
**create_public_playbook_run** | **bool** | A boolean indicating whether the playbook runs created from this playbook should be public or private. | 
**public** | Option<**bool**> | A boolean indicating whether the playbook is licensed as public or private. Required 'true' for free tier. | [optional]
**checklists** | [**Vec<models::CreatePlaybookRequestChecklistsInner>**](createPlaybook_request_checklists_inner.md) | The stages defined by this playbook. | 
**member_ids** | **Vec<String>** | The identifiers of all the users that are members of this playbook. | 
**broadcast_channel_ids** | Option<**Vec<String>**> | The IDs of the channels where all the status updates will be broadcasted. The team of the broadcast channel must be the same as the playbook's team. | [optional]
**invited_user_ids** | Option<**Vec<String>**> | A list with the IDs of the members to be automatically invited to the playbook run's channel as soon as the playbook run is created. | [optional]
**invite_users_enabled** | Option<**bool**> | Boolean that indicates whether the members declared in invited_user_ids will be automatically invited. | [optional]
**default_owner_id** | Option<**String**> | User ID of the member that will be automatically assigned as owner as soon as the playbook run is created. If the member is not part of the playbook run's channel or is not included in the invited_user_ids list, they will be automatically invited to the channel. | [optional]
**default_owner_enabled** | Option<**String**> | Boolean that indicates whether the member declared in default_owner_id will be automatically assigned as owner. | [optional]
**announcement_channel_id** | Option<**String**> | ID of the channel where the playbook run will be automatically announced as soon as the playbook run is created. | [optional]
**announcement_channel_enabled** | Option<**bool**> | Boolean that indicates whether the playbook run creation will be announced in the channel declared in announcement_channel_id. | [optional]
**webhook_on_creation_url** | Option<**String**> | An absolute URL where a POST request will be sent as soon as the playbook run is created. The allowed protocols are HTTP and HTTPS. | [optional]
**webhook_on_creation_enabled** | Option<**bool**> | Boolean that indicates whether the webhook declared in webhook_on_creation_url will be automatically sent. | [optional]
**webhook_on_status_update_url** | Option<**String**> | An absolute URL where a POST request will be sent as soon as the playbook run's status is updated. The allowed protocols are HTTP and HTTPS. | [optional]
**webhook_on_status_update_enabled** | Option<**bool**> | Boolean that indicates whether the webhook declared in webhook_on_status_update_url will be automatically sent. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


