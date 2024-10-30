# \LdapApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_user_to_group_syncables**](LdapApi.md#add_user_to_group_syncables) | **POST** /api/v4/ldap/users/{user_id}/group_sync_memberships | Create memberships for LDAP configured channels and teams for this user
[**delete_ldap_private_certificate**](LdapApi.md#delete_ldap_private_certificate) | **DELETE** /api/v4/ldap/certificate/private | Remove private key
[**delete_ldap_public_certificate**](LdapApi.md#delete_ldap_public_certificate) | **DELETE** /api/v4/ldap/certificate/public | Remove public certificate
[**migrate_auth_to_ldap_2**](LdapApi.md#migrate_auth_to_ldap_2) | **POST** /api/v4/users/migrate_auth/ldap | Migrate user accounts authentication type to LDAP.
[**migrate_id_ldap**](LdapApi.md#migrate_id_ldap) | **POST** /api/v4/ldap/migrateid | Migrate Id LDAP
[**sync_ldap**](LdapApi.md#sync_ldap) | **POST** /api/v4/ldap/sync | Sync with LDAP
[**test_ldap**](LdapApi.md#test_ldap) | **POST** /api/v4/ldap/test | Test LDAP configuration
[**upload_ldap_private_certificate**](LdapApi.md#upload_ldap_private_certificate) | **POST** /api/v4/ldap/certificate/private | Upload private key
[**upload_ldap_public_certificate**](LdapApi.md#upload_ldap_public_certificate) | **POST** /api/v4/ldap/certificate/public | Upload public certificate



## add_user_to_group_syncables

> models::StatusOk add_user_to_group_syncables(user_id)
Create memberships for LDAP configured channels and teams for this user

Add the user to each channel and team configured for each LDAP group of whicht the user is a member. ##### Permissions Must have `sysconsole_write_user_management_groups` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** | User Id | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_ldap_private_certificate

> models::StatusOk delete_ldap_private_certificate()
Remove private key

Delete the current private key being used with your TLS verification. ##### Permissions Must have `manage_system` permission. 

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


## delete_ldap_public_certificate

> models::StatusOk delete_ldap_public_certificate()
Remove public certificate

Delete the current public certificate being used for TLS verification. ##### Permissions Must have `manage_system` permission. 

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


## migrate_auth_to_ldap_2

> migrate_auth_to_ldap_2(migrate_auth_to_ldap_request)
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


## migrate_id_ldap

> models::StatusOk migrate_id_ldap(migrate_id_ldap_request)
Migrate Id LDAP

Migrate LDAP IdAttribute to new value. ##### Permissions Must have `manage_system` permission. __Minimum server version__: 5.26 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**migrate_id_ldap_request** | [**MigrateIdLdapRequest**](MigrateIdLdapRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## sync_ldap

> models::StatusOk sync_ldap()
Sync with LDAP

Synchronize any user attribute changes in the configured AD/LDAP server with Mattermost. ##### Permissions Must have `manage_system` permission. 

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


## test_ldap

> models::StatusOk test_ldap()
Test LDAP configuration

Test the current AD/LDAP configuration to see if the AD/LDAP server can be contacted successfully. ##### Permissions Must have `manage_system` permission. 

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


## upload_ldap_private_certificate

> models::StatusOk upload_ldap_private_certificate(certificate)
Upload private key

Upload the private key to be used for TLS verification. The server will pick a hard-coded filename for the PrivateKeyFile setting in your `config.json`. ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**certificate** | **std::path::PathBuf** | The private key file | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upload_ldap_public_certificate

> models::StatusOk upload_ldap_public_certificate(certificate)
Upload public certificate

Upload the public certificate to be used for TLS verification. The server will pick a hard-coded filename for the PublicCertificateFile setting in your `config.json`. ##### Permissions Must have `manage_system` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**certificate** | **std::path::PathBuf** | The public certificate file | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

