# \UsersApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**attach_device_extra_props**](UsersApi.md#attach_device_extra_props) | **PUT** /api/v4/users/sessions/device | Attach mobile device and extra props to the session object
[**autocomplete_users**](UsersApi.md#autocomplete_users) | **GET** /api/v4/users/autocomplete | Autocomplete users
[**check_user_mfa**](UsersApi.md#check_user_mfa) | **POST** /api/v4/users/mfa | Check MFA
[**convert_bot_to_user_0**](UsersApi.md#convert_bot_to_user_0) | **POST** /api/v4/bots/{bot_user_id}/convert_to_user | Convert a bot into a user
[**convert_user_to_bot_0**](UsersApi.md#convert_user_to_bot_0) | **POST** /api/v4/users/{user_id}/convert_to_bot | Convert a user into a bot
[**create_user**](UsersApi.md#create_user) | **POST** /api/v4/users | Create a user
[**create_user_access_token**](UsersApi.md#create_user_access_token) | **POST** /api/v4/users/{user_id}/tokens | Create a user access token
[**delete_user**](UsersApi.md#delete_user) | **DELETE** /api/v4/users/{user_id} | Deactivate a user account.
[**demote_user_to_guest**](UsersApi.md#demote_user_to_guest) | **POST** /api/v4/users/{user_id}/demote | Demote a user to a guest
[**disable_user_access_token**](UsersApi.md#disable_user_access_token) | **POST** /api/v4/users/tokens/disable | Disable personal access token
[**enable_user_access_token**](UsersApi.md#enable_user_access_token) | **POST** /api/v4/users/tokens/enable | Enable personal access token
[**generate_mfa_secret**](UsersApi.md#generate_mfa_secret) | **POST** /api/v4/users/{user_id}/mfa/generate | Generate MFA secret
[**get_channel_members_with_team_data_for_user**](UsersApi.md#get_channel_members_with_team_data_for_user) | **GET** /api/v4/users/{user_id}/channel_members | Get all channel members from all teams for a user
[**get_default_profile_image**](UsersApi.md#get_default_profile_image) | **GET** /api/v4/users/{user_id}/image/default | Return user's default (generated) profile image
[**get_known_users**](UsersApi.md#get_known_users) | **GET** /api/v4/users/known | Get user IDs of known users
[**get_profile_image**](UsersApi.md#get_profile_image) | **GET** /api/v4/users/{user_id}/image | Get user's profile image
[**get_server_limits**](UsersApi.md#get_server_limits) | **GET** /api/v4/limits/server | Gets the server limits for the server
[**get_sessions**](UsersApi.md#get_sessions) | **GET** /api/v4/users/{user_id}/sessions | Get user's sessions
[**get_total_users_stats**](UsersApi.md#get_total_users_stats) | **GET** /api/v4/users/stats | Get total count of users in the system
[**get_total_users_stats_filtered**](UsersApi.md#get_total_users_stats_filtered) | **GET** /api/v4/users/stats/filtered | Get total count of users in the system matching the specified filters
[**get_uploads_for_user**](UsersApi.md#get_uploads_for_user) | **GET** /api/v4/users/{user_id}/uploads | Get uploads for a user
[**get_user**](UsersApi.md#get_user) | **GET** /api/v4/users/{user_id} | Get a user
[**get_user_access_token**](UsersApi.md#get_user_access_token) | **GET** /api/v4/users/tokens/{token_id} | Get a user access token
[**get_user_access_tokens**](UsersApi.md#get_user_access_tokens) | **GET** /api/v4/users/tokens | Get user access tokens
[**get_user_access_tokens_for_user**](UsersApi.md#get_user_access_tokens_for_user) | **GET** /api/v4/users/{user_id}/tokens | Get user access tokens
[**get_user_audits**](UsersApi.md#get_user_audits) | **GET** /api/v4/users/{user_id}/audits | Get user's audits
[**get_user_by_email**](UsersApi.md#get_user_by_email) | **GET** /api/v4/users/email/{email} | Get a user by email
[**get_user_by_username**](UsersApi.md#get_user_by_username) | **GET** /api/v4/users/username/{username} | Get a user by username
[**get_user_count_for_reporting**](UsersApi.md#get_user_count_for_reporting) | **GET** /api/v4/reports/users/count | Gets the full count of users that match the filter.
[**get_user_terms_of_service**](UsersApi.md#get_user_terms_of_service) | **GET** /api/v4/users/{user_id}/terms_of_service | Fetches user's latest terms of service action if the latest action was for acceptance.
[**get_users**](UsersApi.md#get_users) | **GET** /api/v4/users | Get users
[**get_users_by_group_channel_ids**](UsersApi.md#get_users_by_group_channel_ids) | **POST** /api/v4/users/group_channels | Get users by group channels ids
[**get_users_by_ids**](UsersApi.md#get_users_by_ids) | **POST** /api/v4/users/ids | Get users by ids
[**get_users_by_usernames**](UsersApi.md#get_users_by_usernames) | **POST** /api/v4/users/usernames | Get users by usernames
[**get_users_for_reporting**](UsersApi.md#get_users_for_reporting) | **GET** /api/v4/reports/users | Get a list of paged and sorted users for admin reporting purposes
[**get_users_with_invalid_emails**](UsersApi.md#get_users_with_invalid_emails) | **GET** /api/v4/users/invalid_emails | Get users with invalid emails
[**login**](UsersApi.md#login) | **POST** /api/v4/users/login | Login to Mattermost server
[**login_by_cws_token**](UsersApi.md#login_by_cws_token) | **POST** /api/v4/users/login/cws | Auto-Login to Mattermost server using CWS token
[**logout**](UsersApi.md#logout) | **POST** /api/v4/users/logout | Logout from the Mattermost server
[**migrate_auth_to_ldap**](UsersApi.md#migrate_auth_to_ldap) | **POST** /api/v4/users/migrate_auth/ldap | Migrate user accounts authentication type to LDAP.
[**migrate_auth_to_saml**](UsersApi.md#migrate_auth_to_saml) | **POST** /api/v4/users/migrate_auth/saml | Migrate user accounts authentication type to SAML.
[**patch_user**](UsersApi.md#patch_user) | **PUT** /api/v4/users/{user_id}/patch | Patch a user
[**permanent_delete_all_users**](UsersApi.md#permanent_delete_all_users) | **DELETE** /api/v4/users | Permanent delete all users
[**promote_guest_to_user**](UsersApi.md#promote_guest_to_user) | **POST** /api/v4/users/{user_id}/promote | Promote a guest to user
[**publish_user_typing**](UsersApi.md#publish_user_typing) | **POST** /api/v4/users/{user_id}/typing | Publish a user typing websocket event.
[**register_terms_of_service_action**](UsersApi.md#register_terms_of_service_action) | **POST** /api/v4/users/{user_id}/terms_of_service | Records user action when they accept or decline custom terms of service
[**reset_password**](UsersApi.md#reset_password) | **POST** /api/v4/users/password/reset | Reset password
[**revoke_all_sessions**](UsersApi.md#revoke_all_sessions) | **POST** /api/v4/users/{user_id}/sessions/revoke/all | Revoke all active sessions for a user
[**revoke_session**](UsersApi.md#revoke_session) | **POST** /api/v4/users/{user_id}/sessions/revoke | Revoke a user session
[**revoke_sessions_from_all_users**](UsersApi.md#revoke_sessions_from_all_users) | **POST** /api/v4/users/sessions/revoke/all | Revoke all sessions from all users.
[**revoke_user_access_token**](UsersApi.md#revoke_user_access_token) | **POST** /api/v4/users/tokens/revoke | Revoke a user access token
[**search_user_access_tokens**](UsersApi.md#search_user_access_tokens) | **POST** /api/v4/users/tokens/search | Search tokens
[**search_users**](UsersApi.md#search_users) | **POST** /api/v4/users/search | Search users
[**send_password_reset_email**](UsersApi.md#send_password_reset_email) | **POST** /api/v4/users/password/reset/send | Send password reset email
[**send_verification_email**](UsersApi.md#send_verification_email) | **POST** /api/v4/users/email/verify/send | Send verification email
[**set_default_profile_image**](UsersApi.md#set_default_profile_image) | **DELETE** /api/v4/users/{user_id}/image | Delete user's profile image
[**set_profile_image**](UsersApi.md#set_profile_image) | **POST** /api/v4/users/{user_id}/image | Set user's profile image
[**switch_account_type**](UsersApi.md#switch_account_type) | **POST** /api/v4/users/login/switch | Switch login method
[**update_user**](UsersApi.md#update_user) | **PUT** /api/v4/users/{user_id} | Update a user
[**update_user_active**](UsersApi.md#update_user_active) | **PUT** /api/v4/users/{user_id}/active | Update user active status
[**update_user_auth**](UsersApi.md#update_user_auth) | **PUT** /api/v4/users/{user_id}/auth | Update a user's authentication method
[**update_user_mfa**](UsersApi.md#update_user_mfa) | **PUT** /api/v4/users/{user_id}/mfa | Update a user's MFA
[**update_user_password**](UsersApi.md#update_user_password) | **PUT** /api/v4/users/{user_id}/password | Update a user's password
[**update_user_roles**](UsersApi.md#update_user_roles) | **PUT** /api/v4/users/{user_id}/roles | Update a user's roles
[**verify_user_email**](UsersApi.md#verify_user_email) | **POST** /api/v4/users/email/verify | Verify user email
[**verify_user_email_without_token**](UsersApi.md#verify_user_email_without_token) | **POST** /api/v4/users/{user_id}/email/verify/member | Verify user email by ID



## attach_device_extra_props

> models::StatusOk attach_device_extra_props(attach_device_extra_props_request)
Attach mobile device and extra props to the session object

Attach extra props to the session object of the currently logged in session. Adding a mobile device id will enable push notifications for a user, if configured by the server. Other props are also available, like whether the device has notifications disabled and the mobile version. ##### Permissions Must be authenticated. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attach_device_extra_props_request** | [**AttachDeviceExtraPropsRequest**](AttachDeviceExtraPropsRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## autocomplete_users

> models::UserAutocomplete autocomplete_users(name, team_id, channel_id, limit)
Autocomplete users

Get a list of users for the purpose of autocompleting based on the provided search term. Specify a combination of `team_id` and `channel_id` to filter results further. ##### Permissions Requires an active session and `view_team` and `read_channel` on any teams or channels used to filter the results further. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Username, nickname first name or last name | [required] |
**team_id** | Option<**String**> | Team ID |  |
**channel_id** | Option<**String**> | Channel ID |  |
**limit** | Option<**i32**> | The maximum number of users to return in each subresult  __Available as of server version 5.6. Defaults to `100` if not provided or on an earlier server version.__  |  |[default to 100]

### Return type

[**models::UserAutocomplete**](UserAutocomplete.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## check_user_mfa

> models::CheckUserMfa200Response check_user_mfa(check_user_mfa_request)
Check MFA

Check if a user has multi-factor authentication active on their account by providing a login id. Used to check whether an MFA code needs to be provided when logging in. ##### Permissions No permission required. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**check_user_mfa_request** | [**CheckUserMfaRequest**](CheckUserMfaRequest.md) |  | [required] |

### Return type

[**models::CheckUserMfa200Response**](CheckUserMfa_200_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## convert_bot_to_user_0

> models::StatusOk convert_bot_to_user_0(bot_user_id, convert_bot_to_user_request, set_system_admin)
Convert a bot into a user

Convert a bot into a user.  __Minimum server version__: 5.26  ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**bot_user_id** | **String** | Bot user ID | [required] |
**convert_bot_to_user_request** | [**ConvertBotToUserRequest**](ConvertBotToUserRequest.md) | Data to be used in the user creation | [required] |
**set_system_admin** | Option<**bool**> | Whether to give the user the system admin role. |  |[default to false]

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## convert_user_to_bot_0

> models::StatusOk convert_user_to_bot_0(user_id)
Convert a user into a bot

Convert a user into a bot.  __Minimum server version__: 5.26  ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_user

> models::User create_user(create_user_request, t, iid)
Create a user

Create a new user on the system. Password is required for email login. For other authentication types such as LDAP or SAML, auth_data and auth_service fields are required. ##### Permissions No permission required for creating email/username accounts on an open server. Auth Token is required for other authentication types such as LDAP or SAML. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_user_request** | [**CreateUserRequest**](CreateUserRequest.md) | User object to be created | [required] |
**t** | Option<**String**> | Token id from an email invitation |  |
**iid** | Option<**String**> | Token id from an invitation link |  |

### Return type

[**models::User**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_user_access_token

> models::UserAccessToken create_user_access_token(user_id, create_user_access_token_request)
Create a user access token

Generate a user access token that can be used to authenticate with the Mattermost REST API.  __Minimum server version__: 4.1  ##### Permissions Must have `create_user_access_token` permission. For non-self requests, must also have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**create_user_access_token_request** | [**CreateUserAccessTokenRequest**](CreateUserAccessTokenRequest.md) |  | [required] |

### Return type

[**models::UserAccessToken**](UserAccessToken.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_user

> models::StatusOk delete_user(user_id)
Deactivate a user account.

Deactivates the user and revokes all its sessions by archiving its user object.  As of server version 5.28, optionally use the `permanent=true` query parameter to permanently delete the user for compliance reasons. To use this feature `ServiceSettings.EnableAPIUserDeletion` must be set to `true` in the server's configuration. ##### Permissions Must be logged in as the user being deactivated or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## demote_user_to_guest

> models::StatusOk demote_user_to_guest(user_id)
Demote a user to a guest

Convert a regular user into a guest. This will convert the user into a guest for the whole system while retaining their existing team and channel memberships.  __Minimum server version__: 5.16  ##### Permissions Must be logged in as the user or have the `demote_to_guest` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## disable_user_access_token

> models::StatusOk disable_user_access_token(disable_user_access_token_request)
Disable personal access token

Disable a personal access token and delete any sessions using the token. The token can be re-enabled using `/users/tokens/enable`.  __Minimum server version__: 4.4  ##### Permissions Must have `revoke_user_access_token` permission. For non-self requests, must also have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**disable_user_access_token_request** | [**DisableUserAccessTokenRequest**](DisableUserAccessTokenRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## enable_user_access_token

> models::StatusOk enable_user_access_token(enable_user_access_token_request)
Enable personal access token

Re-enable a personal access token that has been disabled.  __Minimum server version__: 4.4  ##### Permissions Must have `create_user_access_token` permission. For non-self requests, must also have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**enable_user_access_token_request** | [**EnableUserAccessTokenRequest**](EnableUserAccessTokenRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## generate_mfa_secret

> models::GenerateMfaSecret200Response generate_mfa_secret(user_id)
Generate MFA secret

Generates an multi-factor authentication secret for a user and returns it as a string and as base64 encoded QR code image. ##### Permissions Must be logged in as the user or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**models::GenerateMfaSecret200Response**](GenerateMfaSecret_200_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_channel_members_with_team_data_for_user

> Vec<models::ChannelMemberWithTeamData> get_channel_members_with_team_data_for_user(user_id, page, per_page)
Get all channel members from all teams for a user

Get all channel members from all teams for a user.  __Minimum server version__: 6.2.0  ##### Permissions Logged in as the user, or have `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | The ID of the user. This can also be \"me\" which will point to the current user. | [required] |
**page** | Option<**i32**> | Page specifies which part of the results to return, by perPage. |  |
**per_page** | Option<**i32**> | The size of the returned chunk of results. |  |[default to 60]

### Return type

[**Vec<models::ChannelMemberWithTeamData>**](ChannelMemberWithTeamData.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_default_profile_image

> get_default_profile_image(user_id)
Return user's default (generated) profile image

Returns the default (generated) user profile image based on user_id string parameter. ##### Permissions Must be logged in. __Minimum server version__: 5.5 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_known_users

> Vec<String> get_known_users()
Get user IDs of known users

Get the list of user IDs of users with any direct relationship with a user. That means any user sharing any channel, including direct and group channels. ##### Permissions Must be authenticated.  __Minimum server version__: 5.23 

### Parameters

This endpoint does not need any parameter.

### Return type

**Vec<String>**

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_profile_image

> get_profile_image(user_id, _)
Get user's profile image

Get a user's profile image based on user_id string parameter. ##### Permissions Must be logged in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**_** | Option<**f64**> | Not used by the server. Clients can pass in the last picture update time of the user to potentially take advantage of caching |  |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_server_limits

> Vec<models::ServerLimits> get_server_limits()
Gets the server limits for the server

Gets the server limits for the server ##### Permissions Requires `sysconsole_read_user_management_users`. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::ServerLimits>**](ServerLimits.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_sessions

> Vec<models::Session> get_sessions(user_id)
Get user's sessions

Get a list of sessions by providing the user GUID. Sensitive information will be sanitized out. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**Vec<models::Session>**](Session.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_total_users_stats

> models::UsersStats get_total_users_stats()
Get total count of users in the system

Get a total count of users in the system. ##### Permissions Must be authenticated. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::UsersStats**](UsersStats.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_total_users_stats_filtered

> models::UsersStats get_total_users_stats_filtered(in_team, in_channel, include_deleted, include_bots, roles, channel_roles, team_roles)
Get total count of users in the system matching the specified filters

Get a count of users in the system matching the specified filters.  __Minimum server version__: 5.26  ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**in_team** | Option<**String**> | The ID of the team to get user stats for. |  |
**in_channel** | Option<**String**> | The ID of the channel to get user stats for. |  |
**include_deleted** | Option<**bool**> | If deleted accounts should be included in the count. |  |
**include_bots** | Option<**bool**> | If bot accounts should be included in the count. |  |
**roles** | Option<**String**> | Comma separated string used to filter users based on any of the specified system roles  Example: `?roles=system_admin,system_user` will include users that are either system admins or system users  |  |
**channel_roles** | Option<**String**> | Comma separated string used to filter users based on any of the specified channel roles, can only be used in conjunction with `in_channel`  Example: `?in_channel=4eb6axxw7fg3je5iyasnfudc5y&channel_roles=channel_user` will include users that are only channel users and not admins or guests  |  |
**team_roles** | Option<**String**> | Comma separated string used to filter users based on any of the specified team roles, can only be used in conjunction with `in_team`  Example: `?in_team=4eb6axxw7fg3je5iyasnfudc5y&team_roles=team_user` will include users that are only team users and not admins or guests  |  |

### Return type

[**models::UsersStats**](UsersStats.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_uploads_for_user

> Vec<models::UploadSession> get_uploads_for_user(user_id)
Get uploads for a user

Gets all the upload sessions belonging to a user.  __Minimum server version__: 5.28  ##### Permissions Must be logged in as the user who created the upload sessions. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | The ID of the user. This can also be \"me\" which will point to the current user. | [required] |

### Return type

[**Vec<models::UploadSession>**](UploadSession.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user

> models::User get_user(user_id)
Get a user

Get a user a object. Sensitive information will be sanitized out. ##### Permissions Requires an active session but no other permissions. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID. This can also be \"me\" which will point to the current user. | [required] |

### Return type

[**models::User**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_access_token

> models::UserAccessTokenSanitized get_user_access_token(token_id)
Get a user access token

Get a user access token. Does not include the actual authentication token.  __Minimum server version__: 4.1  ##### Permissions Must have `read_user_access_token` permission. For non-self requests, must also have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**token_id** | **String** | User access token GUID | [required] |

### Return type

[**models::UserAccessTokenSanitized**](UserAccessTokenSanitized.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_access_tokens

> Vec<models::UserAccessTokenSanitized> get_user_access_tokens(page, per_page)
Get user access tokens

Get a page of user access tokens for users on the system. Does not include the actual authentication tokens. Use query parameters for paging.  __Minimum server version__: 4.7  ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of tokens per page. |  |[default to 60]

### Return type

[**Vec<models::UserAccessTokenSanitized>**](UserAccessTokenSanitized.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_access_tokens_for_user

> Vec<models::UserAccessTokenSanitized> get_user_access_tokens_for_user(user_id, page, per_page)
Get user access tokens

Get a list of user access tokens for a user. Does not include the actual authentication tokens. Use query parameters for paging.  __Minimum server version__: 4.1  ##### Permissions Must have `read_user_access_token` permission. For non-self requests, must also have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of tokens per page. |  |[default to 60]

### Return type

[**Vec<models::UserAccessTokenSanitized>**](UserAccessTokenSanitized.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_audits

> Vec<models::Audit> get_user_audits(user_id)
Get user's audits

Get a list of audit by providing the user GUID. ##### Permissions Must be logged in as the user or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**Vec<models::Audit>**](Audit.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_by_email

> models::User get_user_by_email(email)
Get a user by email

Get a user object by providing a user email. Sensitive information will be sanitized out. ##### Permissions Requires an active session and for the current session to be able to view another user's email based on the server's privacy settings. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**email** | **String** | User Email | [required] |

### Return type

[**models::User**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_by_username

> models::User get_user_by_username(username)
Get a user by username

Get a user object by providing a username. Sensitive information will be sanitized out. ##### Permissions Requires an active session but no other permissions. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**username** | **String** | Username | [required] |

### Return type

[**models::User**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_count_for_reporting

> f64 get_user_count_for_reporting(role_filter, team_filter, has_no_team, hide_active, hide_inactive, search_term)
Gets the full count of users that match the filter.

Get the full count of users admin reporting purposes, based on provided parameters.  Must be a system admin to invoke this API. ##### Permissions Requires `sysconsole_read_user_management_users`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**role_filter** | Option<**String**> | Filter users by their role. |  |
**team_filter** | Option<**String**> | Filter users by a specified team ID. |  |
**has_no_team** | Option<**bool**> | If true, show only users that have no team. Will ignore provided \"team_filter\" if true. |  |
**hide_active** | Option<**bool**> | If true, show only users that are inactive. Cannot be used at the same time as \"hide_inactive\" |  |
**hide_inactive** | Option<**bool**> | If true, show only users that are active. Cannot be used at the same time as \"hide_active\" |  |
**search_term** | Option<**String**> | A filtering search term that allows filtering by Username, FirstName, LastName, Nickname or Email |  |

### Return type

**f64**

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_terms_of_service

> models::UserTermsOfService get_user_terms_of_service(user_id)
Fetches user's latest terms of service action if the latest action was for acceptance.

Will be deprecated in v6.0 Fetches user's latest terms of service action if the latest action was for acceptance.  __Minimum server version__: 5.6 ##### Permissions Must be logged in as the user being acted on. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**models::UserTermsOfService**](UserTermsOfService.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_users

> Vec<models::User> get_users(page, per_page, in_team, not_in_team, in_channel, not_in_channel, in_group, group_constrained, without_team, active, inactive, role, sort, roles, channel_roles, team_roles)
Get users

Get a page of a list of users. Based on query string parameters, select users from a team, channel, or select users not in a specific channel.  Since server version 4.0, some basic sorting is available using the `sort` query parameter. Sorting is currently only supported when selecting users on a team. ##### Permissions Requires an active session and (if specified) membership to the channel or team being selected from. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of users per page. |  |[default to 60]
**in_team** | Option<**String**> | The ID of the team to get users for. |  |
**not_in_team** | Option<**String**> | The ID of the team to exclude users for. Must not be used with \"in_team\" query parameter. |  |
**in_channel** | Option<**String**> | The ID of the channel to get users for. |  |
**not_in_channel** | Option<**String**> | The ID of the channel to exclude users for. Must be used with \"in_channel\" query parameter. |  |
**in_group** | Option<**String**> | The ID of the group to get users for. Must have `manage_system` permission. |  |
**group_constrained** | Option<**bool**> | When used with `not_in_channel` or `not_in_team`, returns only the users that are allowed to join the channel or team based on its group constrains. |  |
**without_team** | Option<**bool**> | Whether or not to list users that are not on any team. This option takes precendence over `in_team`, `in_channel`, and `not_in_channel`. |  |
**active** | Option<**bool**> | Whether or not to list only users that are active. This option cannot be used along with the `inactive` option. |  |
**inactive** | Option<**bool**> | Whether or not to list only users that are deactivated. This option cannot be used along with the `active` option. |  |
**role** | Option<**String**> | Returns users that have this role. |  |
**sort** | Option<**String**> | Sort is only available in conjunction with certain options below. The paging parameter is also always available.  ##### `in_team` Can be \"\", \"last_activity_at\" or \"create_at\". When left blank, sorting is done by username. Note that when \"last_activity_at\" is specified, an additional \"last_activity_at\" field will be returned in the response packet. __Minimum server version__: 4.0 ##### `in_channel` Can be \"\", \"status\". When left blank, sorting is done by username. `status` will sort by User's current status (Online, Away, DND, Offline), then by Username. __Minimum server version__: 4.7 ##### `in_group` Can be \"\", \"display_name\". When left blank, sorting is done by username. `display_name` will sort alphabetically by user's display name. __Minimum server version__: 7.7  |  |
**roles** | Option<**String**> | Comma separated string used to filter users based on any of the specified system roles  Example: `?roles=system_admin,system_user` will return users that are either system admins or system users  __Minimum server version__: 5.26  |  |
**channel_roles** | Option<**String**> | Comma separated string used to filter users based on any of the specified channel roles, can only be used in conjunction with `in_channel`  Example: `?in_channel=4eb6axxw7fg3je5iyasnfudc5y&channel_roles=channel_user` will return users that are only channel users and not admins or guests  __Minimum server version__: 5.26  |  |
**team_roles** | Option<**String**> | Comma separated string used to filter users based on any of the specified team roles, can only be used in conjunction with `in_team`  Example: `?in_team=4eb6axxw7fg3je5iyasnfudc5y&team_roles=team_user` will return users that are only team users and not admins or guests  __Minimum server version__: 5.26  |  |

### Return type

[**Vec<models::User>**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_users_by_group_channel_ids

> models::GetUsersByGroupChannelIds200Response get_users_by_group_channel_ids(request_body)
Get users by group channels ids

Get an object containing a key per group channel id in the query and its value as a list of users members of that group channel.  The user must be a member of the group ids in the query, or they will be omitted from the response. ##### Permissions Requires an active session but no other permissions.  __Minimum server version__: 5.14 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_body** | [**Vec<String>**](String.md) | List of group channel ids | [required] |

### Return type

[**models::GetUsersByGroupChannelIds200Response**](GetUsersByGroupChannelIds_200_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_users_by_ids

> Vec<models::User> get_users_by_ids(request_body, since)
Get users by ids

Get a list of users based on a provided list of user ids. ##### Permissions Requires an active session but no other permissions. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_body** | [**Vec<String>**](String.md) | List of user ids | [required] |
**since** | Option<**i32**> | Only return users that have been modified since the given Unix timestamp (in milliseconds).  __Minimum server version__: 5.14  |  |

### Return type

[**Vec<models::User>**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_users_by_usernames

> Vec<models::User> get_users_by_usernames(request_body)
Get users by usernames

Get a list of users based on a provided list of usernames. ##### Permissions Requires an active session but no other permissions. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_body** | [**Vec<String>**](String.md) | List of usernames | [required] |

### Return type

[**Vec<models::User>**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_users_for_reporting

> Vec<models::UserReport> get_users_for_reporting(sort_column, direction, sort_direction, page_size, from_column_value, from_id, date_range, role_filter, team_filter, has_no_team, hide_active, hide_inactive, search_term)
Get a list of paged and sorted users for admin reporting purposes

Get a list of paged users for admin reporting purposes, based on provided parameters. Must be a system admin to invoke this API. ##### Permissions Requires `sysconsole_read_user_management_users`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**sort_column** | Option<**String**> | The column to sort the users by. Must be one of (\"CreateAt\", \"Username\", \"FirstName\", \"LastName\", \"Nickname\", \"Email\") or the API will return an error. |  |[default to Username]
**direction** | Option<**String**> | The direction in which to accept paging values from. Will return values ahead of the cursor if \"up\", and below the cursor if \"down\". Default is \"down\". |  |[default to down]
**sort_direction** | Option<**String**> | The sorting direction. Must be one of (\"asc\", \"desc\"). Will default to 'asc' if not specified or the input is invalid. |  |[default to asc]
**page_size** | Option<**i32**> | The maximum number of users to return. |  |[default to 50]
**from_column_value** | Option<**String**> | The value of the sorted column corresponding to the cursor to read from. Should be blank for the first page asked for. |  |
**from_id** | Option<**String**> | The value of the user id corresponding to the cursor to read from. Should be blank for the first page asked for. |  |
**date_range** | Option<**String**> | The date range of the post statistics to display. Must be one of (\"last30days\", \"previousmonth\", \"last6months\", \"alltime\"). Will default to 'alltime' if the input is not valid. |  |[default to alltime]
**role_filter** | Option<**String**> | Filter users by their role. |  |
**team_filter** | Option<**String**> | Filter users by a specified team ID. |  |
**has_no_team** | Option<**bool**> | If true, show only users that have no team. Will ignore provided \"team_filter\" if true. |  |
**hide_active** | Option<**bool**> | If true, show only users that are inactive. Cannot be used at the same time as \"hide_inactive\" |  |
**hide_inactive** | Option<**bool**> | If true, show only users that are active. Cannot be used at the same time as \"hide_active\" |  |
**search_term** | Option<**String**> | A filtering search term that allows filtering by Username, FirstName, LastName, Nickname or Email |  |

### Return type

[**Vec<models::UserReport>**](UserReport.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_users_with_invalid_emails

> Vec<models::User> get_users_with_invalid_emails(page, per_page)
Get users with invalid emails

Get users whose emails are considered invalid. It is an error to invoke this API if your team settings enable an open server. ##### Permissions Requires `sysconsole_read_user_management_users`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of users per page. |  |[default to 60]

### Return type

[**Vec<models::User>**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## login

> models::User login(login_request)
Login to Mattermost server

##### Permissions No permission required 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**login_request** | [**LoginRequest**](LoginRequest.md) | User authentication object | [required] |

### Return type

[**models::User**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## login_by_cws_token

> login_by_cws_token(login_by_cws_token_request)
Auto-Login to Mattermost server using CWS token

CWS stands for Customer Web Server which is the cloud service used to manage cloud instances. ##### Permissions A Cloud license is required 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**login_by_cws_token_request** | [**LoginByCwsTokenRequest**](LoginByCwsTokenRequest.md) | User authentication object | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## logout

> models::StatusOk logout()
Logout from the Mattermost server

##### Permissions An active session is required 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## migrate_auth_to_ldap

> migrate_auth_to_ldap(migrate_auth_to_ldap_request)
Migrate user accounts authentication type to LDAP.

Migrates accounts from one authentication provider to another. For example, you can upgrade your authentication provider from email to LDAP. __Minimum server version__: 5.28 ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**migrate_auth_to_ldap_request** | Option<[**MigrateAuthToLdapRequest**](MigrateAuthToLdapRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## migrate_auth_to_saml

> migrate_auth_to_saml(migrate_auth_to_saml_request)
Migrate user accounts authentication type to SAML.

Migrates accounts from one authentication provider to another. For example, you can upgrade your authentication provider from email to SAML. __Minimum server version__: 5.28 ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**migrate_auth_to_saml_request** | Option<[**MigrateAuthToSamlRequest**](MigrateAuthToSamlRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## patch_user

> models::User patch_user(user_id, patch_user_request)
Patch a user

Partially update a user by providing only the fields you want to update. Omitted fields will not be updated. The fields that can be updated are defined in the request body, all other provided fields will be ignored. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**patch_user_request** | [**PatchUserRequest**](PatchUserRequest.md) | User object that is to be updated | [required] |

### Return type

[**models::User**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## permanent_delete_all_users

> permanent_delete_all_users()
Permanent delete all users

Permanently deletes all users and all their related information, including posts.  __Minimum server version__: 5.26.0  __Local mode only__: This endpoint is only available through [local mode](https://docs.mattermost.com/administration/mmctl-cli-tool.html#local-mode). 

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## promote_guest_to_user

> models::StatusOk promote_guest_to_user(user_id)
Promote a guest to user

Convert a guest into a regular user. This will convert the guest into a user for the whole system while retaining any team and channel memberships and automatically joining them to the default channels.  __Minimum server version__: 5.16  ##### Permissions Must be logged in as the user or have the `promote_guest` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## publish_user_typing

> publish_user_typing(user_id, publish_user_typing_request)
Publish a user typing websocket event.

Notify users in the given channel via websocket that the given user is typing. __Minimum server version__: 5.26 ##### Permissions Must have `manage_system` permission to publish for any user other than oneself. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**publish_user_typing_request** | Option<[**PublishUserTypingRequest**](PublishUserTypingRequest.md)> |  |  |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## register_terms_of_service_action

> models::StatusOk register_terms_of_service_action(user_id, register_terms_of_service_action_request)
Records user action when they accept or decline custom terms of service

Records user action when they accept or decline custom terms of service. Records the action in audit table. Updates user's last accepted terms of service ID if they accepted it.  __Minimum server version__: 5.4 ##### Permissions Must be logged in as the user being acted on. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**register_terms_of_service_action_request** | [**RegisterTermsOfServiceActionRequest**](RegisterTermsOfServiceActionRequest.md) | terms of service details | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reset_password

> models::StatusOk reset_password(reset_password_request)
Reset password

Update the password for a user using a one-use, timed recovery code tied to the user's account. Only works for non-SSO users. ##### Permissions No permissions required. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**reset_password_request** | [**ResetPasswordRequest**](ResetPasswordRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## revoke_all_sessions

> models::StatusOk revoke_all_sessions(user_id)
Revoke all active sessions for a user

Revokes all user sessions from the provided user id and session id strings. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. __Minimum server version__: 4.4 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## revoke_session

> models::StatusOk revoke_session(user_id, revoke_session_request)
Revoke a user session

Revokes a user session from the provided user id and session id strings. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**revoke_session_request** | [**RevokeSessionRequest**](RevokeSessionRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## revoke_sessions_from_all_users

> revoke_sessions_from_all_users()
Revoke all sessions from all users.

For any session currently on the server (including admin) it will be revoked. Clients will be notified to log out users.  __Minimum server version__: 5.14  ##### Permissions Must have `manage_system` permission. 

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## revoke_user_access_token

> models::StatusOk revoke_user_access_token(revoke_user_access_token_request)
Revoke a user access token

Revoke a user access token and delete any sessions using the token.  __Minimum server version__: 4.1  ##### Permissions Must have `revoke_user_access_token` permission. For non-self requests, must also have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**revoke_user_access_token_request** | [**RevokeUserAccessTokenRequest**](RevokeUserAccessTokenRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_user_access_tokens

> Vec<models::UserAccessTokenSanitized> search_user_access_tokens(search_user_access_tokens_request)
Search tokens

Get a list of tokens based on search criteria provided in the request body. Searches are done against the token id, user id and username.  __Minimum server version__: 4.7  ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**search_user_access_tokens_request** | [**SearchUserAccessTokensRequest**](SearchUserAccessTokensRequest.md) | Search criteria | [required] |

### Return type

[**Vec<models::UserAccessTokenSanitized>**](UserAccessTokenSanitized.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_users

> Vec<models::User> search_users(search_users_request)
Search users

Get a list of users based on search criteria provided in the request body. Searches are typically done against username, full name, nickname and email unless otherwise configured by the server. ##### Permissions Requires an active session and `read_channel` and/or `view_team` permissions for any channels or teams specified in the request body. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**search_users_request** | [**SearchUsersRequest**](SearchUsersRequest.md) | Search criteria | [required] |

### Return type

[**Vec<models::User>**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## send_password_reset_email

> models::StatusOk send_password_reset_email(send_password_reset_email_request)
Send password reset email

Send an email containing a link for resetting the user's password. The link will contain a one-use, timed recovery code tied to the user's account. Only works for non-SSO users. ##### Permissions No permissions required. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**send_password_reset_email_request** | [**SendPasswordResetEmailRequest**](SendPasswordResetEmailRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## send_verification_email

> models::StatusOk send_verification_email(send_verification_email_request)
Send verification email

Send an email with a verification link to a user that has an email matching the one in the request body. This endpoint will return success even if the email does not match any users on the system. ##### Permissions No permissions required. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**send_verification_email_request** | [**SendVerificationEmailRequest**](SendVerificationEmailRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_default_profile_image

> models::StatusOk set_default_profile_image(user_id)
Delete user's profile image

Delete user's profile image and reset to default image based on user_id string parameter. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. __Minimum server version__: 5.5 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_profile_image

> models::StatusOk set_profile_image(user_id, image)
Set user's profile image

Set a user's profile image based on user_id string parameter. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**image** | **std::path::PathBuf** | The image to be uploaded | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## switch_account_type

> models::SwitchAccountType200Response switch_account_type(switch_account_type_request)
Switch login method

Switch a user's login method from using email to OAuth2/SAML/LDAP or back to email. When switching to OAuth2/SAML, account switching is not complete until the user follows the returned link and completes any steps on the OAuth2/SAML service provider.  To switch from email to OAuth2/SAML, specify `current_service`, `new_service`, `email` and `password`.  To switch from OAuth2/SAML to email, specify `current_service`, `new_service`, `email` and `new_password`.  To switch from email to LDAP/AD, specify `current_service`, `new_service`, `email`, `password`, `ldap_ip` and `new_password` (this is the user's LDAP password).  To switch from LDAP/AD to email, specify `current_service`, `new_service`, `ldap_ip`, `password` (this is the user's LDAP password), `email`  and `new_password`.  Additionally, specify `mfa_code` when trying to switch an account on LDAP/AD or email that has MFA activated.  ##### Permissions No current authentication required except when switching from OAuth2/SAML to email. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**switch_account_type_request** | [**SwitchAccountTypeRequest**](SwitchAccountTypeRequest.md) |  | [required] |

### Return type

[**models::SwitchAccountType200Response**](SwitchAccountType_200_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_user

> models::User update_user(user_id, update_user_request)
Update a user

Update a user by providing the user object. The fields that can be updated are defined in the request body, all other provided fields will be ignored. Any fields not included in the request body will be set to null or reverted to default values. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**update_user_request** | [**UpdateUserRequest**](UpdateUserRequest.md) | User object that is to be updated | [required] |

### Return type

[**models::User**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_user_active

> models::StatusOk update_user_active(user_id, update_user_active_request)
Update user active status

Update user active or inactive status.  __Since server version 4.6, users using a SSO provider to login can be activated or deactivated with this endpoint. However, if their activation status in Mattermost does not reflect their status in the SSO provider, the next synchronization or login by that user will reset the activation status to that of their account in the SSO provider. Server versions 4.5 and before do not allow activation or deactivation of SSO users from this endpoint.__ ##### Permissions User can deactivate themselves. User with `manage_system` permission can activate or deactivate a user. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**update_user_active_request** | [**UpdateUserActiveRequest**](UpdateUserActiveRequest.md) | Use `true` to set the user active, `false` for inactive | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_user_auth

> models::UserAuthData update_user_auth(user_id, user_auth_data)
Update a user's authentication method

Updates a user's authentication method. This can be used to change them to/from LDAP authentication for example.  __Minimum server version__: 4.6 ##### Permissions Must have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**user_auth_data** | [**UserAuthData**](UserAuthData.md) |  | [required] |

### Return type

[**models::UserAuthData**](UserAuthData.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_user_mfa

> models::StatusOk update_user_mfa(user_id, update_user_mfa_request)
Update a user's MFA

Activates multi-factor authentication for the user if `activate` is true and a valid `code` is provided. If activate is false, then `code` is not required and multi-factor authentication is disabled for the user. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**update_user_mfa_request** | [**UpdateUserMfaRequest**](UpdateUserMfaRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_user_password

> models::StatusOk update_user_password(user_id, update_user_password_request)
Update a user's password

Update a user's password. New password must meet password policy set by server configuration. Current password is required if you're updating your own password. ##### Permissions Must be logged in as the user the password is being changed for or have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**update_user_password_request** | [**UpdateUserPasswordRequest**](UpdateUserPasswordRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_user_roles

> models::StatusOk update_user_roles(user_id, update_user_roles_request)
Update a user's roles

Update a user's system-level roles. Valid user roles are \"system_user\", \"system_admin\" or both of them. Overwrites any previously assigned system-level roles. ##### Permissions Must have the `manage_roles` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**update_user_roles_request** | [**UpdateUserRolesRequest**](UpdateUserRolesRequest.md) | Space-delimited system roles to assign to the user | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## verify_user_email

> models::StatusOk verify_user_email(verify_user_email_request)
Verify user email

Verify the email used by a user to sign-up their account with. ##### Permissions No permissions required. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**verify_user_email_request** | [**VerifyUserEmailRequest**](VerifyUserEmailRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## verify_user_email_without_token

> models::User verify_user_email_without_token(user_id)
Verify user email by ID

Verify the email used by a user without a token.  __Minimum server version__: 5.24  ##### Permissions  Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**models::User**](User.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

