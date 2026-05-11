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
}

impl InstructionRepository {
    pub fn new(root_path: impl Into<PathBuf>) -> Result<Self> {
        let root_path = root_path.into();
        let documents = scan_documents(&root_path)?;

        Ok(Self {
            root_path,
            documents: documents
                .into_iter()
                .filter(|loaded| loaded.parse_error.is_none())
                .map(|loaded| (loaded.document.id.clone(), loaded.document))
                .collect(),
        })
    }

    pub fn load(root_path: impl Into<PathBuf>) -> Result<Self> {
        let root_path = root_path.into();
        for issue in Self::lint(&root_path)? {
            let path = issue
                .path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "-".to_string());
            let id = issue.id.as_deref().unwrap_or("-");
            tracing::error!(
                issue_kind = ?issue.kind,
                path = %path,
                id = %id,
                message = %issue.message,
                "support-bot: instruction repository lint issue"
            );
        }

        Self::new(root_path)
    }

    pub fn lint(root_path: impl AsRef<Path>) -> Result<Vec<InstructionLintIssue>> {
        let documents = scan_documents(root_path.as_ref())?;
        Ok(lint_documents(&documents))
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
        Ok(Some(LoadedInstruction {
            document: document.clone(),
            content: parsed.body,
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

#[cfg(test)]
#[path = "../tests/instructions/repository.rs"]
mod tests;
