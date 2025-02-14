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

/// struct for typed errors of method [`get_all_shared_channels`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetAllSharedChannelsError {
    Status400(models::AppError),
    Status401(models::AppError),
    Status403(models::AppError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_remote_cluster_info`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetRemoteClusterInfoError {
    Status400(models::AppError),
    Status401(models::AppError),
    Status403(models::AppError),
    Status404(models::AppError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_shared_channel_remotes_by_remote_cluster`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetSharedChannelRemotesByRemoteClusterError {
    Status401(models::AppError),
    Status403(models::AppError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`invite_remote_cluster_to_channel`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InviteRemoteClusterToChannelError {
    Status401(models::AppError),
    Status403(models::AppError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`uninvite_remote_cluster_to_channel`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UninviteRemoteClusterToChannelError {
    Status401(models::AppError),
    Status403(models::AppError),
    UnknownValue(serde_json::Value),
}

/// Get all shared channels for a team.  __Minimum server version__: 5.50  ##### Permissions Must be authenticated.
pub async fn get_all_shared_channels(
    configuration: &configuration::Configuration,
    team_id: &str,
    page: Option<i32>,
    per_page: Option<i32>,
) -> Result<Vec<models::SharedChannel>, Error<GetAllSharedChannelsError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/sharedchannels/{team_id}",
        local_var_configuration.base_path,
        team_id = crate::apis::urlencode(team_id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = page {
        local_var_req_builder =
            local_var_req_builder.query(&[("page", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = per_page {
        local_var_req_builder =
            local_var_req_builder.query(&[("per_page", &local_var_str.to_string())]);
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
        let local_var_entity: Option<GetAllSharedChannelsError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Get remote cluster info based on remoteId.  __Minimum server version__: 5.50  ##### Permissions Must be authenticated and user must belong to at least one channel shared with the remote cluster.
pub async fn get_remote_cluster_info(
    configuration: &configuration::Configuration,
    remote_id: &str,
) -> Result<models::RemoteClusterInfo, Error<GetRemoteClusterInfoError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/sharedchannels/remote_info/{remote_id}",
        local_var_configuration.base_path,
        remote_id = crate::apis::urlencode(remote_id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

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
        let local_var_entity: Option<GetRemoteClusterInfoError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Get a list of the channels shared with a given remote cluster and their status.  ##### Permissions `manage_secure_connections`
pub async fn get_shared_channel_remotes_by_remote_cluster(
    configuration: &configuration::Configuration,
    remote_id: &str,
    include_unconfirmed: Option<bool>,
    exclude_confirmed: Option<bool>,
    exclude_home: Option<bool>,
    exclude_remote: Option<bool>,
    include_deleted: Option<bool>,
    page: Option<i32>,
    per_page: Option<i32>,
) -> Result<Vec<models::SharedChannelRemote>, Error<GetSharedChannelRemotesByRemoteClusterError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/remotecluster/{remote_id}/sharedchannelremotes",
        local_var_configuration.base_path,
        remote_id = crate::apis::urlencode(remote_id)
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = include_unconfirmed {
        local_var_req_builder =
            local_var_req_builder.query(&[("include_unconfirmed", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = exclude_confirmed {
        local_var_req_builder =
            local_var_req_builder.query(&[("exclude_confirmed", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = exclude_home {
        local_var_req_builder =
            local_var_req_builder.query(&[("exclude_home", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = exclude_remote {
        local_var_req_builder =
            local_var_req_builder.query(&[("exclude_remote", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = include_deleted {
        local_var_req_builder =
            local_var_req_builder.query(&[("include_deleted", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = page {
        local_var_req_builder =
            local_var_req_builder.query(&[("page", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = per_page {
        local_var_req_builder =
            local_var_req_builder.query(&[("per_page", &local_var_str.to_string())]);
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
        let local_var_entity: Option<GetSharedChannelRemotesByRemoteClusterError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Invites a remote cluster to a channel, sharing the channel if needed. If the remote cluster was already invited to the channel, calling this endpoint will have no effect.  ##### Permissions `manage_shared_channels`
pub async fn invite_remote_cluster_to_channel(
    configuration: &configuration::Configuration,
    remote_id: &str,
    channel_id: &str,
) -> Result<models::StatusOk, Error<InviteRemoteClusterToChannelError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/remotecluster/{remote_id}/channels/{channel_id}/invite",
        local_var_configuration.base_path,
        remote_id = crate::apis::urlencode(remote_id),
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

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<InviteRemoteClusterToChannelError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Stops sharing a channel with a remote cluster. If the channel was not shared with the remote, calling this endpoint will have no effect.  ##### Permissions `manage_shared_channels`
pub async fn uninvite_remote_cluster_to_channel(
    configuration: &configuration::Configuration,
    remote_id: &str,
    channel_id: &str,
) -> Result<models::StatusOk, Error<UninviteRemoteClusterToChannelError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/remotecluster/{remote_id}/channels/{channel_id}/uninvite",
        local_var_configuration.base_path,
        remote_id = crate::apis::urlencode(remote_id),
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

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<UninviteRemoteClusterToChannelError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}
