# \CloudApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**confirm_customer_payment**](CloudApi.md#confirm_customer_payment) | **POST** /api/v4/cloud/payment/confirm | Completes the payment setup intent
[**create_customer_payment**](CloudApi.md#create_customer_payment) | **POST** /api/v4/cloud/payment | Create a customer setup payment intent
[**get_cloud_customer**](CloudApi.md#get_cloud_customer) | **GET** /api/v4/cloud/customer | Get cloud customer
[**get_cloud_limits**](CloudApi.md#get_cloud_limits) | **GET** /api/v4/cloud/limits | Get cloud workspace limits
[**get_cloud_products**](CloudApi.md#get_cloud_products) | **GET** /api/v4/cloud/products | Get cloud products
[**get_endpoint_for_installation_information**](CloudApi.md#get_endpoint_for_installation_information) | **GET** /api/v4/cloud/installation | GET endpoint for Installation information
[**get_invoice_for_subscription_as_pdf**](CloudApi.md#get_invoice_for_subscription_as_pdf) | **GET** /api/v4/cloud/subscription/invoices/{invoice_id}/pdf | Get cloud invoice PDF
[**get_invoices_for_subscription**](CloudApi.md#get_invoices_for_subscription) | **GET** /api/v4/cloud/subscription/invoices | Get cloud subscription invoices
[**get_subscription**](CloudApi.md#get_subscription) | **GET** /api/v4/cloud/subscription | Get cloud subscription
[**post_endpoint_for_cws_webhooks**](CloudApi.md#post_endpoint_for_cws_webhooks) | **POST** /api/v4/cloud/webhook | POST endpoint for CWS Webhooks
[**update_cloud_customer**](CloudApi.md#update_cloud_customer) | **PUT** /api/v4/cloud/customer | Update cloud customer
[**update_cloud_customer_address**](CloudApi.md#update_cloud_customer_address) | **PUT** /api/v4/cloud/customer/address | Update cloud customer address



## confirm_customer_payment

> confirm_customer_payment(stripe_setup_intent_id)
Completes the payment setup intent

Confirms the payment setup intent initiated when posting to `/cloud/payment`. ##### Permissions Must have `manage_system` permission and be licensed for Cloud. __Minimum server version__: 5.28 __Note:__ This is intended for internal use and is subject to change. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**stripe_setup_intent_id** | Option<**String**> |  |  |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_customer_payment

> models::PaymentSetupIntent create_customer_payment()
Create a customer setup payment intent

Creates a customer setup payment intent for the given Mattermost cloud installation.  ##### Permissions  Must have `manage_system` permission and be licensed for Cloud.  __Minimum server version__: 5.28 __Note:__: This is intended for internal use and is subject to change. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::PaymentSetupIntent**](PaymentSetupIntent.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_cloud_customer

> models::CloudCustomer get_cloud_customer()
Get cloud customer

Retrieves the customer information for the Mattermost Cloud customer bound to this installation. ##### Permissions Must have `manage_system` permission and be licensed for Cloud. __Minimum server version__: 5.28 __Note:__ This is intended for internal use and is subject to change. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::CloudCustomer**](CloudCustomer.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_cloud_limits

> models::ProductLimits get_cloud_limits()
Get cloud workspace limits

Retrieve any cloud workspace limits applicable to this instance. ##### Permissions Must be authenticated and be licensed for Cloud. __Minimum server version__: 7.0 __Note:__ This is intended for internal use and is subject to change. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::ProductLimits**](ProductLimits.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_cloud_products

> Vec<models::Product> get_cloud_products()
Get cloud products

Retrieve a list of all products that are offered for Mattermost Cloud. ##### Permissions Must have `manage_system` permission and be licensed for Cloud. __Minimum server version__: 5.28 __Note:__ This is intended for internal use and is subject to change. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Product>**](Product.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_endpoint_for_installation_information

> models::Installation get_endpoint_for_installation_information()
GET endpoint for Installation information

An endpoint for fetching the installation information. ##### Permissions Must have `sysconsole_read_site_ip_filters` permission and be licensed for Cloud. __Minimum server version__: 9.1 __Note:__ This is intended for internal use and is subject to change. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::Installation**](Installation.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_invoice_for_subscription_as_pdf

> get_invoice_for_subscription_as_pdf(invoice_id)
Get cloud invoice PDF

Retrieves the PDF for the invoice passed as parameter ##### Permissions Must have `manage_system` permission and be licensed for Cloud. __Minimum server version__: 5.30 __Note:__ This is intended for internal use and is subject to change. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**invoice_id** | **String** | Invoice ID | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_invoices_for_subscription

> Vec<models::Invoice> get_invoices_for_subscription()
Get cloud subscription invoices

Retrieves the invoices for the subscription bound to this installation. ##### Permissions Must have `manage_system` permission and be licensed for Cloud. __Minimum server version__: 5.30 __Note:__ This is intended for internal use and is subject to change. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Invoice>**](Invoice.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_subscription

> models::Subscription get_subscription()
Get cloud subscription

Retrieves the subscription information for the Mattermost Cloud customer bound to this installation. ##### Permissions Must have `manage_system` permission and be licensed for Cloud. __Minimum server version__: 5.28 __Note:__ This is intended for internal use and is subject to change. 

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::Subscription**](Subscription.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_endpoint_for_cws_webhooks

> post_endpoint_for_cws_webhooks()
POST endpoint for CWS Webhooks

An endpoint for processing webhooks from the Customer Portal ##### Permissions This endpoint should only be accessed by CWS, in a Mattermost Cloud instance __Minimum server version__: 5.30 __Note:__ This is intended for internal use and is subject to change. 

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


## update_cloud_customer

> models::CloudCustomer update_cloud_customer(update_cloud_customer_request)
Update cloud customer

Updates the customer information for the Mattermost Cloud customer bound to this installation. ##### Permissions Must have `manage_system` permission and be licensed for Cloud. __Minimum server version__: 5.29 __Note:__ This is intended for internal use and is subject to change. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**update_cloud_customer_request** | [**UpdateCloudCustomerRequest**](UpdateCloudCustomerRequest.md) | Customer patch including information to update | [required] |

### Return type

[**models::CloudCustomer**](CloudCustomer.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_cloud_customer_address

> models::CloudCustomer update_cloud_customer_address(address)
Update cloud customer address

Updates the company address for the Mattermost Cloud customer bound to this installation. ##### Permissions Must have `manage_system` permission and be licensed for Cloud. __Minimum server version__: 5.29 __Note:__ This is intended for internal use and is subject to change. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**address** | [**Address**](Address.md) | Company address information to update | [required] |

### Return type

[**models::CloudCustomer**](CloudCustomer.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

