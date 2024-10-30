# \OauthApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_outgoing_o_auth_connection**](OauthApi.md#create_outgoing_o_auth_connection) | **POST** /api/v4/oauth/outgoing_connections | Create a connection
[**list_outgoing_o_auth_connections**](OauthApi.md#list_outgoing_o_auth_connections) | **GET** /api/v4/oauth/outgoing_connections | List all connections
[**validate_outgoing_o_auth_connection**](OauthApi.md#validate_outgoing_o_auth_connection) | **POST** /api/v4/oauth/outgoing_connections/validate | Validate a connection configuration



## create_outgoing_o_auth_connection

> models::OutgoingOAuthConnectionGetItem create_outgoing_o_auth_connection(team_id, outgoing_o_auth_connection_post_item)
Create a connection

Create an outgoing OAuth connection. __Minimum server version__: 9.6 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Current Team ID in integrations backstage | [required] |
**outgoing_o_auth_connection_post_item** | Option<[**OutgoingOAuthConnectionPostItem**](OutgoingOAuthConnectionPostItem.md)> | Outgoing OAuth connection to create |  |

### Return type

[**models::OutgoingOAuthConnectionGetItem**](OutgoingOAuthConnectionGetItem.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_outgoing_o_auth_connections

> Vec<models::OutgoingOAuthConnectionGetItem> list_outgoing_o_auth_connections(team_id)
List all connections

List all outgoing OAuth connections. __Minimum server version__: 9.6 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Current Team ID in integrations backstage | [required] |

### Return type

[**Vec<models::OutgoingOAuthConnectionGetItem>**](OutgoingOAuthConnectionGetItem.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## validate_outgoing_o_auth_connection

> validate_outgoing_o_auth_connection(team_id, outgoing_o_auth_connection_post_item)
Validate a connection configuration

Validate an outgoing OAuth connection. If an id is provided in the payload, and no client secret is provided, then the stored client secret is implicitly used for the validation. __Minimum server version__: 9.6 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Current Team ID in integrations backstage | [required] |
**outgoing_o_auth_connection_post_item** | Option<[**OutgoingOAuthConnectionPostItem**](OutgoingOAuthConnectionPostItem.md)> | Outgoing OAuth connection to validate |  |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

