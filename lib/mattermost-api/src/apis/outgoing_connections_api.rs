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

/// struct for typed errors of method [`create_outgoing_o_auth_connection_0`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateOutgoingOAuthConnection0Error {
    Status400(models::AppError),
    Status401(models::AppError),
    Status500(models::AppError),
    Status501(models::AppError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`list_outgoing_o_auth_connections_0`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListOutgoingOAuthConnections0Error {
    Status401(models::AppError),
    Status500(models::AppError),
    Status501(models::AppError),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`validate_outgoing_o_auth_connection_0`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValidateOutgoingOAuthConnection0Error {
    Status400(models::AppError),
    Status401(models::AppError),
    Status404(models::AppError),
    Status500(models::AppError),
    Status501(models::AppError),
    Status502(models::AppError),
    UnknownValue(serde_json::Value),
}

/// Create an outgoing OAuth connection. __Minimum server version__: 9.6
pub async fn create_outgoing_o_auth_connection_0(
    configuration: &configuration::Configuration,
    team_id: &str,
    outgoing_o_auth_connection_post_item: Option<models::OutgoingOAuthConnectionPostItem>,
) -> Result<models::OutgoingOAuthConnectionGetItem, Error<CreateOutgoingOAuthConnection0Error>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/oauth/outgoing_connections",
        local_var_configuration.base_path
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("team_id", &team_id.to_string())]);
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&outgoing_o_auth_connection_post_item);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<CreateOutgoingOAuthConnection0Error> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// List all outgoing OAuth connections. __Minimum server version__: 9.6
pub async fn list_outgoing_o_auth_connections_0(
    configuration: &configuration::Configuration,
    team_id: &str,
) -> Result<Vec<models::OutgoingOAuthConnectionGetItem>, Error<ListOutgoingOAuthConnections0Error>>
{
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/oauth/outgoing_connections",
        local_var_configuration.base_path
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("team_id", &team_id.to_string())]);
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
        let local_var_entity: Option<ListOutgoingOAuthConnections0Error> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Validate an outgoing OAuth connection. If an id is provided in the payload, and no client secret is provided, then the stored client secret is implicitly used for the validation. __Minimum server version__: 9.6
pub async fn validate_outgoing_o_auth_connection_0(
    configuration: &configuration::Configuration,
    team_id: &str,
    outgoing_o_auth_connection_post_item: Option<models::OutgoingOAuthConnectionPostItem>,
) -> Result<(), Error<ValidateOutgoingOAuthConnection0Error>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/v4/oauth/outgoing_connections/validate",
        local_var_configuration.base_path
    );
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    local_var_req_builder = local_var_req_builder.query(&[("team_id", &team_id.to_string())]);
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&outgoing_o_auth_connection_post_item);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<ValidateOutgoingOAuthConnection0Error> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}
