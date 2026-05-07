use super::frontmatter::parse_markdown;
use super::lint::{lint_documents, InstructionLintIssue};
use crate::error::{Result, SupportBotError};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

const DEFAULT_INDEX_ID: &str = "/index";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionDocument {
    pub id: String,
    pub path: PathBuf,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedInstruction {
    pub document: InstructionDocument,
    pub content: String,
}

pub struct InstructionRepository {
    root_path: PathBuf,
    documents: BTreeMap<String, InstructionDocument>,
    max_instruction_bytes: usize,
}

impl InstructionRepository {
    pub fn new(root_path: impl Into<PathBuf>) -> Result<Self> {
        let root_path = root_path.into();
        let documents = scan_documents(&root_path)?;
        let issues = lint_documents(&documents);
        if !issues.is_empty() {
            return Err(SupportBotError::Instruction(format!(
                "instruction repository has lint errors: {}",
                issues
                    .iter()
                    .map(|issue| issue.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            )));
        }

        Ok(Self {
            root_path,
            documents: documents
                .into_iter()
                .map(|loaded| (loaded.document.id.clone(), loaded.document))
                .collect(),
            max_instruction_bytes: 16 * 1024,
        })
    }

    pub fn lint(root_path: impl AsRef<Path>) -> Result<Vec<InstructionLintIssue>> {
        let documents = scan_documents(root_path.as_ref())?;
        Ok(lint_documents(&documents))
    }

    pub fn with_max_instruction_bytes(mut self, max_instruction_bytes: usize) -> Self {
        self.max_instruction_bytes = max_instruction_bytes;
        self
    }

    pub fn with_max_load_documents(self, _max_load_documents: usize) -> Self {
        self
    }

    pub fn documents(&self) -> impl Iterator<Item = &InstructionDocument> {
        self.documents.values()
    }

    pub fn get_document(&self, id: Option<&str>) -> Result<Option<LoadedInstruction>> {
        let id = id.unwrap_or(DEFAULT_INDEX_ID);
        let Some(document) = self.documents.get(id) else {
            return Ok(None);
        };

        let path = safe_join(&self.root_path, &document.path)?;
        let raw = std::fs::read_to_string(&path).map_err(|err| {
            SupportBotError::Instruction(format!("failed to read {}: {err}", path.display()))
        })?;
        let parsed = parse_markdown(&raw)?;
        let content = truncate_to_char_boundary(parsed.body, self.max_instruction_bytes);

        Ok(Some(LoadedInstruction {
            document: document.clone(),
            content,
        }))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScannedInstructionDocument {
    pub(crate) document: InstructionDocument,
    pub(crate) body: Option<String>,
    pub(crate) parse_error: Option<String>,
}

pub(crate) fn scan_documents(root_path: &Path) -> Result<Vec<ScannedInstructionDocument>> {
    let mut paths = Vec::new();
    collect_markdown_paths(root_path, &mut paths)?;
    paths.sort();

    let mut documents = Vec::new();
    for path in paths {
        let relative = path.strip_prefix(root_path).map_err(|error| {
            SupportBotError::Instruction(format!(
                "failed to relativize instruction path {}: {error}",
                path.display()
            ))
        })?;
        let id = id_from_relative_path(relative)?;
        let raw = std::fs::read_to_string(&path).map_err(|err| {
            SupportBotError::Instruction(format!("failed to read {}: {err}", path.display()))
        })?;

        match parse_markdown(&raw) {
            Ok(parsed) => documents.push(ScannedInstructionDocument {
                document: InstructionDocument {
                    id,
                    path: relative.to_path_buf(),
                    title: parsed.title,
                },
                body: Some(parsed.body),
                parse_error: None,
            }),
            Err(error) => documents.push(ScannedInstructionDocument {
                document: InstructionDocument {
                    id,
                    path: relative.to_path_buf(),
                    title: String::new(),
                },
                body: None,
                parse_error: Some(error.to_string()),
            }),
        }
    }

    Ok(documents)
}

fn collect_markdown_paths(root_path: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(root_path).map_err(|error| {
        SupportBotError::Instruction(format!(
            "failed to read instruction directory {}: {error}",
            root_path.display()
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            SupportBotError::Instruction(format!(
                "failed to read instruction directory entry in {}: {error}",
                root_path.display()
            ))
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            SupportBotError::Instruction(format!(
                "failed to read instruction file type {}: {error}",
                path.display()
            ))
        })?;
        if file_type.is_dir() {
            collect_markdown_paths(&path, paths)?;
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            paths.push(path);
        }
    }

    Ok(())
}

pub(crate) fn id_from_relative_path(relative: &Path) -> Result<String> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        || relative
            .extension()
            .is_none_or(|extension| extension != "md")
    {
        return Err(SupportBotError::Instruction(format!(
            "invalid instruction path: {}",
            relative.display()
        )));
    }

    let without_extension = relative.with_extension("");
    let parts = without_extension
        .components()
        .map(|component| match component {
            Component::Normal(value) => value.to_string_lossy().into_owned(),
            _ => String::new(),
        })
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return Err(SupportBotError::Instruction(format!(
            "invalid instruction path: {}",
            relative.display()
        )));
    }

    Ok(format!("/{}", parts.join("/")))
}

fn safe_join(root: &Path, relative: &Path) -> Result<PathBuf> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(SupportBotError::Instruction(format!(
            "invalid instruction path: {}",
            relative.display()
        )));
    }

    Ok(root.join(relative))
}

fn truncate_to_char_boundary(mut content: String, max_bytes: usize) -> String {
    if content.len() <= max_bytes {
        return content;
    }

    let mut boundary = max_bytes;
    while !content.is_char_boundary(boundary) {
        boundary -= 1;
    }
    content.truncate(boundary);
    content
}

#[cfg(test)]
mod tests {
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
}
