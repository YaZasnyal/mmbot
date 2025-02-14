# \AuthenticationApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**migrate_auth_to_ldap_1**](AuthenticationApi.md#migrate_auth_to_ldap_1) | **POST** /api/v4/users/migrate_auth/ldap | Migrate user accounts authentication type to LDAP.
[**migrate_auth_to_saml_1**](AuthenticationApi.md#migrate_auth_to_saml_1) | **POST** /api/v4/users/migrate_auth/saml | Migrate user accounts authentication type to SAML.



## migrate_auth_to_ldap_1

> migrate_auth_to_ldap_1(migrate_auth_to_ldap_request)
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


## migrate_auth_to_saml_1

> migrate_auth_to_saml_1(migrate_auth_to_saml_request)
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

