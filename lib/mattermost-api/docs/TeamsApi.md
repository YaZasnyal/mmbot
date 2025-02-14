# \TeamsApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_team_member**](TeamsApi.md#add_team_member) | **POST** /api/v4/teams/{team_id}/members | Add user to team
[**add_team_member_from_invite**](TeamsApi.md#add_team_member_from_invite) | **POST** /api/v4/teams/members/invite | Add user to team from invite
[**add_team_members**](TeamsApi.md#add_team_members) | **POST** /api/v4/teams/{team_id}/members/batch | Add multiple users to team
[**create_team**](TeamsApi.md#create_team) | **POST** /api/v4/teams | Create a team
[**get_all_teams**](TeamsApi.md#get_all_teams) | **GET** /api/v4/teams | Get teams
[**get_team**](TeamsApi.md#get_team) | **GET** /api/v4/teams/{team_id} | Get a team
[**get_team_by_name**](TeamsApi.md#get_team_by_name) | **GET** /api/v4/teams/name/{name} | Get a team by name
[**get_team_icon**](TeamsApi.md#get_team_icon) | **GET** /api/v4/teams/{team_id}/image | Get the team icon
[**get_team_invite_info**](TeamsApi.md#get_team_invite_info) | **GET** /api/v4/teams/invite/{invite_id} | Get invite info for a team
[**get_team_member**](TeamsApi.md#get_team_member) | **GET** /api/v4/teams/{team_id}/members/{user_id} | Get a team member
[**get_team_members**](TeamsApi.md#get_team_members) | **GET** /api/v4/teams/{team_id}/members | Get team members
[**get_team_members_by_ids**](TeamsApi.md#get_team_members_by_ids) | **POST** /api/v4/teams/{team_id}/members/ids | Get team members by ids
[**get_team_members_for_user**](TeamsApi.md#get_team_members_for_user) | **GET** /api/v4/users/{user_id}/teams/members | Get team members for a user
[**get_team_stats**](TeamsApi.md#get_team_stats) | **GET** /api/v4/teams/{team_id}/stats | Get a team stats
[**get_team_unread**](TeamsApi.md#get_team_unread) | **GET** /api/v4/users/{user_id}/teams/{team_id}/unread | Get unreads for a team
[**get_teams_for_user**](TeamsApi.md#get_teams_for_user) | **GET** /api/v4/users/{user_id}/teams | Get a user's teams
[**get_teams_unread_for_user**](TeamsApi.md#get_teams_unread_for_user) | **GET** /api/v4/users/{user_id}/teams/unread | Get team unreads for a user
[**import_team**](TeamsApi.md#import_team) | **POST** /api/v4/teams/{team_id}/import | Import a Team from other application
[**invalidate_email_invites**](TeamsApi.md#invalidate_email_invites) | **DELETE** /api/v4/teams/invites/email | Invalidate active email invitations
[**invite_guests_to_team**](TeamsApi.md#invite_guests_to_team) | **POST** /api/v4/teams/{team_id}/invite-guests/email | Invite guests to the team by email
[**invite_users_to_team**](TeamsApi.md#invite_users_to_team) | **POST** /api/v4/teams/{team_id}/invite/email | Invite users to the team by email
[**patch_team**](TeamsApi.md#patch_team) | **PUT** /api/v4/teams/{team_id}/patch | Patch a team
[**regenerate_team_invite_id**](TeamsApi.md#regenerate_team_invite_id) | **POST** /api/v4/teams/{team_id}/regenerate_invite_id | Regenerate the Invite ID from a Team
[**remove_team_icon**](TeamsApi.md#remove_team_icon) | **DELETE** /api/v4/teams/{team_id}/image | Remove the team icon
[**remove_team_member**](TeamsApi.md#remove_team_member) | **DELETE** /api/v4/teams/{team_id}/members/{user_id} | Remove user from team
[**restore_team**](TeamsApi.md#restore_team) | **POST** /api/v4/teams/{team_id}/restore | Restore a team
[**search_files**](TeamsApi.md#search_files) | **POST** /api/v4/teams/{team_id}/files/search | Search files in a team
[**search_teams**](TeamsApi.md#search_teams) | **POST** /api/v4/teams/search | Search teams
[**set_team_icon**](TeamsApi.md#set_team_icon) | **POST** /api/v4/teams/{team_id}/image | Sets the team icon
[**soft_delete_team**](TeamsApi.md#soft_delete_team) | **DELETE** /api/v4/teams/{team_id} | Delete a team
[**team_exists**](TeamsApi.md#team_exists) | **GET** /api/v4/teams/name/{name}/exists | Check if team exists
[**team_members_minus_group_members**](TeamsApi.md#team_members_minus_group_members) | **GET** /api/v4/teams/{team_id}/members_minus_group_members | Team members minus group members.
[**update_team**](TeamsApi.md#update_team) | **PUT** /api/v4/teams/{team_id} | Update a team
[**update_team_member_roles**](TeamsApi.md#update_team_member_roles) | **PUT** /api/v4/teams/{team_id}/members/{user_id}/roles | Update a team member roles
[**update_team_member_scheme_roles**](TeamsApi.md#update_team_member_scheme_roles) | **PUT** /api/v4/teams/{team_id}/members/{user_id}/schemeRoles | Update the scheme-derived roles of a team member.
[**update_team_privacy**](TeamsApi.md#update_team_privacy) | **PUT** /api/v4/teams/{team_id}/privacy | Update teams's privacy
[**update_team_scheme**](TeamsApi.md#update_team_scheme) | **PUT** /api/v4/teams/{team_id}/scheme | Set a team's scheme



## add_team_member

> models::TeamMember add_team_member(team_id, add_team_member_request)
Add user to team

Add user to the team by user_id. ##### Permissions Must be authenticated and team be open to add self. For adding another user, authenticated user must have the `add_user_to_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**add_team_member_request** | [**AddTeamMemberRequest**](AddTeamMemberRequest.md) |  | [required] |

### Return type

[**models::TeamMember**](TeamMember.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## add_team_member_from_invite

> models::TeamMember add_team_member_from_invite(token)
Add user to team from invite

Using either an invite id or hash/data pair from an email invite link, add a user to a team. ##### Permissions Must be authenticated. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**token** | **String** | Token id from the invitation | [required] |

### Return type

[**models::TeamMember**](TeamMember.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## add_team_members

> Vec<models::TeamMember> add_team_members(team_id, team_member, graceful)
Add multiple users to team

Add a number of users to the team by user_id. ##### Permissions Must be authenticated. Authenticated user must have the `add_user_to_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**team_member** | [**Vec<models::TeamMember>**](TeamMember.md) |  | [required] |
**graceful** | Option<**bool**> | Instead of aborting the operation if a user cannot be added, return an arrray that will contain both the success and added members and the ones with error, in form of `[{\"member\": {...}, \"user_id\", \"...\", \"error\": {...}}]` |  |

### Return type

[**Vec<models::TeamMember>**](TeamMember.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_team

> models::Team create_team(create_team_request)
Create a team

Create a new team on the system. ##### Permissions Must be authenticated and have the `create_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_team_request** | [**CreateTeamRequest**](CreateTeamRequest.md) | Team that is to be created | [required] |

### Return type

[**models::Team**](Team.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_teams

> Vec<models::Team> get_all_teams(page, per_page, include_total_count, exclude_policy_constrained)
Get teams

For regular users only returns open teams. Users with the \"manage_system\" permission will return teams regardless of type. The result is based on query string parameters - page and per_page. ##### Permissions Must be authenticated. \"manage_system\" permission is required to show all teams. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of teams per page. |  |[default to 60]
**include_total_count** | Option<**bool**> | Appends a total count of returned teams inside the response object - ex: `{ \"teams\": [], \"total_count\" : 0 }`.       |  |[default to false]
**exclude_policy_constrained** | Option<**bool**> | If set to true, teams which are part of a data retention policy will be excluded. The `sysconsole_read_compliance` permission is required to use this parameter. __Minimum server version__: 5.35 |  |[default to false]

### Return type

[**Vec<models::Team>**](Team.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_team

> models::Team get_team(team_id)
Get a team

Get a team on the system. ##### Permissions Must be authenticated and have the `view_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |

### Return type

[**models::Team**](Team.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_team_by_name

> models::Team get_team_by_name(name)
Get a team by name

Get a team based on provided name string ##### Permissions Must be authenticated, team type is open and have the `view_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Team Name | [required] |

### Return type

[**models::Team**](Team.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_team_icon

> get_team_icon(team_id)
Get the team icon

Get the team icon of the team.  __Minimum server version__: 4.9  ##### Permissions User must be authenticated. In addition, team must be open or the user must have the `view_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_team_invite_info

> models::GetTeamInviteInfo200Response get_team_invite_info(invite_id)
Get invite info for a team

Get the `name`, `display_name`, `description` and `id` for a team from the invite id.  __Minimum server version__: 4.0  ##### Permissions No authentication required. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**invite_id** | **String** | Invite id for a team | [required] |

### Return type

[**models::GetTeamInviteInfo200Response**](GetTeamInviteInfo_200_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_team_member

> models::TeamMember get_team_member(team_id, user_id)
Get a team member

Get a team member on the system. ##### Permissions Must be authenticated and have the `view_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**user_id** | **String** | User GUID | [required] |

### Return type

[**models::TeamMember**](TeamMember.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_team_members

> Vec<models::TeamMember> get_team_members(team_id, page, per_page, sort, exclude_deleted_users)
Get team members

Get a page team members list based on query string parameters - team id, page and per page. ##### Permissions Must be authenticated and have the `view_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of users per page. |  |[default to 60]
**sort** | Option<**String**> | To sort by Username, set to 'Username', otherwise sort is by 'UserID' |  |[default to ]
**exclude_deleted_users** | Option<**bool**> | Excludes deleted users from the results |  |[default to false]

### Return type

[**Vec<models::TeamMember>**](TeamMember.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_team_members_by_ids

> Vec<models::TeamMember> get_team_members_by_ids(team_id, request_body)
Get team members by ids

Get a list of team members based on a provided array of user ids. ##### Permissions Must have `view_team` permission for the team. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**request_body** | [**Vec<String>**](String.md) | List of user ids | [required] |

### Return type

[**Vec<models::TeamMember>**](TeamMember.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_team_members_for_user

> Vec<models::TeamMember> get_team_members_for_user(user_id)
Get team members for a user

Get a list of team members for a user. Useful for getting the ids of teams the user is on and the roles they have in those teams. ##### Permissions Must be logged in as the user or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**Vec<models::TeamMember>**](TeamMember.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_team_stats

> models::TeamStats get_team_stats(team_id)
Get a team stats

Get a team stats on the system. ##### Permissions Must be authenticated and have the `view_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |

### Return type

[**models::TeamStats**](TeamStats.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_team_unread

> models::TeamUnread get_team_unread(user_id, team_id)
Get unreads for a team

Get the unread mention and message counts for a team for the specified user. ##### Permissions Must be the user or have `edit_other_users` permission and have `view_team` permission for the team. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**team_id** | **String** | Team GUID | [required] |

### Return type

[**models::TeamUnread**](TeamUnread.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_teams_for_user

> Vec<models::Team> get_teams_for_user(user_id)
Get a user's teams

Get a list of teams that a user is on. ##### Permissions Must be authenticated as the user or have the `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**Vec<models::Team>**](Team.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_teams_unread_for_user

> Vec<models::TeamUnread> get_teams_unread_for_user(user_id, exclude_team, include_collapsed_threads)
Get team unreads for a user

Get the count for unread messages and mentions in the teams the user is a member of. ##### Permissions Must be logged in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**exclude_team** | **String** | Optional team id to be excluded from the results | [required] |
**include_collapsed_threads** | Option<**bool**> | Boolean to determine whether the collapsed threads should be included or not |  |[default to false]

### Return type

[**Vec<models::TeamUnread>**](TeamUnread.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## import_team

> models::ImportTeam200Response import_team(team_id, file, filesize, import_from)
Import a Team from other application

Import a team into a existing team. Import users, channels, posts, hooks. ##### Permissions Must have `permission_import_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**file** | **std::path::PathBuf** | A file to be uploaded in zip format. | [required] |
**filesize** | **i32** | The size of the zip file to be imported. | [required] |
**import_from** | **String** | String that defines from which application the team was exported to be imported into Mattermost. | [required] |

### Return type

[**models::ImportTeam200Response**](ImportTeam_200_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## invalidate_email_invites

> models::StatusOk invalidate_email_invites()
Invalidate active email invitations

Invalidate active email invitations that have not been accepted by the user. ##### Permissions Must have `sysconsole_write_authentication` permission. 

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


## invite_guests_to_team

> models::StatusOk invite_guests_to_team(team_id, invite_guests_to_team_request)
Invite guests to the team by email

Invite guests to existing team channels usign the user's email.  The number of emails that can be sent is rate limited to 20 per hour with a burst of 20 emails. If the rate limit exceeds, the error message contains details on when to retry and when the timer will be reset.  __Minimum server version__: 5.16  ##### Permissions Must have `invite_guest` permission for the team. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**invite_guests_to_team_request** | [**InviteGuestsToTeamRequest**](InviteGuestsToTeamRequest.md) | Guests invite information | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## invite_users_to_team

> models::StatusOk invite_users_to_team(team_id, request_body)
Invite users to the team by email

Invite users to the existing team using the user's email.  The number of emails that can be sent is rate limited to 20 per hour with a burst of 20 emails. If the rate limit exceeds, the error message contains details on when to retry and when the timer will be reset. ##### Permissions Must have `invite_user` and `add_user_to_team` permissions for the team. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**request_body** | [**Vec<String>**](String.md) | List of user's email | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## patch_team

> models::Team patch_team(team_id, patch_team_request)
Patch a team

Partially update a team by providing only the fields you want to update. Omitted fields will not be updated. The fields that can be updated are defined in the request body, all other provided fields will be ignored. ##### Permissions Must have the `manage_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**patch_team_request** | [**PatchTeamRequest**](PatchTeamRequest.md) | Team object that is to be updated | [required] |

### Return type

[**models::Team**](Team.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## regenerate_team_invite_id

> models::Team regenerate_team_invite_id(team_id)
Regenerate the Invite ID from a Team

Regenerates the invite ID used in invite links of a team ##### Permissions Must be authenticated and have the `manage_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |

### Return type

[**models::Team**](Team.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## remove_team_icon

> models::StatusOk remove_team_icon(team_id)
Remove the team icon

Remove the team icon for the team.  __Minimum server version__: 4.10  ##### Permissions Must be authenticated and have the `manage_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## remove_team_member

> models::StatusOk remove_team_member(team_id, user_id)
Remove user from team

Delete the team member object for a user, effectively removing them from a team. ##### Permissions Must be logged in as the user or have the `remove_user_from_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**user_id** | **String** | User GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## restore_team

> models::Team restore_team(team_id)
Restore a team

Restore a team that was previously soft deleted.  __Minimum server version__: 5.24  ##### Permissions Must have the `manage_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |

### Return type

[**models::Team**](Team.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_files

> models::FileInfoList search_files(team_id, terms, is_or_search, time_zone_offset, include_deleted_channels, page, per_page)
Search files in a team

Search for files in a team based on file name, extention and file content (if file content extraction is enabled and supported for the files). __Minimum server version__: 5.34 ##### Permissions Must be authenticated and have the `view_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**terms** | **String** | The search terms as inputed by the user. To search for files from a user include `from:someusername`, using a user's username. To search in a specific channel include `in:somechannel`, using the channel name (not the display name). To search for specific extensions included `ext:extension`. | [required] |
**is_or_search** | **bool** | Set to true if an Or search should be performed vs an And search. | [required] |
**time_zone_offset** | Option<**i32**> | Offset from UTC of user timezone for date searches. |  |[default to 0]
**include_deleted_channels** | Option<**bool**> | Set to true if deleted channels should be included in the search. (archived channels) |  |
**page** | Option<**i32**> | The page to select. (Only works with Elasticsearch) |  |[default to 0]
**per_page** | Option<**i32**> | The number of posts per page. (Only works with Elasticsearch) |  |[default to 60]

### Return type

[**models::FileInfoList**](FileInfoList.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_teams

> models::SearchTeams200Response search_teams(search_teams_request)
Search teams

Search teams based on search term and options provided in the request body.  ##### Permissions Logged in user only shows open teams Logged in user with \"manage_system\" permission shows all teams 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**search_teams_request** | [**SearchTeamsRequest**](SearchTeamsRequest.md) | Search criteria | [required] |

### Return type

[**models::SearchTeams200Response**](SearchTeams_200_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_team_icon

> models::StatusOk set_team_icon(team_id, image)
Sets the team icon

Sets the team icon for the team.  __Minimum server version__: 4.9  ##### Permissions Must be authenticated and have the `manage_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**image** | **std::path::PathBuf** | The image to be uploaded | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## soft_delete_team

> models::StatusOk soft_delete_team(team_id, permanent)
Delete a team

Soft deletes a team, by marking the team as deleted in the database. Soft deleted teams will not be accessible in the user interface.  Optionally use the permanent query parameter to hard delete the team for compliance reasons. As of server version 5.0, to use this feature `ServiceSettings.EnableAPITeamDeletion` must be set to `true` in the server's configuration. ##### Permissions Must have the `manage_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**permanent** | Option<**bool**> | Permanently delete the team, to be used for compliance reasons only. As of server version 5.0, `ServiceSettings.EnableAPITeamDeletion` must be set to `true` in the server's configuration. |  |[default to false]

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## team_exists

> models::TeamExists team_exists(name)
Check if team exists

Check if the team exists based on a team name. ##### Permissions Must be authenticated. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Team Name | [required] |

### Return type

[**models::TeamExists**](TeamExists.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## team_members_minus_group_members

> team_members_minus_group_members(team_id, group_ids, page, per_page)
Team members minus group members.

Get the set of users who are members of the team minus the set of users who are members of the given groups. Each user object contains an array of group objects representing the group memberships for that user. Each user object contains the boolean fields `scheme_guest`, `scheme_user`, and `scheme_admin` representing the roles that user has for the given team.  ##### Permissions Must have `manage_system` permission.  __Minimum server version__: 5.14 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**group_ids** | **String** | A comma-separated list of group ids. | [required] |[default to ]
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of users per page. |  |[default to 0]

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_team

> models::Team update_team(team_id, update_team_request)
Update a team

Update a team by providing the team object. The fields that can be updated are defined in the request body, all other provided fields will be ignored. ##### Permissions Must have the `manage_team` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**update_team_request** | [**UpdateTeamRequest**](UpdateTeamRequest.md) | Team to update | [required] |

### Return type

[**models::Team**](Team.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_team_member_roles

> models::StatusOk update_team_member_roles(team_id, user_id, update_user_roles_request)
Update a team member roles

Update a team member roles. Valid team roles are \"team_user\", \"team_admin\" or both of them. Overwrites any previously assigned team roles. ##### Permissions Must be authenticated and have the `manage_team_roles` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**user_id** | **String** | User GUID | [required] |
**update_user_roles_request** | [**UpdateUserRolesRequest**](UpdateUserRolesRequest.md) | Space-delimited team roles to assign to the user | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_team_member_scheme_roles

> models::StatusOk update_team_member_scheme_roles(team_id, user_id, update_team_member_scheme_roles_request)
Update the scheme-derived roles of a team member.

Update a team member's scheme_admin/scheme_user properties. Typically this should either be `scheme_admin=false, scheme_user=true` for ordinary team member, or `scheme_admin=true, scheme_user=true` for a team admin.  __Minimum server version__: 5.0  ##### Permissions Must be authenticated and have the `manage_team_roles` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**user_id** | **String** | User GUID | [required] |
**update_team_member_scheme_roles_request** | [**UpdateTeamMemberSchemeRolesRequest**](UpdateTeamMemberSchemeRolesRequest.md) | Scheme properties. | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_team_privacy

> models::Team update_team_privacy(team_id, update_team_privacy_request)
Update teams's privacy

Updates team's privacy allowing changing a team from Public (open) to Private (invitation only) and back.  __Minimum server version__: 5.24  ##### Permissions `manage_team` permission for the team of the team. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**update_team_privacy_request** | [**UpdateTeamPrivacyRequest**](UpdateTeamPrivacyRequest.md) |  | [required] |

### Return type

[**models::Team**](Team.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_team_scheme

> models::StatusOk update_team_scheme(team_id, update_team_scheme_request)
Set a team's scheme

Set a team's scheme, more specifically sets the scheme_id value of a team record.  ##### Permissions Must have `manage_system` permission.  __Minimum server version__: 5.0 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**team_id** | **String** | Team GUID | [required] |
**update_team_scheme_request** | [**UpdateTeamSchemeRequest**](UpdateTeamSchemeRequest.md) | Scheme GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

