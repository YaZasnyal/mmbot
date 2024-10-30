# Playbook

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | A unique, 26 characters long, alphanumeric identifier for the playbook. | [optional]
**title** | Option<**String**> | The title of the playbook. | [optional]
**description** | Option<**String**> | The description of the playbook. | [optional]
**team_id** | Option<**String**> | The identifier of the team where the playbook is in. | [optional]
**create_public_playbook_run** | Option<**bool**> | A boolean indicating whether the playbook runs created from this playbook should be public or private. | [optional]
**create_at** | Option<**i64**> | The playbook creation timestamp, formatted as the number of milliseconds since the Unix epoch. | [optional]
**delete_at** | Option<**i64**> | The playbook deletion timestamp, formatted as the number of milliseconds since the Unix epoch. It equals 0 if the playbook is not deleted. | [optional]
**num_stages** | Option<**i64**> | The number of stages defined in this playbook. | [optional]
**num_steps** | Option<**i64**> | The total number of steps from all the stages defined in this playbook. | [optional]
**checklists** | Option<[**Vec<models::Checklist>**](Checklist.md)> | The stages defined in this playbook. | [optional]
**member_ids** | Option<**Vec<String>**> | The identifiers of all the users that are members of this playbook. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


