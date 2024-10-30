# \JobsApi

All URIs are relative to *http://your-mattermost-url.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**cancel_job**](JobsApi.md#cancel_job) | **POST** /api/v4/jobs/{job_id}/cancel | Cancel a job.
[**create_job**](JobsApi.md#create_job) | **POST** /api/v4/jobs | Create a new job.
[**download_job**](JobsApi.md#download_job) | **GET** /api/v4/jobs/{job_id}/download | Download the results of a job.
[**get_job**](JobsApi.md#get_job) | **GET** /api/v4/jobs/{job_id} | Get a job.
[**get_jobs**](JobsApi.md#get_jobs) | **GET** /api/v4/jobs | Get the jobs.
[**get_jobs_by_type**](JobsApi.md#get_jobs_by_type) | **GET** /api/v4/jobs/type/{type} | Get the jobs of the given type.
[**update_job_status**](JobsApi.md#update_job_status) | **PATCH** /api/v4/jobs/{job_id}/status | Update the status of a job



## cancel_job

> models::StatusOk cancel_job(job_id)
Cancel a job.

Cancel a job. __Minimum server version: 4.1__ ##### Permissions Must have `manage_jobs` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** | Job GUID | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_job

> models::Job create_job(create_job_request)
Create a new job.

Create a new job. __Minimum server version: 4.1__ ##### Permissions Must have `manage_jobs` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_job_request** | [**CreateJobRequest**](CreateJobRequest.md) | Job object to be created | [required] |

### Return type

[**models::Job**](Job.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## download_job

> download_job(job_id)
Download the results of a job.

Download the result of a single job. __Minimum server version: 5.28__ ##### Permissions Must have `manage_jobs` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** | Job GUID | [required] |

### Return type

 (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_job

> models::Job get_job(job_id)
Get a job.

Gets a single job. __Minimum server version: 4.1__ ##### Permissions Must have `manage_jobs` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** | Job GUID | [required] |

### Return type

[**models::Job**](Job.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_jobs

> Vec<models::Job> get_jobs(page, per_page, job_type, status)
Get the jobs.

Get a page of jobs. Use the query parameters to modify the behaviour of this endpoint. __Minimum server version: 4.1__ ##### Permissions Must have `manage_jobs` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of jobs per page. |  |[default to 5]
**job_type** | Option<**String**> | The type of jobs to fetch. |  |
**status** | Option<**String**> | The status of jobs to fetch. |  |

### Return type

[**Vec<models::Job>**](Job.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_jobs_by_type

> Vec<models::Job> get_jobs_by_type(r#type, page, per_page)
Get the jobs of the given type.

Get a page of jobs of the given type. Use the query parameters to modify the behaviour of this endpoint. __Minimum server version: 4.1__ ##### Permissions Must have `manage_jobs` permission. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**r#type** | **String** | Job type | [required] |
**page** | Option<**i32**> | The page to select. |  |[default to 0]
**per_page** | Option<**i32**> | The number of jobs per page. |  |[default to 60]

### Return type

[**Vec<models::Job>**](Job.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_job_status

> models::StatusOk update_job_status(job_id, update_job_status_request)
Update the status of a job

Update the status of a job. Valid status updates: - 'in_progress' -> 'pending' - 'in_progress' | 'pending' -> 'cancel_requested' - 'cancel_requested' -> 'canceled' Add force to the body of the PATCH request to bypass the given rules, the only statuses you can go to are: pending, cancel_requested and canceled. This can have unexpected consequences and should be used with caution. 

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**job_id** | **String** | Job GUID | [required] |
**update_job_status_request** | [**UpdateJobStatusRequest**](UpdateJobStatusRequest.md) |  | [required] |

### Return type

[**models::StatusOk**](StatusOK.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

