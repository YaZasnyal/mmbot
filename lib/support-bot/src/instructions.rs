use crate::error::{Result, SupportBotError};
use crate::tools::{
    SupportTool, ToolCall, ToolContext, ToolExecutionOutcome, ToolKind, ToolResult, ToolSpec,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstructionManifest {
    pub documents: Vec<InstructionDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstructionDocument {
    pub id: String,
    pub path: PathBuf,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstructionItem {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub has_children: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedInstruction {
    pub document: InstructionDocument,
    pub content: String,
}

pub struct InstructionRepository {
    root_path: PathBuf,
    manifest: InstructionManifest,
    max_instruction_bytes: usize,
    max_load_documents: usize,
}

impl InstructionRepository {
    pub fn new(root_path: impl Into<PathBuf>, manifest: InstructionManifest) -> Self {
        Self {
            root_path: root_path.into(),
            manifest,
            max_instruction_bytes: 16 * 1024,
            max_load_documents: 5,
        }
    }

    pub fn with_max_instruction_bytes(mut self, max_instruction_bytes: usize) -> Self {
        self.max_instruction_bytes = max_instruction_bytes;
        self
    }

    pub fn with_max_load_documents(mut self, max_load_documents: usize) -> Self {
        self.max_load_documents = max_load_documents;
        self
    }

    pub fn manifest(&self) -> &InstructionManifest {
        &self.manifest
    }

    pub fn list_children<'a>(&'a self, parent: Option<&str>) -> Vec<&'a InstructionDocument> {
        let mut documents = self
            .manifest
            .documents
            .iter()
            .filter(|document| document.parent.as_deref() == parent)
            .collect::<Vec<_>>();
        documents.sort_by(|left, right| left.id.cmp(&right.id));
        documents
    }

    pub fn list_child_items(&self, parent: Option<&str>) -> Vec<InstructionItem> {
        self.list_children(parent)
            .into_iter()
            .map(|document| self.to_item(document))
            .collect()
    }

    pub fn get_document(&self, id: &str, max_bytes: usize) -> Result<Option<LoadedInstruction>> {
        self.manifest
            .documents
            .iter()
            .find(|document| document.id == id)
            .map(|document| self.load_document(document, max_bytes))
            .transpose()
    }

    fn load_document(
        &self,
        document: &InstructionDocument,
        max_bytes: usize,
    ) -> Result<LoadedInstruction> {
        let path = safe_join(&self.root_path, &document.path)?;
        let content = std::fs::read_to_string(&path).map_err(|err| {
            SupportBotError::Instruction(format!("failed to read {}: {err}", path.display()))
        })?;
        let content = truncate_to_char_boundary(content, max_bytes);

        Ok(LoadedInstruction {
            document: document.clone(),
            content,
        })
    }

    fn to_item(&self, document: &InstructionDocument) -> InstructionItem {
        InstructionItem {
            id: document.id.clone(),
            title: document.title.clone(),
            summary: document.summary.clone(),
            has_children: self.has_children(&document.id),
            tags: document.tags.clone(),
        }
    }

    fn has_children(&self, id: &str) -> bool {
        self.manifest
            .documents
            .iter()
            .any(|document| document.parent.as_deref() == Some(id))
    }
}

#[async_trait]
impl SupportTool for InstructionRepository {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "instructions".to_string(),
            description: "Navigate and load support runbooks/instructions. Use list to inspect a tree level and load to read one or more concrete articles.".to_string(),
            kind: ToolKind::ReadOnly,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list", "load"],
                        "description": "Instruction repository action."
                    },
                    "parent_id": {
                        "type": ["string", "null"],
                        "description": "Parent id for list. Omit or null to list root items."
                    },
                    "ids": {
                        "type": "array",
                        "items": { "type": "string" },
                        "minItems": 1,
                        "description": "Document ids for load action."
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 20,
                        "description": "Maximum number of items for list."
                    }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        }
    }

    async fn call(&self, _ctx: ToolContext, call: ToolCall) -> Result<ToolExecutionOutcome> {
        let call_id = call.id;
        let args: InstructionToolArgs = serde_json::from_value(call.arguments)?;
        let content = match args.action.as_str() {
            "list" => {
                let limit = args.limit.unwrap_or(20).min(20);
                let items = self
                    .list_child_items(args.parent_id.as_deref())
                    .into_iter()
                    .take(limit)
                    .collect::<Vec<_>>();
                serde_json::json!({ "items": items })
            }
            "load" => {
                let ids = args.ids.ok_or_else(|| {
                    SupportBotError::Tool("instructions.load requires ids".to_string())
                })?;
                if ids.len() > self.max_load_documents {
                    return Err(SupportBotError::Tool(format!(
                        "instructions.load accepts at most {} ids",
                        self.max_load_documents
                    )));
                }

                let mut documents = Vec::new();
                let mut missing = Vec::new();

                for id in ids {
                    match self.get_document(&id, self.max_instruction_bytes)? {
                        Some(loaded) => documents.push(serde_json::json!({
                            "document": {
                                "id": loaded.document.id,
                                "title": loaded.document.title,
                                "summary": loaded.document.summary,
                                "parent": loaded.document.parent,
                                "tags": loaded.document.tags,
                                "has_children": self.has_children(&id)
                            },
                            "content": loaded.content
                        })),
                        None => missing.push(id),
                    }
                }

                serde_json::json!({
                    "documents": documents,
                    "missing": missing
                })
            }
            other => {
                return Err(SupportBotError::Tool(format!(
                    "unknown instructions action: {other}"
                )))
            }
        };

        Ok(ToolExecutionOutcome::ToolResult(ToolResult {
            call_id,
            content,
            is_error: false,
        }))
    }
}

#[derive(Debug, Deserialize)]
struct InstructionToolArgs {
    action: String,
    parent_id: Option<String>,
    ids: Option<Vec<String>>,
    limit: Option<usize>,
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

    #[test]
    fn lists_children_by_parent() {
        let repository = InstructionRepository::new(
            ".",
            InstructionManifest {
                documents: vec![
                    InstructionDocument {
                        id: "diagnostics".to_string(),
                        path: "diagnostics/README.md".into(),
                        title: "Diagnostics".to_string(),
                        summary: "Diagnostics index".to_string(),
                        parent: None,
                        tags: Vec::new(),
                    },
                    InstructionDocument {
                        id: "diagnostics.timeout".to_string(),
                        path: "diagnostics/timeout.md".into(),
                        title: "Timeouts".to_string(),
                        summary: "Diagnose timeouts".to_string(),
                        parent: Some("diagnostics".to_string()),
                        tags: Vec::new(),
                    },
                ],
            },
        );

        let children = repository.list_children(Some("diagnostics"));

        assert_eq!(children[0].id, "diagnostics.timeout");
    }

    #[tokio::test]
    async fn instruction_repository_works_as_tool_for_listing() {
        let repository = InstructionRepository::new(
            ".",
            InstructionManifest {
                documents: vec![InstructionDocument {
                    id: "diagnostics".to_string(),
                    path: "diagnostics/README.md".into(),
                    title: "Diagnostics".to_string(),
                    summary: "Diagnostics index".to_string(),
                    parent: None,
                    tags: vec!["triage".to_string()],
                }],
            },
        );

        let outcome = repository
            .call(
                ToolContext::without_thread(),
                ToolCall {
                    id: "call_1".to_string(),
                    name: "instructions".to_string(),
                    arguments: serde_json::json!({
                        "action": "list",
                        "parent_id": null
                    }),
                },
            )
            .await
            .unwrap();

        let ToolExecutionOutcome::ToolResult(result) = outcome else {
            panic!("expected tool result");
        };

        assert_eq!(result.call_id, "call_1");
        assert_eq!(result.content["items"][0]["id"], "diagnostics");
        assert_eq!(result.content["items"][0]["has_children"], false);
    }

    #[tokio::test]
    async fn instruction_repository_loads_multiple_documents_as_tool() {
        let temp_dir =
            std::env::temp_dir().join(format!("support-bot-instructions-{}", std::process::id()));
        std::fs::create_dir_all(temp_dir.join("diagnostics")).unwrap();
        std::fs::write(temp_dir.join("diagnostics/one.md"), "First runbook").unwrap();
        std::fs::write(temp_dir.join("diagnostics/two.md"), "Second runbook").unwrap();

        let repository = InstructionRepository::new(
            &temp_dir,
            InstructionManifest {
                documents: vec![
                    InstructionDocument {
                        id: "diagnostics.one".to_string(),
                        path: "diagnostics/one.md".into(),
                        title: "One".to_string(),
                        summary: "First".to_string(),
                        parent: Some("diagnostics".to_string()),
                        tags: Vec::new(),
                    },
                    InstructionDocument {
                        id: "diagnostics.two".to_string(),
                        path: "diagnostics/two.md".into(),
                        title: "Two".to_string(),
                        summary: "Second".to_string(),
                        parent: Some("diagnostics".to_string()),
                        tags: Vec::new(),
                    },
                ],
            },
        );

        let outcome = repository
            .call(
                ToolContext::without_thread(),
                ToolCall {
                    id: "call_2".to_string(),
                    name: "instructions".to_string(),
                    arguments: serde_json::json!({
                        "action": "load",
                        "ids": ["diagnostics.one", "diagnostics.two"]
                    }),
                },
            )
            .await
            .unwrap();

        let ToolExecutionOutcome::ToolResult(result) = outcome else {
            panic!("expected tool result");
        };

        assert_eq!(result.call_id, "call_2");
        assert_eq!(result.content["documents"].as_array().unwrap().len(), 2);
        assert_eq!(result.content["documents"][0]["content"], "First runbook");

        std::fs::remove_dir_all(temp_dir).unwrap();
    }
}
