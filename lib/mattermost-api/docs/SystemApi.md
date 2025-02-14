# \SystemApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**check_integrity**](SystemApi.md#check_integrity) | **POST** /api/v4/integrity | Perform a database integrity check
[**clear_server_busy**](SystemApi.md#clear_server_busy) | **DELETE** /api/v4/server_busy | Clears the server busy (high load) flag
[**database_recycle**](SystemApi.md#database_recycle) | **POST** /api/v4/database/recycle | Recycle database connections
[**generate_support_packet**](SystemApi.md#generate_support_packet) | **GET** /api/v4/system/support_packet | Download a zip file which contains helpful and useful information for troubleshooting your mattermost instance.
[**get_analytics_old**](SystemApi.md#get_analytics_old) | **GET** /api/v4/analytics/old | Get analytics
[**get_audits**](SystemApi.md#get_audits) | **GET** /api/v4/audits | Get audits
[**get_client_config**](SystemApi.md#get_client_config) | **GET** /api/v4/config/client | Get client configuration
[**get_client_license**](SystemApi.md#get_client_license) | **GET** /api/v4/license/client | Get client license
[**get_config**](SystemApi.md#get_config) | **GET** /api/v4/config | Get configuration
[**get_environment_config**](SystemApi.md#get_environment_config) | **GET** /api/v4/config/environment | Get configuration made through environment variables
[**get_image_by_url**](SystemApi.md#get_image_by_url) | **GET** /api/v4/image | Get an image by url
[**get_logs**](SystemApi.md#get_logs) | **GET** /api/v4/logs | Get logs
[**get_notices**](SystemApi.md#get_notices) | **GET** /api/v4/system/notices/{teamId} | Get notices for logged in user in specified team
[**get_ping**](SystemApi.md#get_ping) | **GET** /api/v4/system/ping | Check system health
[**get_prev_trial_license**](SystemApi.md#get_prev_trial_license) | **GET** /api/v4/trial-license/prev | Get last trial license used
[**get_redirect_location**](SystemApi.md#get_redirect_location) | **GET** /api/v4/redirect_location | Get redirect location
[**get_server_busy_expires**](SystemApi.md#get_server_busy_expires) | **GET** /api/v4/server_busy | Get server busy expiry time.
[**get_supported_timezone**](SystemApi.md#get_supported_timezone) | **GET** /api/v4/system/timezones | Retrieve a list of supported timezones
[**invalidate_caches**](SystemApi.md#invalidate_caches) | **POST** /api/v4/caches/invalidate | Invalidate all the caches
[**mark_notices_viewed**](SystemApi.md#mark_notices_viewed) | **PUT** /api/v4/system/notices/view | Update notices as 'viewed'
[**patch_config**](SystemApi.md#patch_config) | **PUT** /api/v4/config/patch | Patch configuration
[**post_log**](SystemApi.md#post_log) | **POST** /api/v4/logs | Add log message
[**reload_config**](SystemApi.md#reload_config) | **POST** /api/v4/config/reload | Reload configuration
[**remove_license_file**](SystemApi.md#remove_license_file) | **DELETE** /api/v4/license | Remove license file
[**request_license_renewal_link**](SystemApi.md#request_license_renewal_link) | **GET** /api/v4/license/renewal | Request the license renewal link
[**request_trial_license**](SystemApi.md#request_trial_license) | **POST** /api/v4/trial-license | Request and install a trial license for your server
[**restart_server**](SystemApi.md#restart_server) | **POST** /api/v4/restart | Restart the system after an upgrade from Team Edition to Enterprise Edition
[**set_server_busy**](SystemApi.md#set_server_busy) | **POST** /api/v4/server_busy | Set the server busy (high load) flag
[**test_email**](SystemApi.md#test_email) | **POST** /api/v4/email/test | Send a test email
[**test_s3_connection**](SystemApi.md#test_s3_connection) | **POST** /api/v4/file/s3_test | Test AWS S3 connection
[**test_site_url**](SystemApi.md#test_site_url) | **POST** /api/v4/site_url/test | Checks the validity of a Site URL
[**update_config**](SystemApi.md#update_config) | **PUT** /api/v4/config | Update configuration
[**update_marketplace_visited_by_admin**](SystemApi.md#update_marketplace_visited_by_admin) | **POST** /api/v4/plugins/marketplace/first_admin_visit | Stores that the Plugin Marketplace has been visited by at least an admin.
[**upgrade_to_enterprise**](SystemApi.md#upgrade_to_enterprise) | **POST** /api/v4/upgrade_to_enterprise | Executes an inplace upgrade from Team Edition to Enterprise Edition
[**upgrade_to_enterprise_status**](SystemApi.md#upgrade_to_enterprise_status) | **GET** /api/v4/upgrade_to_enterprise/status | Get the current status for the inplace upgrade from Team Edition to Enterprise Edition
[**upload_license_file**](SystemApi.md#upload_license_file) | **POST** /api/v4/license | Upload license file



## check_integrity

> Vec<models::IntegrityCheckResult> check_integrity()
Perform a database integrity check

Performs a database integrity check.   __Note__: This check may temporarily harm system performance.   __Minimum server version__: 5.28.0   __Local mode only__: This endpoint is only available through [local mode](https://docs.mattermost.com/administration/mmctl-cli-tool.html#local-mode). 

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::IntegrityCheckResult>**](IntegrityCheckResult.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## clear_server_busy

> models::StatusOk clear_server_busy()
Clears the server busy (high load) flag

Marks the server as not having high load which re-enables non-critical services such as search, statuses and typing notifications.  __Minimum server version__: 5.20  ##### Permissions Must have `manage_system` permission. 

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


## database_recycle

> models::StatusOk database_recycle()
Recycle database connections

Recycle database connections by closing and reconnecting all connections to master and read replica databases. ##### Permissions Must have `manage_system` permission. 

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


## generate_support_packet

> generate_support_packet(basic_server_logs, plugin_packets)
Download a zip file which contains helpful and useful information for troubleshooting your mattermost instance.

Download a zip file which contains helpful and useful information for troubleshooting your mattermost instance. __Minimum server version: 5.32__ ##### Permissions Must have any of the system console read permissions. ##### License Requires either a E10 or E20 license. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**basic_server_logs** | Option<**bool**> | Specifies whether the server should include or exclude log files. Default value is true.  __Minimum server version__: 9.8.0  |  |
**plugin_packets** | Option<**String**> | Specifies plugin identifiers whose content should be included in the Support Packet.  __Minimum server version__: 9.8.0  |  |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_analytics_old

> get_analytics_old(name, team_id)
Get analytics

Get some analytics data about the system. This endpoint uses the old format, the `/analytics` route is reserved for the new format when it gets implemented.  The returned JSON changes based on the `name` query parameter but is always key/value pairs.  __Minimum server version__: 4.0  ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | Option<**String**> | Possible values are \"standard\", \"bot_post_counts_day\", \"post_counts_day\", \"user_counts_with_posts_day\" or \"extra_counts\" |  |[default to standard]
**team_id** | Option<**String**> | The team ID to filter the data by |  |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_audits

> Vec<models::Audit> get_audits(page, per_page)
Get audits

Get a page of audits for all users on the system, selected with `page` and `per_page` query parameters. ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of audits per page. |  |[default to 60]

### Return type

[**Vec<models::Audit>**](Audit.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_client_config

> get_client_config(format)
Get client configuration

Get a subset of the server configuration needed by the client. ##### Permissions No permission required. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**format** | **String** | Must be `old`, other formats not implemented yet | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_client_license

> get_client_license(format)
Get client license

Get a subset of the server license needed by the client. ##### Permissions No permission required but having the `manage_system` permission returns more information. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**format** | **String** | Must be `old`, other formats not implemented yet | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_config

> models::Config get_config()
Get configuration

Retrieve the current server configuration ##### Permissions Must have `manage_system` permission. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::Config**](Config.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_environment_config

> models::EnvironmentConfig get_environment_config()
Get configuration made through environment variables

Retrieve a json object mirroring the server configuration where fields are set to true if the corresponding config setting is set through an environment variable. Settings that haven't been set through environment variables will be missing from the object.  __Minimum server version__: 4.10  ##### Permissions Must have `manage_system` permission. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::EnvironmentConfig**](EnvironmentConfig.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_image_by_url

> std::path::PathBuf get_image_by_url()
Get an image by url

Fetches an image via Mattermost image proxy. __Minimum server version__: 3.10 ##### Permissions Must be logged in. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: image/*, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_logs

> Vec<String> get_logs(page, logs_per_page)
Get logs

Get a page of server logs, selected with `page` and `logs_per_page` query parameters. ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**logs_per_page** | Option<**String**> | The number of logs per page. There is a maximum limit of 10000 logs per page. |  |[default to 10000]

### Return type

**Vec<String>**

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_notices

> Vec<models::Notice> get_notices(client_version, client, team_id, locale)
Get notices for logged in user in specified team

Will return appropriate product notices for current user in the team specified by teamId parameter. __Minimum server version__: 5.26 ##### Permissions Must be logged in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_version** | **String** | Version of the client (desktop/mobile/web) that issues the request | [required] |
**client** | **String** | Client type (web/mobile-ios/mobile-android/desktop) | [required] |
**team_id** | **String** | ID of the team | [required] |
**locale** | Option<**String**> | Client locale |  |

### Return type

[**Vec<models::Notice>**](Notice.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_ping

> models::SystemStatusResponse get_ping(get_server_status, device_id, use_rest_semantics)
Check system health

Check if the server is up and healthy based on the configuration setting `GoRoutineHealthThreshold`. If `GoRoutineHealthThreshold` and the number of goroutines on the server exceeds that threshold the server is considered unhealthy. If `GoRoutineHealthThreshold` is not set or the number of goroutines is below the threshold the server is considered healthy. __Minimum server version__: 3.10 If a \"device_id\" is passed in the query, it will test the Push Notification Proxy in order to discover whether the device is able to receive notifications. The response will have a \"CanReceiveNotifications\" property with one of the following values: - true: It can receive notifications - false: It cannot receive notifications - unknown: There has been an unknown error, and it is not certain whether it can    receive notifications.  __Minimum server version__: 6.5 If \"use_rest_semantics\" is set to true in the query, the endpoint will not return an error status code in the header if the request is somehow completed successfully. __Minimum server version__: 9.6 ##### Permissions None. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**get_server_status** | Option<**bool**> | Check the status of the database and file storage as well |  |
**device_id** | Option<**String**> | Check whether this device id can receive push notifications |  |
**use_rest_semantics** | Option<**bool**> | Returns 200 status code even if the server status is unhealthy. |  |

### Return type

[**models::SystemStatusResponse**](SystemStatusResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_prev_trial_license

> get_prev_trial_license()
Get last trial license used

Get the last trial license used on the sevrer __Minimum server version__: 5.36 ##### Permissions Must have `manage_systems` permissions. 

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


## get_redirect_location

> models::GetRedirectLocation200Response get_redirect_location(url)
Get redirect location

__Minimum server version__: 3.10 ##### Permissions Must be logged in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**url** | **String** | Url to check | [required] |

### Return type

[**models::GetRedirectLocation200Response**](GetRedirectLocation_200_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: image/*, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_server_busy_expires

> models::ServerBusy get_server_busy_expires()
Get server busy expiry time.

Gets the timestamp corresponding to when the server busy flag will be automatically cleared.  __Minimum server version__: 5.20  ##### Permissions Must have `manage_system` permission. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ServerBusy**](Server_Busy.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_supported_timezone

> Vec<String> get_supported_timezone()
Retrieve a list of supported timezones

__Minimum server version__: 3.10 ##### Permissions Must be logged in. 

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


## invalidate_caches

> models::StatusOk invalidate_caches()
Invalidate all the caches

Purge all the in-memory caches for the Mattermost server. This can have a temporary negative effect on performance while the caches are re-populated. ##### Permissions Must have `manage_system` permission. 

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


## mark_notices_viewed

> models::StatusOk mark_notices_viewed(request_body)
Update notices as 'viewed'

Will mark the specified notices as 'viewed' by the logged in user. __Minimum server version__: 5.26 ##### Permissions Must be logged in. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_body** | [**Vec<String>**](String.md) | Array of notice IDs | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## patch_config

> models::Config patch_config(config)
Patch configuration

Submit configuration to patch. As of server version 4.8, the `PluginSettings.EnableUploads` setting cannot be modified by this endpoint. ##### Permissions Must have `manage_system` permission. __Minimum server version__: 5.20 ##### Note The Plugins are stored as a map, and since a map may recursively go  down to any depth, individual fields of a map are not changed.  Consider using the `update config` (PUT api/v4/config) endpoint to update a plugins configurations. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config** | [**Config**](Config.md) | Mattermost configuration | [required] |

### Return type

[**models::Config**](Config.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_log

> Vec<String> post_log(post_log_request)
Add log message

Add log messages to the server logs. ##### Permissions Users with `manage_system` permission can log ERROR or DEBUG messages. Logged in users can log ERROR or DEBUG messages when `ServiceSettings.EnableDeveloper` is `true` or just DEBUG messages when `false`. Non-logged in users can log ERROR or DEBUG messages when `ServiceSettings.EnableDeveloper` is `true` and cannot log when `false`. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**post_log_request** | [**PostLogRequest**](PostLogRequest.md) |  | [required] |

### Return type

**Vec<String>**

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reload_config

> models::StatusOk reload_config()
Reload configuration

Reload the configuration file to pick up on any changes made to it. ##### Permissions Must have `manage_system` permission. 

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


## remove_license_file

> remove_license_file()
Remove license file

Remove the license file from the server. This will disable all enterprise features.  __Minimum server version__: 4.0  ##### Permissions Must have `manage_system` permission. 

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


## request_license_renewal_link

> models::LicenseRenewalLink request_license_renewal_link()
Request the license renewal link

Request the renewal link that would be used to start the license renewal process __Minimum server version__: 5.32 ##### Permissions Must have `sysconsole_write_about` permission. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::LicenseRenewalLink**](LicenseRenewalLink.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## request_trial_license

> request_trial_license(request_trial_license_request)
Request and install a trial license for your server

Request and install a trial license for your server __Minimum server version__: 5.25 ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**request_trial_license_request** | [**RequestTrialLicenseRequest**](RequestTrialLicenseRequest.md) | License request | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## restart_server

> models::StatusOk restart_server()
Restart the system after an upgrade from Team Edition to Enterprise Edition

It restarts the current running mattermost instance to execute the new Enterprise binary. __Minimum server version__: 5.27 ##### Permissions Must have `manage_system` permission. 

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


## set_server_busy

> models::StatusOk set_server_busy(seconds)
Set the server busy (high load) flag

Marks the server as currently having high load which disables non-critical services such as search, statuses and typing notifications.  __Minimum server version__: 5.20  ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**seconds** | Option<**String**> | Number of seconds until server is automatically marked as not busy. |  |[default to 3600]

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## test_email

> models::StatusOk test_email(config)
Send a test email

Send a test email to make sure you have your email settings configured correctly. Optionally provide a configuration in the request body to test. If no valid configuration is present in the request body the current server configuration will be tested. ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config** | [**Config**](Config.md) | Mattermost configuration | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## test_s3_connection

> models::StatusOk test_s3_connection(config)
Test AWS S3 connection

Send a test to validate if can connect to AWS S3. Optionally provide a configuration in the request body to test. If no valid configuration is present in the request body the current server configuration will be tested. ##### Permissions Must have `manage_system` permission. __Minimum server version__: 4.8 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config** | [**Config**](Config.md) | Mattermost configuration | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## test_site_url

> models::StatusOk test_site_url(test_site_url_request)
Checks the validity of a Site URL

Sends a Ping request to the mattermost server using the specified Site URL.  ##### Permissions Must have `manage_system` permission.  __Minimum server version__: 5.16 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**test_site_url_request** | [**TestSiteUrlRequest**](TestSiteUrlRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_config

> models::Config update_config(config)
Update configuration

Submit a new configuration for the server to use. As of server version 4.8, the `PluginSettings.EnableUploads` setting cannot be modified by this endpoint. Note that the parameters that aren't set in the configuration that you provide will be reset to default values. Therefore, if you want to change a configuration parameter and leave the other ones unchanged, you need to get the existing configuration first, change the field that you want, then put that new configuration. ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**config** | [**Config**](Config.md) | Mattermost configuration | [required] |

### Return type

[**models::Config**](Config.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_marketplace_visited_by_admin

> models::StatusOk update_marketplace_visited_by_admin(system)
Stores that the Plugin Marketplace has been visited by at least an admin.

Stores the system-level status that specifies that at least an admin has visited the in-product Plugin Marketplace. __Minimum server version: 5.33__ ##### Permissions Must have `manage_system` permissions. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**system** | [**System**](System.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upgrade_to_enterprise

> models::PushNotification upgrade_to_enterprise()
Executes an inplace upgrade from Team Edition to Enterprise Edition

It downloads the Mattermost Enterprise Edition of your current version and replace your current version with it. After the upgrade you need to restart the Mattermost server. __Minimum server version__: 5.27 ##### Permissions Must have `manage_system` permission. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::PushNotification**](PushNotification.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upgrade_to_enterprise_status

> models::UpgradeToEnterpriseStatus200Response upgrade_to_enterprise_status()
Get the current status for the inplace upgrade from Team Edition to Enterprise Edition

It returns the percentage of completion of the current upgrade or the error if there is any. __Minimum server version__: 5.27 ##### Permissions Must have `manage_system` permission. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::UpgradeToEnterpriseStatus200Response**](UpgradeToEnterpriseStatus_200_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upload_license_file

> models::StatusOk upload_license_file(license)
Upload license file

Upload a license to enable enterprise features.  __Minimum server version__: 4.0  ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**license** | **std::path::PathBuf** | The license to be uploaded | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

