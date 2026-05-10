use super::*;

fn workflow_tool(name: &str) -> WorkflowTool {
    let spec = WORKFLOW_TOOL_SPECS
        .iter()
        .find(|spec| spec.name == name)
        .expect("workflow tool spec should exist");
    WorkflowTool::new(spec)
}

#[test]
fn registers_default_workflow_tools() {
    let mut registry = ToolRegistry::new();
    register_default_workflow_tools(&mut registry).unwrap();

    let names = registry
        .specs()
        .into_iter()
        .map(|spec| spec.name)
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec!["finish_request", "notify_engineer", "send_user_message"]
    );
}

#[tokio::test]
async fn workflow_tool_returns_support_action() {
    let tool = workflow_tool("send_user_message");
    let outcome = tool
        .call(
            ToolContext::without_thread(),
            ToolCall {
                id: "call-1".to_string(),
                name: "send_user_message".to_string(),
                arguments: serde_json::json!({ "message": "hello" }),
            },
        )
        .await
        .unwrap();

    assert_eq!(
        outcome,
        ToolExecutionOutcome::Action(SupportAction::SendUserMessage {
            message: "hello".to_string()
        })
    );
}

#[tokio::test]
async fn workflow_tool_rejects_empty_required_string() {
    let tool = workflow_tool("send_user_message");
    let error = tool
        .call(
            ToolContext::without_thread(),
            ToolCall {
                id: "call-1".to_string(),
                name: "send_user_message".to_string(),
                arguments: serde_json::json!({ "message": "   " }),
            },
        )
        .await
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("missing required string argument: message"));
}

#[tokio::test]
async fn workflow_tool_rejects_wrong_argument_type() {
    let tool = workflow_tool("notify_engineer");
    let error = tool
        .call(
            ToolContext::without_thread(),
            ToolCall {
                id: "call-1".to_string(),
                name: "notify_engineer".to_string(),
                arguments: serde_json::json!({ "message": 42 }),
            },
        )
        .await
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("invalid workflow tool arguments"));
}

#[tokio::test]
async fn workflow_tool_rejects_extra_fields() {
    let tool = workflow_tool("finish_request");
    let error = tool
        .call(
            ToolContext::without_thread(),
            ToolCall {
                id: "call-1".to_string(),
                name: "finish_request".to_string(),
                arguments: serde_json::json!({
                    "summary": "done",
                    "unexpected": "value"
                }),
            },
        )
        .await
        .unwrap_err();

    assert!(error.to_string().contains("unknown field"));
}
