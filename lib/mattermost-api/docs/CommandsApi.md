# \CommandsApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_command**](CommandsApi.md#create_command) | **POST** /api/v4/commands | Create a command
[**delete_command**](CommandsApi.md#delete_command) | **DELETE** /api/v4/commands/{command_id} | Delete a command
[**execute_command**](CommandsApi.md#execute_command) | **POST** /api/v4/commands/execute | Execute a command
[**get_command_by_id**](CommandsApi.md#get_command_by_id) | **GET** /api/v4/commands/{command_id} | Get a command
[**list_autocomplete_commands**](CommandsApi.md#list_autocomplete_commands) | **GET** /api/v4/teams/{team_id}/commands/autocomplete | List autocomplete commands
[**list_command_autocomplete_suggestions**](CommandsApi.md#list_command_autocomplete_suggestions) | **GET** /api/v4/teams/{team_id}/commands/autocomplete_suggestions | List commands' autocomplete data
[**list_commands**](CommandsApi.md#list_commands) | **GET** /api/v4/commands | List commands for a team
[**move_command**](CommandsApi.md#move_command) | **PUT** /api/v4/commands/{command_id}/move | Move a command
[**regen_command_token**](CommandsApi.md#regen_command_token) | **PUT** /api/v4/commands/{command_id}/regen_token | Generate a new token
[**update_command**](CommandsApi.md#update_command) | **PUT** /api/v4/commands/{command_id} | Update a command



## create_command

> models::Command create_command(create_command_request)
Create a command

Create a command for a team. ##### Permissions `manage_slash_commands` for the team the command is in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_command_request** | [**CreateCommandRequest**](CreateCommandRequest.md) | command to be created | [required] |

### Return type

[**models::Command**](Command.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_command

> models::StatusOk delete_command(command_id)
Delete a command

Delete a command based on command id string. ##### Permissions Must have `manage_slash_commands` permission for the team the command is in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**command_id** | **String** | ID of the command to delete | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## execute_command

> models::CommandResponse execute_command(execute_command_request)
Execute a command

Execute a command on a team. ##### Permissions Must have `use_slash_commands` permission for the team the command is in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**execute_command_request** | [**ExecuteCommandRequest**](ExecuteCommandRequest.md) | command to be executed | [required] |

### Return type

[**models::CommandResponse**](CommandResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_command_by_id

> models::Command get_command_by_id(command_id)
Get a command

Get a command definition based on command id string. ##### Permissions Must have `manage_slash_commands` permission for the team the command is in.  __Minimum server version__: 5.22 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**command_id** | **String** | ID of the command to get | [required] |

### Return type

[**models::Command**](Command.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_autocomplete_commands

> Vec<models::Command> list_autocomplete_commands(team_id)
List autocomplete commands

List autocomplete commands in the team. ##### Permissions `view_team` for the team. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |

### Return type

[**Vec<models::Command>**](Command.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_command_autocomplete_suggestions

> Vec<models::AutocompleteSuggestion> list_command_autocomplete_suggestions(team_id, user_input)
List commands' autocomplete data

List commands' autocomplete data for the team. ##### Permissions `view_team` for the team. __Minimum server version__: 5.24 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**user_input** | **String** | String inputted by the user. | [required] |

### Return type

[**Vec<models::AutocompleteSuggestion>**](AutocompleteSuggestion.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_commands

> Vec<models::Command> list_commands(team_id, custom_only)
List commands for a team

List commands for a team. ##### Permissions `manage_slash_commands` if need list custom commands. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | Option<**String**> | The team id. |  |
**custom_only** | Option<**bool**> | To get only the custom commands. If set to false will get the custom if the user have access plus the system commands, otherwise just the system commands.  |  |[default to false]

### Return type

[**Vec<models::Command>**](Command.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## move_command

> models::StatusOk move_command(command_id, move_command_request)
Move a command

Move a command to a different team based on command id string. ##### Permissions Must have `manage_slash_commands` permission for the team the command is currently in and the destination team.  __Minimum server version__: 5.22 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**command_id** | **String** | ID of the command to move | [required] |
**move_command_request** | [**MoveCommandRequest**](MoveCommandRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## regen_command_token

> models::RegenCommandToken200Response regen_command_token(command_id)
Generate a new token

Generate a new token for the command based on command id string. ##### Permissions Must have `manage_slash_commands` permission for the team the command is in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**command_id** | **String** | ID of the command to generate the new token | [required] |

### Return type

[**models::RegenCommandToken200Response**](RegenCommandToken_200_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_command

> models::Command update_command(command_id, command)
Update a command

Update a single command based on command id string and Command struct. ##### Permissions Must have `manage_slash_commands` permission for the team the command is in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**command_id** | **String** | ID of the command to update | [required] |
**command** | [**Command**](Command.md) |  | [required] |

### Return type

[**models::Command**](Command.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

