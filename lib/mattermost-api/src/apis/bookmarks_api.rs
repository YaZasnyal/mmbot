/*
 * Mattermost API Reference
 *
 * There is also a work-in-progress [Postman API reference](https://documenter.getpostman.com/view/4508214/RW8FERUn).
 *
 * The version of the OpenAPI document: 4.0.0
 * Contact: feedback@mattermost.com
 * Generated by: https://openapi-generator.tech
 */

use super::{configuration, Error};
use crate::{apis::ResponseContent, models};
use reqwest;
use serde::{Deserialize, Serialize};

/// struct for typed errors of method [`create_channel_bookmark`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateChannelBookmarkError {
    Status400(models::AppError),
    Status401(models::AppError),
    Status403(models::AppError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`delete_channel_bookmark`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeleteChannelBookmarkError {
    Status400(models::AppError),
    Status401(models::AppError),
    Status403(models::AppError),
    Status404(models::AppError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`list_channel_bookmarks_for_channel`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListChannelBookmarksForChannelError {
    Status400(models::AppError),
    Status401(models::AppError),
    Status403(models::AppError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`update_channel_bookmark`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UpdateChannelBookmarkError {
    Status400(models::AppError),
    Status401(models::AppError),
    Status403(models::AppError),
    Status404(models::AppError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`update_channel_bookmark_sort_order`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UpdateChannelBookmarkSortOrderError {
    Status400(models::AppError),
    Status401(models::AppError),
    Status403(models::AppError),
    UnknownValue(serde_json::Value),
}

/// Creates a new channel bookmark for this channel.  __Minimum server version__: 9.5  ##### Permissions Must have the `add_bookmark_public_channel` or `add_bookmark_private_channel` depending on the channel type. If the channel is a DM or GM, must be a non-guest member.
pub async fn create_channel_bookmark(
    configuration: &configuration::Configuration,
    channel_id: &str,
    create_channel_bookmark_request: models::CreateChannelBookmarkRequest,
) -> Result<models::ChannelBookmarkWithFileInfo, Error<CreateChannelBookmarkError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/channels/{channel_id}/bookmarks",
        local_var_configuration.base_path,
        channel_id = crate::apis::urlencode(channel_id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&create_channel_bookmark_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<CreateChannelBookmarkError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Archives a channel bookmark. This will set the `deleteAt` to the current timestamp in the database.  __Minimum server version__: 9.5  ##### Permissions Must have the `delete_bookmark_public_channel` or `delete_bookmark_private_channel` depending on the channel type. If the channel is a DM or GM, must be a non-guest member.
pub async fn delete_channel_bookmark(
    configuration: &configuration::Configuration,
    channel_id: &str,
    bookmark_id: &str,
) -> Result<models::ChannelBookmarkWithFileInfo, Error<DeleteChannelBookmarkError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/channels/{channel_id}/bookmarks/{bookmark_id}",
        local_var_configuration.base_path,
        channel_id = crate::apis::urlencode(channel_id),
        bookmark_id = crate::apis::urlencode(bookmark_id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::DELETE, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<DeleteChannelBookmarkError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// __Minimum server version__: 9.5
pub async fn list_channel_bookmarks_for_channel(
    configuration: &configuration::Configuration,
    channel_id: &str,
    bookmarks_since: Option<f64>,
) -> Result<Vec<models::ChannelBookmarkWithFileInfo>, Error<ListChannelBookmarksForChannelError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/channels/{channel_id}/bookmarks",
        local_var_configuration.base_path,
        channel_id = crate::apis::urlencode(channel_id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = bookmarks_since {
        local_var_req_builder =
            local_var_req_builder.query(&[("bookmarks_since", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<ListChannelBookmarksForChannelError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Partially update a channel bookmark by providing only the fields you want to update. Ommited fields will not be updated. The fields that can be updated are defined in the request body, all other provided fields will be ignored.  __Minimum server version__: 9.5  ##### Permissions Must have the `edit_bookmark_public_channel` or `edit_bookmark_private_channel` depending on the channel type. If the channel is a DM or GM, must be a non-guest member.
pub async fn update_channel_bookmark(
    configuration: &configuration::Configuration,
    channel_id: &str,
    bookmark_id: &str,
    update_channel_bookmark_request: models::UpdateChannelBookmarkRequest,
) -> Result<models::UpdateChannelBookmarkResponse, Error<UpdateChannelBookmarkError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/channels/{channel_id}/bookmarks/{bookmark_id}",
        local_var_configuration.base_path,
        channel_id = crate::apis::urlencode(channel_id),
        bookmark_id = crate::apis::urlencode(bookmark_id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::PATCH, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&update_channel_bookmark_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<UpdateChannelBookmarkError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Updates the order of a channel bookmark, setting its new order from the parameters and updating the rest of the bookmarks of the channel to accomodate for this change.  __Minimum server version__: 9.5  ##### Permissions Must have the `order_bookmark_public_channel` or `order_bookmark_private_channel` depending on the channel type. If the channel is a DM or GM, must be a non-guest member.
pub async fn update_channel_bookmark_sort_order(
    configuration: &configuration::Configuration,
    channel_id: &str,
    bookmark_id: &str,
    body: Option<f64>,
) -> Result<Vec<models::ChannelBookmarkWithFileInfo>, Error<UpdateChannelBookmarkSortOrderError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/channels/{channel_id}/bookmarks/{bookmark_id}/sort_order",
        local_var_configuration.base_path,
        channel_id = crate::apis::urlencode(channel_id),
        bookmark_id = crate::apis::urlencode(bookmark_id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&body);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<UpdateChannelBookmarkSortOrderError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}
