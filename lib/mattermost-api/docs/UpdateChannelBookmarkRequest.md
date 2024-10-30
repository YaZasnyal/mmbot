# UpdateChannelBookmarkRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**file_id** | Option<**String**> | The ID of the file associated with the channel bookmark. Required for bookmarks of type 'file' | [optional]
**display_name** | Option<**String**> | The name of the channel bookmark | [optional]
**sort_order** | Option<**i64**> | The order of the channel bookmark | [optional]
**link_url** | Option<**String**> | The URL associated with the channel bookmark. Required for type bookmarks of type 'link' | [optional]
**image_url** | Option<**String**> | The URL of the image associated with the channel bookmark | [optional]
**emoji** | Option<**String**> | The emoji of the channel bookmark | [optional]
**r#type** | Option<**String**> | * `link` for channel bookmarks that reference a link. `link_url` is requied * `file` for channel bookmarks that reference a file. `file_id` is required  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


