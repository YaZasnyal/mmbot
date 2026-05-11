use super::*;
use crate::tools::{SupportTool, ToolCall, ToolContext, ToolExecutionOutcome};

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "support-bot-instructions-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn write_doc(root: &Path, relative: &str, title: &str, body: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, format!("---\ntitle: {title}\n---\n{body}")).unwrap();
}

#[test]
fn derives_ids_from_paths() {
    assert_eq!(
        id_from_relative_path(Path::new("index.md")).unwrap(),
        "/index"
    );
    assert_eq!(
        id_from_relative_path(Path::new("runbooks/403/no-public-read-allowed.md")).unwrap(),
        "/runbooks/403/no-public-read-allowed"
    );
}

#[test]
fn rejects_traversal_paths() {
    let error = id_from_relative_path(Path::new("../secret.md")).unwrap_err();

    assert!(error.to_string().contains("invalid instruction path"));
}

#[test]
fn loads_document_without_frontmatter_body() {
    let root = temp_dir("load");
    write_doc(
        &root,
        "index.md",
        "Index",
        "\n# Index\n[/diagnostics/index]",
    );
    write_doc(
        &root,
        "diagnostics/index.md",
        "Diagnostics",
        "\n# Diagnostics",
    );

    let repository = InstructionRepository::new(&root).unwrap();
    let loaded = repository.get_document(None).unwrap().unwrap();

    assert_eq!(loaded.document.id, "/index");
    assert_eq!(loaded.document.title, "Index");
    assert_eq!(loaded.content, "\n# Index\n[/diagnostics/index]");

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn repository_skips_invalid_documents_without_failing() {
    let root = temp_dir("invalid-skip");
    write_doc(&root, "index.md", "Index", "\n# Index");
    std::fs::create_dir_all(root.join("broken")).unwrap();
    std::fs::write(
        root.join("broken/no-title.md"),
        "---\nsummary: Missing\n---\n# Broken",
    )
    .unwrap();

    let repository = InstructionRepository::new(&root).unwrap();

    assert_eq!(repository.documents().count(), 1);
    assert!(repository
        .get_document(Some("/broken/no-title"))
        .unwrap()
        .is_none());

    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn instruction_tool_loads_index_by_default() {
    let root = temp_dir("tool-default");
    write_doc(&root, "index.md", "Index", "\n# Index");

    let repository = InstructionRepository::new(&root).unwrap();
    let outcome = repository
        .call(
            ToolContext::without_thread(),
            ToolCall {
                id: "call_1".to_string(),
                name: "instructions".to_string(),
                arguments: serde_json::json!({}),
            },
        )
        .await
        .unwrap();

    let ToolExecutionOutcome::ToolResult(result) = outcome else {
        panic!("expected tool result");
    };

    assert!(!result.is_error);
    assert_eq!(result.content["document"]["id"], "/index");

    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn instruction_tool_loads_explicit_document() {
    let root = temp_dir("tool-explicit");
    write_doc(&root, "index.md", "Index", "\n[/diagnostics/timeout]");
    write_doc(&root, "diagnostics/timeout.md", "Timeouts", "\n# Timeout");

    let repository = InstructionRepository::new(&root).unwrap();
    let outcome = repository
        .call(
            ToolContext::without_thread(),
            ToolCall {
                id: "call_2".to_string(),
                name: "instructions".to_string(),
                arguments: serde_json::json!({ "id": "/diagnostics/timeout" }),
            },
        )
        .await
        .unwrap();

    let ToolExecutionOutcome::ToolResult(result) = outcome else {
        panic!("expected tool result");
    };

    assert!(!result.is_error);
    assert_eq!(result.content["document"]["id"], "/diagnostics/timeout");
    assert_eq!(result.content["content"], "\n# Timeout");

    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn instruction_tool_reports_unknown_document_as_tool_error() {
    let root = temp_dir("tool-missing");
    write_doc(&root, "index.md", "Index", "\n# Index");

    let repository = InstructionRepository::new(&root).unwrap();
    let outcome = repository
        .call(
            ToolContext::without_thread(),
            ToolCall {
                id: "call_3".to_string(),
                name: "instructions".to_string(),
                arguments: serde_json::json!({ "id": "/missing" }),
            },
        )
        .await
        .unwrap();

    let ToolExecutionOutcome::ToolResult(result) = outcome else {
        panic!("expected tool result");
    };

    assert!(result.is_error);
    assert_eq!(result.content["missing"], "/missing");

    std::fs::remove_dir_all(root).unwrap();
}
