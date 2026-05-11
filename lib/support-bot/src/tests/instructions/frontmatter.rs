use super::*;

#[test]
fn parses_title_and_body_without_frontmatter() {
    let parsed = parse_markdown("---\ntitle: Diagnostics\n---\n\n# Diagnostics").unwrap();

    assert_eq!(parsed.title, "Diagnostics");
    assert_eq!(parsed.body, "\n# Diagnostics");
}

#[test]
fn rejects_missing_title() {
    let error = parse_markdown("---\nsummary: Missing\n---\nBody").unwrap_err();

    assert!(error.to_string().contains("missing instruction title"));
}

#[test]
fn rejects_empty_title() {
    let error = parse_markdown("---\ntitle: '   '\n---\nBody").unwrap_err();

    assert!(error.to_string().contains("missing instruction title"));
}
