use super::*;
use crate::instructions::repository::id_from_relative_path;
use crate::instructions::InstructionRepository;
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("support-bot-lint-{name}-{}", std::process::id()));
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
