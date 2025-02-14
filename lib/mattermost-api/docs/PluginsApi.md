# \PluginsApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**disable_plugin**](PluginsApi.md#disable_plugin) | **POST** /api/v4/plugins/{plugin_id}/disable | Disable plugin
[**enable_plugin**](PluginsApi.md#enable_plugin) | **POST** /api/v4/plugins/{plugin_id}/enable | Enable plugin
[**get_marketplace_plugins**](PluginsApi.md#get_marketplace_plugins) | **GET** /api/v4/plugins/marketplace | Gets all the marketplace plugins
[**get_marketplace_visited_by_admin**](PluginsApi.md#get_marketplace_visited_by_admin) | **GET** /api/v4/plugins/marketplace/first_admin_visit | Get if the Plugin Marketplace has been visited by at least an admin.
[**get_plugin_statuses**](PluginsApi.md#get_plugin_statuses) | **GET** /api/v4/plugins/statuses | Get plugins status
[**get_plugins**](PluginsApi.md#get_plugins) | **GET** /api/v4/plugins | Get plugins
[**get_webapp_plugins**](PluginsApi.md#get_webapp_plugins) | **GET** /api/v4/plugins/webapp | Get webapp plugins
[**install_marketplace_plugin**](PluginsApi.md#install_marketplace_plugin) | **POST** /api/v4/plugins/marketplace | Installs a marketplace plugin
[**install_plugin_from_url**](PluginsApi.md#install_plugin_from_url) | **POST** /api/v4/plugins/install_from_url | Install plugin from url
[**remove_plugin**](PluginsApi.md#remove_plugin) | **DELETE** /api/v4/plugins/{plugin_id} | Remove plugin
[**upload_plugin**](PluginsApi.md#upload_plugin) | **POST** /api/v4/plugins | Upload plugin



## disable_plugin

> models::StatusOk disable_plugin(plugin_id)
Disable plugin

Disable a previously enabled plugin. Plugins must be enabled in the server's config settings.  ##### Permissions Must have `manage_system` permission.  __Minimum server version__: 4.4 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**plugin_id** | **String** | Id of the plugin to be disabled | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## enable_plugin

> models::StatusOk enable_plugin(plugin_id)
Enable plugin

Enable a previously uploaded plugin. Plugins must be enabled in the server's config settings.  ##### Permissions Must have `manage_system` permission.  __Minimum server version__: 4.4 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**plugin_id** | **String** | Id of the plugin to be enabled | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_marketplace_plugins

> Vec<models::MarketplacePlugin> get_marketplace_plugins(page, per_page, filter, server_version, local_only)
Gets all the marketplace plugins

Gets all plugins from the marketplace server, merging data from locally installed plugins as well as prepackaged plugins shipped with the server.  ##### Permissions Must have `manage_system` permission.  __Minimum server version__: 5.16 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | Page number to be fetched. (not yet implemented) |  |
**per_page** | Option<**i32**> | Number of item per page. (not yet implemented) |  |
**filter** | Option<**String**> | Set to filter plugins by ID, name, or description. |  |
**server_version** | Option<**String**> | Set to filter minimum plugin server version. (not yet implemented) |  |
**local_only** | Option<**bool**> | Set true to only retrieve local plugins. |  |

### Return type

[**Vec<models::MarketplacePlugin>**](MarketplacePlugin.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_marketplace_visited_by_admin

> models::System get_marketplace_visited_by_admin()
Get if the Plugin Marketplace has been visited by at least an admin.

Retrieves the status that specifies that at least one System Admin has visited the in-product Plugin Marketplace. __Minimum server version: 5.33__ ##### Permissions Must have `manage_system` permissions. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::System**](System.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_plugin_statuses

> Vec<models::PluginStatus> get_plugin_statuses()
Get plugins status

Returns the status for plugins installed anywhere in the cluster  ##### Permissions No permissions required.  __Minimum server version__: 4.4 

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::PluginStatus>**](PluginStatus.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_plugins

> models::GetPlugins200Response get_plugins()
Get plugins

Get a list of inactive and a list of active plugin manifests. Plugins must be enabled in the server's config settings.  ##### Permissions Must have `manage_system` permission.  __Minimum server version__: 4.4 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetPlugins200Response**](GetPlugins_200_response.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_webapp_plugins

> Vec<models::PluginManifestWebapp> get_webapp_plugins()
Get webapp plugins

Get a list of web app plugins installed and activated on the server.  ##### Permissions No permissions required.  __Minimum server version__: 4.4 

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::PluginManifestWebapp>**](PluginManifestWebapp.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## install_marketplace_plugin

> models::PluginManifest install_marketplace_plugin(install_marketplace_plugin_request)
Installs a marketplace plugin

Installs a plugin listed in the marketplace server.  ##### Permissions Must have `manage_system` permission.  __Minimum server version__: 5.16 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**install_marketplace_plugin_request** | [**InstallMarketplacePluginRequest**](InstallMarketplacePluginRequest.md) | The metadata identifying the plugin to install. | [required] |

### Return type

[**models::PluginManifest**](PluginManifest.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## install_plugin_from_url

> models::StatusOk install_plugin_from_url(plugin_download_url, force)
Install plugin from url

Supply a URL to a plugin compressed in a .tar.gz file. Plugins must be enabled in the server's config settings.  ##### Permissions Must have `manage_system` permission.  __Minimum server version__: 5.14 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**plugin_download_url** | **String** | URL used to download the plugin | [required] |
**force** | Option<**String**> | Set to 'true' to overwrite a previously installed plugin with the same ID, if any |  |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## remove_plugin

> models::StatusOk remove_plugin(plugin_id)
Remove plugin

Remove the plugin with the provided ID from the server. All plugin files are deleted. Plugins must be enabled in the server's config settings.  ##### Permissions Must have `manage_system` permission.  __Minimum server version__: 4.4 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**plugin_id** | **String** | Id of the plugin to be removed | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upload_plugin

> models::StatusOk upload_plugin(plugin, force)
Upload plugin

Upload a plugin that is contained within a compressed .tar.gz file. Plugins and plugin uploads must be enabled in the server's config settings.  ##### Permissions Must have `manage_system` permission.  __Minimum server version__: 4.4 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**plugin** | **std::path::PathBuf** | The plugin image to be uploaded | [required] |
**force** | Option<**String**> | Set to 'true' to overwrite a previously installed plugin with the same ID, if any |  |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

