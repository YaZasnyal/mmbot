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
    let client = RemoteMcpClient::new(
        "http://localhost:5555/mcp".to_string(),
        None,
        Duration::from_secs(1),
    )
    .unwrap();
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
            timeout: Duration::from_secs(1),
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
async fn remote_mcp_client_applies_timeout() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(body_partial_json(serde_json::json!({
            "method": "tools/list"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(100))
                .set_body_json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "list-tools",
                    "result": { "tools": [] }
                })),
        )
        .mount(&server)
        .await;

    let client = RemoteMcpClient::new(
        format!("{}/mcp", server.uri()),
        None,
        Duration::from_millis(1),
    )
    .unwrap();

    assert!(client.list_tools().await.is_err());
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
            timeout: Duration::from_secs(1),
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
