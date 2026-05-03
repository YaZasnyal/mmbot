use crate::config::ToolConfig;
use crate::error::{Result, SupportBotError};
use crate::tools::{
    SupportTool, ToolCall, ToolContext, ToolExecutionOutcome, ToolKind, ToolRegistry, ToolResult,
    ToolSpec,
};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;

pub async fn register_remote_mcp_tools(
    registry: &mut ToolRegistry,
    config: &ToolConfig,
) -> Result<()> {
    for endpoint in &config.remote_mcp_endpoints {
        let client = RemoteMcpClient::new(endpoint.url.clone(), endpoint.auth_header.clone())?;
        let tools = client.list_tools().await.map_err(|error| {
            SupportBotError::Config(format!(
                "failed to load MCP tools from endpoint '{}' ({}): {}",
                endpoint.name, endpoint.url, error
            ))
        })?;

        for remote_tool in tools {
            registry.register(RemoteMcpTool::new(
                endpoint.name.clone(),
                remote_tool,
                client.clone(),
            ))?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct McpToolDescriptor {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    input_schema: serde_json::Value,
}

#[derive(Clone)]
struct RemoteMcpClient {
    base_url: String,
    http: reqwest::Client,
}

impl RemoteMcpClient {
    fn new(base_url: String, auth_header: Option<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(auth_header) = auth_header {
            let value = HeaderValue::from_str(auth_header.trim()).map_err(|error| {
                SupportBotError::Config(format!("invalid MCP auth header: {error}"))
            })?;
            headers.insert(AUTHORIZATION, value);
        }

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(SupportBotError::Http)?;

        Ok(Self { base_url, http })
    }

    async fn list_tools(&self) -> Result<Vec<McpToolDescriptor>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "list-tools",
            "method": "tools/list"
        });

        let payload = self.rpc(request).await?;
        let result = payload.get("result").unwrap_or(&payload);
        let tools_value = result
            .get("tools")
            .ok_or_else(|| SupportBotError::Tool("missing `tools` in MCP list response".into()))?
            .clone();

        serde_json::from_value(tools_value).map_err(SupportBotError::Serialization)
    }

    async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "call-tool",
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });
        let payload = self.rpc(request).await?;

        if let Some(error) = payload.get("error") {
            return Err(SupportBotError::Tool(format!(
                "MCP tool call failed for `{name}`: {error}"
            )));
        }

        Ok(payload
            .get("result")
            .cloned()
            .unwrap_or(serde_json::json!({ "ok": true })))
    }

    async fn rpc(&self, request: serde_json::Value) -> Result<serde_json::Value> {
        let response = self
            .http
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .map_err(SupportBotError::Http)?
            .error_for_status()
            .map_err(SupportBotError::Http)?;

        response
            .json::<serde_json::Value>()
            .await
            .map_err(SupportBotError::Http)
    }
}

struct RemoteMcpTool {
    exposed_name: String,
    remote_name: String,
    description: String,
    input_schema: serde_json::Value,
    client: RemoteMcpClient,
}

impl RemoteMcpTool {
    fn new(endpoint_name: String, descriptor: McpToolDescriptor, client: RemoteMcpClient) -> Self {
        Self {
            exposed_name: format!("{endpoint_name}.{}", descriptor.name),
            remote_name: descriptor.name,
            description: descriptor.description,
            input_schema: descriptor.input_schema,
            client,
        }
    }
}

#[async_trait]
impl SupportTool for RemoteMcpTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: self.exposed_name.clone(),
            description: self.description.clone(),
            input_schema: if self.input_schema.is_null() {
                serde_json::json!({ "type": "object", "additionalProperties": true })
            } else {
                self.input_schema.clone()
            },
            kind: ToolKind::ReadOnly,
        }
    }

    async fn call(&self, _ctx: ToolContext, call: ToolCall) -> Result<ToolExecutionOutcome> {
        let result = self
            .client
            .call_tool(&self.remote_name, call.arguments)
            .await?;

        Ok(ToolExecutionOutcome::ToolResult(ToolResult {
            call_id: call.id,
            content: result,
            is_error: false,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{RemoteMcpEndpoint, ToolConfig};
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn remote_mcp_tool_is_namespaced_by_endpoint() {
        let descriptor = McpToolDescriptor {
            name: "search_logs".to_string(),
            description: "Search logs".to_string(),
            input_schema: serde_json::json!({"type":"object"}),
        };
        let client = RemoteMcpClient::new("http://localhost:5555/mcp".to_string(), None).unwrap();
        let tool = RemoteMcpTool::new("logs".to_string(), descriptor, client);

        assert_eq!(tool.spec().name, "logs.search_logs");
    }

    #[tokio::test]
    async fn registers_remote_tools_from_mcp_list() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/mcp"))
            .and(body_partial_json(serde_json::json!({
                "method": "tools/list"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "list-tools",
                "result": {
                    "tools": [{
                        "name": "search_logs",
                        "description": "Search logs",
                        "inputSchema": {
                            "type": "object",
                            "properties": { "query": { "type": "string" } },
                            "required": ["query"]
                        }
                    }]
                }
            })))
            .mount(&server)
            .await;

        let mut registry = ToolRegistry::new();
        let config = ToolConfig {
            remote_mcp_endpoints: vec![RemoteMcpEndpoint {
                name: "logs".to_string(),
                url: format!("{}/mcp", server.uri()),
                auth_header: None,
            }],
        };

        register_remote_mcp_tools(&mut registry, &config)
            .await
            .unwrap();

        let specs = registry.specs();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "logs.search_logs");
        assert_eq!(specs[0].kind, ToolKind::ReadOnly);
    }

    #[tokio::test]
    async fn calls_remote_tool_through_registry() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/mcp"))
            .and(body_partial_json(serde_json::json!({
                "method": "tools/list"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "list-tools",
                "result": {
                    "tools": [{
                        "name": "search_logs",
                        "description": "Search logs",
                        "inputSchema": { "type": "object" }
                    }]
                }
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/mcp"))
            .and(header("authorization", "Bearer test-token"))
            .and(body_partial_json(serde_json::json!({
                "method": "tools/call",
                "params": {
                    "name": "search_logs",
                    "arguments": { "query": "timeout" }
                }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": "call-tool",
                "result": {
                    "content": [{ "type": "text", "text": "found 3 matches" }]
                }
            })))
            .mount(&server)
            .await;

        let mut registry = ToolRegistry::new();
        let config = ToolConfig {
            remote_mcp_endpoints: vec![RemoteMcpEndpoint {
                name: "logs".to_string(),
                url: format!("{}/mcp", server.uri()),
                auth_header: Some("Bearer test-token".to_string()),
            }],
        };
        register_remote_mcp_tools(&mut registry, &config)
            .await
            .unwrap();

        let outcome = registry
            .call(
                ToolContext::without_thread(),
                ToolCall {
                    id: "call-1".to_string(),
                    name: "logs.search_logs".to_string(),
                    arguments: serde_json::json!({ "query": "timeout" }),
                },
            )
            .await
            .unwrap();

        match outcome {
            ToolExecutionOutcome::ToolResult(result) => {
                assert_eq!(result.call_id, "call-1");
                assert!(!result.is_error);
                assert_eq!(result.content["content"][0]["text"], "found 3 matches");
            }
            ToolExecutionOutcome::Action(_) => panic!("expected ToolResult"),
        }
    }
}
