# \WebhooksApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_incoming_webhook**](WebhooksApi.md#create_incoming_webhook) | **POST** /api/v4/hooks/incoming | Create an incoming webhook
[**create_outgoing_webhook**](WebhooksApi.md#create_outgoing_webhook) | **POST** /api/v4/hooks/outgoing | Create an outgoing webhook
[**delete_incoming_webhook**](WebhooksApi.md#delete_incoming_webhook) | **DELETE** /api/v4/hooks/incoming/{hook_id} | Delete an incoming webhook
[**delete_outgoing_webhook**](WebhooksApi.md#delete_outgoing_webhook) | **DELETE** /api/v4/hooks/outgoing/{hook_id} | Delete an outgoing webhook
[**get_incoming_webhook**](WebhooksApi.md#get_incoming_webhook) | **GET** /api/v4/hooks/incoming/{hook_id} | Get an incoming webhook
[**get_incoming_webhooks**](WebhooksApi.md#get_incoming_webhooks) | **GET** /api/v4/hooks/incoming | List incoming webhooks
[**get_outgoing_webhook**](WebhooksApi.md#get_outgoing_webhook) | **GET** /api/v4/hooks/outgoing/{hook_id} | Get an outgoing webhook
[**get_outgoing_webhooks**](WebhooksApi.md#get_outgoing_webhooks) | **GET** /api/v4/hooks/outgoing | List outgoing webhooks
[**regen_outgoing_hook_token**](WebhooksApi.md#regen_outgoing_hook_token) | **POST** /api/v4/hooks/outgoing/{hook_id}/regen_token | Regenerate the token for the outgoing webhook.
[**update_incoming_webhook**](WebhooksApi.md#update_incoming_webhook) | **PUT** /api/v4/hooks/incoming/{hook_id} | Update an incoming webhook
[**update_outgoing_webhook**](WebhooksApi.md#update_outgoing_webhook) | **PUT** /api/v4/hooks/outgoing/{hook_id} | Update an outgoing webhook



## create_incoming_webhook

> models::IncomingWebhook create_incoming_webhook(create_incoming_webhook_request)
Create an incoming webhook

Create an incoming webhook for a channel. ##### Permissions `manage_webhooks` for the team the webhook is in.  `manage_others_incoming_webhooks` for the team the webhook is in if the user is different than the requester. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_incoming_webhook_request** | [**CreateIncomingWebhookRequest**](CreateIncomingWebhookRequest.md) | Incoming webhook to be created | [required] |

### Return type

[**models::IncomingWebhook**](IncomingWebhook.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_outgoing_webhook

> models::OutgoingWebhook create_outgoing_webhook(create_outgoing_webhook_request)
Create an outgoing webhook

Create an outgoing webhook for a team. ##### Permissions `manage_webhooks` for the team the webhook is in.  `manage_others_outgoing_webhooks` for the team the webhook is in if the user is different than the requester. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_outgoing_webhook_request** | [**CreateOutgoingWebhookRequest**](CreateOutgoingWebhookRequest.md) | Outgoing webhook to be created | [required] |

### Return type

[**models::OutgoingWebhook**](OutgoingWebhook.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_incoming_webhook

> models::StatusOk delete_incoming_webhook(hook_id)
Delete an incoming webhook

Delete an incoming webhook given the hook id. ##### Permissions `manage_webhooks` for system or `manage_webhooks` for the specific team or `manage_webhooks` for the channel. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hook_id** | **String** | Incoming webhook GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_outgoing_webhook

> models::StatusOk delete_outgoing_webhook(hook_id)
Delete an outgoing webhook

Delete an outgoing webhook given the hook id. ##### Permissions `manage_webhooks` for system or `manage_webhooks` for the specific team or `manage_webhooks` for the channel. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hook_id** | **String** | Outgoing webhook GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_incoming_webhook

> models::IncomingWebhook get_incoming_webhook(hook_id)
Get an incoming webhook

Get an incoming webhook given the hook id. ##### Permissions `manage_webhooks` for system or `manage_webhooks` for the specific team or `manage_webhooks` for the channel. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hook_id** | **String** | Incoming Webhook GUID | [required] |

### Return type

[**models::IncomingWebhook**](IncomingWebhook.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_incoming_webhooks

> Vec<models::IncomingWebhook> get_incoming_webhooks(page, per_page, team_id, include_total_count)
List incoming webhooks

Get a page of a list of incoming webhooks. Optionally filter for a specific team using query parameters. ##### Permissions `manage_webhooks` for the system or `manage_webhooks` for the specific team. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of hooks per page. |  |[default to 60]
**team_id** | Option<**String**> | The ID of the team to get hooks for. |  |
**include_total_count** | Option<**bool**> | Appends a total count of returned hooks inside the response object - ex: `{ \"incoming_webhooks\": [], \"total_count\": 0 }`. |  |[default to false]

### Return type

[**Vec<models::IncomingWebhook>**](IncomingWebhook.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_outgoing_webhook

> models::OutgoingWebhook get_outgoing_webhook(hook_id)
Get an outgoing webhook

Get an outgoing webhook given the hook id. ##### Permissions `manage_webhooks` for system or `manage_webhooks` for the specific team or `manage_webhooks` for the channel. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hook_id** | **String** | Outgoing webhook GUID | [required] |

### Return type

[**models::OutgoingWebhook**](OutgoingWebhook.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_outgoing_webhooks

> Vec<models::OutgoingWebhook> get_outgoing_webhooks(page, per_page, team_id, channel_id)
List outgoing webhooks

Get a page of a list of outgoing webhooks. Optionally filter for a specific team or channel using query parameters. ##### Permissions `manage_webhooks` for the system or `manage_webhooks` for the specific team/channel. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of hooks per page. |  |[default to 60]
**team_id** | Option<**String**> | The ID of the team to get hooks for. |  |
**channel_id** | Option<**String**> | The ID of the channel to get hooks for. |  |

### Return type

[**Vec<models::OutgoingWebhook>**](OutgoingWebhook.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## regen_outgoing_hook_token

> models::StatusOk regen_outgoing_hook_token(hook_id)
Regenerate the token for the outgoing webhook.

Regenerate the token for the outgoing webhook. ##### Permissions `manage_webhooks` for system or `manage_webhooks` for the specific team or `manage_webhooks` for the channel. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hook_id** | **String** | Outgoing webhook GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_incoming_webhook

> models::IncomingWebhook update_incoming_webhook(hook_id, update_incoming_webhook_request)
Update an incoming webhook

Update an incoming webhook given the hook id. ##### Permissions `manage_webhooks` for system or `manage_webhooks` for the specific team or `manage_webhooks` for the channel. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hook_id** | **String** | Incoming Webhook GUID | [required] |
**update_incoming_webhook_request** | [**UpdateIncomingWebhookRequest**](UpdateIncomingWebhookRequest.md) | Incoming webhook to be updated | [required] |

### Return type

[**models::IncomingWebhook**](IncomingWebhook.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_outgoing_webhook

> models::OutgoingWebhook update_outgoing_webhook(hook_id, update_outgoing_webhook_request)
Update an outgoing webhook

Update an outgoing webhook given the hook id. ##### Permissions `manage_webhooks` for system or `manage_webhooks` for the specific team or `manage_webhooks` for the channel. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**hook_id** | **String** | outgoing Webhook GUID | [required] |
**update_outgoing_webhook_request** | [**UpdateOutgoingWebhookRequest**](UpdateOutgoingWebhookRequest.md) | Outgoing webhook to be updated | [required] |

### Return type

[**models::OutgoingWebhook**](OutgoingWebhook.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

