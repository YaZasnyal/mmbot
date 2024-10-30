# WebhookOnCreationPayload

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | A unique, 26 characters long, alphanumeric identifier for the playbook run. | [optional]
**name** | Option<**String**> | The name of the playbook run. | [optional]
**description** | Option<**String**> | The description of the playbook run. | [optional]
**is_active** | Option<**bool**> | True if the playbook run is ongoing; false if the playbook run is ended. | [optional]
**owner_user_id** | Option<**String**> | The identifier of the user that is commanding the playbook run. | [optional]
**team_id** | Option<**String**> | The identifier of the team where the playbook run's channel is in. | [optional]
**channel_id** | Option<**String**> | The identifier of the playbook run's channel. | [optional]
**create_at** | Option<**i64**> | The playbook run creation timestamp, formatted as the number of milliseconds since the Unix epoch. | [optional]
**end_at** | Option<**i64**> | The playbook run finish timestamp, formatted as the number of milliseconds since the Unix epoch. It equals 0 if the playbook run is not finished. | [optional]
**delete_at** | Option<**i64**> | The playbook run deletion timestamp, formatted as the number of milliseconds since the Unix epoch. It equals 0 if the playbook run is not deleted. | [optional]
**active_stage** | Option<**i32**> | Zero-based index of the currently active stage. | [optional]
**active_stage_title** | Option<**String**> | The title of the currently active stage. | [optional]
**post_id** | Option<**String**> | If the playbook run was created from a post, this field contains the identifier of such post. If not, this field is empty. | [optional]
**playbook_id** | Option<**String**> | The identifier of the playbook with from which this playbook run was created. | [optional]
**checklists** | Option<[**Vec<models::Checklist>**](Checklist.md)> |  | [optional]
**channel_url** | Option<**String**> | Absolute URL to the playbook run's channel. | [optional]
**details_url** | Option<**String**> | Absolute URL to the playbook run's details. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


