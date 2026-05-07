use super::repository::ScannedInstructionDocument;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const INDEX_ID: &str = "/index";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstructionLintIssue {
    pub path: Option<PathBuf>,
    pub id: Option<String>,
    pub kind: InstructionLintIssueKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstructionLintIssueKind {
    InvalidFrontmatter,
    MissingTitle,
    DuplicateId,
    MissingIndex,
    BrokenInternalLink,
}

pub(crate) fn lint_documents(
    documents: &[ScannedInstructionDocument],
) -> Vec<InstructionLintIssue> {
    let mut issues = Vec::new();
    let mut ids = BTreeSet::new();
    let mut counts = BTreeMap::<String, usize>::new();

    for scanned in documents {
        *counts.entry(scanned.document.id.clone()).or_default() += 1;
        ids.insert(scanned.document.id.clone());
    }

    for (id, count) in counts {
        if count > 1 {
            issues.push(InstructionLintIssue {
                path: None,
                id: Some(id.clone()),
                kind: InstructionLintIssueKind::DuplicateId,
                message: format!("duplicate instruction id: {id}"),
            });
        }
    }

    if !ids.contains(INDEX_ID) {
        issues.push(InstructionLintIssue {
            path: None,
            id: Some(INDEX_ID.to_string()),
            kind: InstructionLintIssueKind::MissingIndex,
            message: "instruction repository is missing /index".to_string(),
        });
    }

    for scanned in documents {
        if let Some(error) = &scanned.parse_error {
            let kind = if error.contains("missing instruction title") {
                InstructionLintIssueKind::MissingTitle
            } else {
                InstructionLintIssueKind::InvalidFrontmatter
            };
            issues.push(InstructionLintIssue {
                path: Some(scanned.document.path.clone()),
                id: Some(scanned.document.id.clone()),
                kind,
                message: format!("invalid instruction {}: {error}", scanned.document.id),
            });
            continue;
        }

        let Some(body) = &scanned.body else {
            continue;
        };
        for link in internal_links(body) {
            if !ids.contains(&link) {
                issues.push(InstructionLintIssue {
                    path: Some(scanned.document.path.clone()),
                    id: Some(scanned.document.id.clone()),
                    kind: InstructionLintIssueKind::BrokenInternalLink,
                    message: format!(
                        "instruction {} links to missing document {link}",
                        scanned.document.id
                    ),
                });
            }
        }
    }

    issues
}

fn internal_links(body: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut remaining = body;
    while let Some(start) = remaining.find("[/") {
        let after_start = &remaining[start + 1..];
        let Some(end) = after_start.find(']') else {
            break;
        };
        let candidate = &after_start[..end];
        if !candidate.is_empty()
            && candidate.starts_with('/')
            && !candidate.chars().any(char::is_whitespace)
        {
            links.push(candidate.to_string());
        }
        remaining = &after_start[end + 1..];
    }
    links
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::repository::id_from_relative_path;
    use crate::instructions::InstructionRepository;
    use std::path::{Path, PathBuf};

    fn temp_dir(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("support-bot-lint-{name}-{}", std::process::id()));
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
    fn valid_internal_links_pass() {
        let root = temp_dir("valid-links");
        write_doc(&root, "index.md", "Index", "\n[/runbooks/index]");
        write_doc(&root, "runbooks/index.md", "Runbooks", "\n[/index]");

        let issues = InstructionRepository::lint(&root).unwrap();

        assert!(issues.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn broken_internal_link_is_reported_without_recursive_traversal() {
        let root = temp_dir("broken-link");
        write_doc(&root, "index.md", "Index", "\n[/missing]");

        let issues = InstructionRepository::lint(&root).unwrap();

        assert!(issues
            .iter()
            .any(|issue| issue.kind == InstructionLintIssueKind::BrokenInternalLink));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_index_is_reported() {
        let root = temp_dir("missing-index");
        write_doc(&root, "runbooks/one.md", "One", "\n# One");

        let issues = InstructionRepository::lint(&root).unwrap();

        assert!(issues
            .iter()
            .any(|issue| issue.kind == InstructionLintIssueKind::MissingIndex));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_title_is_reported() {
        let root = temp_dir("missing-title");
        std::fs::write(root.join("index.md"), "---\nsummary: Index\n---\n# Index").unwrap();

        let issues = InstructionRepository::lint(&root).unwrap();

        assert!(issues
            .iter()
            .any(|issue| issue.kind == InstructionLintIssueKind::MissingTitle));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn id_derivation_preserves_case_for_canonicalization() {
        assert_eq!(id_from_relative_path(Path::new("A.md")).unwrap(), "/A");
        assert_eq!(id_from_relative_path(Path::new("a.md")).unwrap(), "/a");
    }
}
