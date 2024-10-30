# PostMetadata

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**embeds** | Option<[**Vec<models::PostMetadataEmbedsInner>**](PostMetadata_embeds_inner.md)> | Information about content embedded in the post including OpenGraph previews, image link previews, and message attachments. This field will be null if the post does not contain embedded content.  | [optional]
**emojis** | Option<[**Vec<models::Emoji>**](Emoji.md)> | The custom emojis that appear in this point or have been used in reactions to this post. This field will be null if the post does not contain custom emojis.  | [optional]
**files** | Option<[**Vec<models::FileInfo>**](FileInfo.md)> | The FileInfo objects for any files attached to the post. This field will be null if the post does not have any file attachments.  | [optional]
**images** | Option<[**Vec<models::PostMetadataImagesInner>**](PostMetadata_images_inner.md)> | An object mapping the URL of an external image to an object containing the dimensions of that image. This field will be null if the post or its embedded content does not reference any external images.  | [optional]
**reactions** | Option<[**Vec<models::Reaction>**](Reaction.md)> | Any reactions made to this point. This field will be null if no reactions have been made to this post.  | [optional]
**priority** | Option<[**models::PostMetadataPriority**](PostMetadata_priority.md)> |  | [optional]
**acknowledgements** | Option<[**Vec<models::PostAcknowledgement>**](PostAcknowledgement.md)> | Any acknowledgements made to this point.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


