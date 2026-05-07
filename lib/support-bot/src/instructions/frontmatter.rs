use crate::error::{Result, SupportBotError};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedMarkdown {
    pub(crate) title: String,
    pub(crate) body: String,
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    title: Option<String>,
}

pub(crate) fn parse_markdown(raw: &str) -> Result<ParsedMarkdown> {
    let (yaml, body) = split_frontmatter(raw)?;
    let frontmatter = serde_yaml::from_str::<Frontmatter>(yaml).map_err(|error| {
        SupportBotError::Instruction(format!("invalid instruction frontmatter yaml: {error}"))
    })?;
    let title = frontmatter
        .title
        .map(|title| title.trim().to_string())
        .filter(|title| !title.is_empty())
        .ok_or_else(|| SupportBotError::Instruction("missing instruction title".to_string()))?;

    Ok(ParsedMarkdown {
        title,
        body: body.to_string(),
    })
}

fn split_frontmatter(raw: &str) -> Result<(&str, &str)> {
    let first_line_end = raw.find('\n').ok_or_else(|| {
        SupportBotError::Instruction("instruction file is missing frontmatter".to_string())
    })?;
    let first_line = raw[..first_line_end].trim_end_matches('\r');
    if first_line != "---" {
        return Err(SupportBotError::Instruction(
            "instruction file must start with yaml frontmatter".to_string(),
        ));
    }

    let yaml_start = first_line_end + 1;
    let mut line_start = yaml_start;
    for line in raw[yaml_start..].split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed == "---" {
            let yaml = &raw[yaml_start..line_start];
            let body_start = line_start + line.len();
            return Ok((yaml, &raw[body_start..]));
        }
        line_start += line.len();
    }

    Err(SupportBotError::Instruction(
        "instruction file is missing closing frontmatter delimiter".to_string(),
    ))
}

#[cfg(test)]
mod tests {
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
}
