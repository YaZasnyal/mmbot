# \PreferencesApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_preferences**](PreferencesApi.md#delete_preferences) | **POST** /api/v4/users/{user_id}/preferences/delete | Delete user's preferences
[**get_preferences**](PreferencesApi.md#get_preferences) | **GET** /api/v4/users/{user_id}/preferences | Get the user's preferences
[**get_preferences_by_category**](PreferencesApi.md#get_preferences_by_category) | **GET** /api/v4/users/{user_id}/preferences/{category} | List a user's preferences by category
[**get_preferences_by_category_by_name**](PreferencesApi.md#get_preferences_by_category_by_name) | **GET** /api/v4/users/{user_id}/preferences/{category}/name/{preference_name} | Get a specific user preference
[**update_preferences**](PreferencesApi.md#update_preferences) | **PUT** /api/v4/users/{user_id}/preferences | Save the user's preferences



## delete_preferences

> models::StatusOk delete_preferences(user_id, preference)
Delete user's preferences

Delete a list of the user's preferences. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**preference** | [**Vec<models::Preference>**](Preference.md) | List of preference objects | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_preferences

> Vec<models::Preference> get_preferences(user_id)
Get the user's preferences

Get a list of the user's preferences. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |

### Return type

[**Vec<models::Preference>**](Preference.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_preferences_by_category

> Vec<models::Preference> get_preferences_by_category(user_id, category)
List a user's preferences by category

Lists the current user's stored preferences in the given category. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**category** | **String** | The category of a group of preferences | [required] |

### Return type

[**Vec<models::Preference>**](Preference.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_preferences_by_category_by_name

> models::Preference get_preferences_by_category_by_name(user_id, category, preference_name)
Get a specific user preference

Gets a single preference for the current user with the given category and name. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**category** | **String** | The category of a group of preferences | [required] |
**preference_name** | **String** | The name of the preference | [required] |

### Return type

[**models::Preference**](Preference.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_preferences

> models::StatusOk update_preferences(user_id, preference)
Save the user's preferences

Save a list of the user's preferences. ##### Permissions Must be logged in as the user being updated or have the `edit_other_users` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User GUID | [required] |
**preference** | [**Vec<models::Preference>**](Preference.md) | List of preference objects | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

