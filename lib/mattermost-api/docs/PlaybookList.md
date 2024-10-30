# PlaybookList

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**total_count** | Option<**i32**> | The total number of playbooks in the list, regardless of the paging. | [optional]
**page_count** | Option<**i32**> | The total number of pages. This depends on the total number of playbooks in the database and the per_page parameter sent with the request. | [optional]
**has_more** | Option<**bool**> | A boolean describing whether there are more pages after the currently returned. | [optional]
**items** | Option<[**Vec<models::Playbook>**](Playbook.md)> | The playbooks in this page. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


