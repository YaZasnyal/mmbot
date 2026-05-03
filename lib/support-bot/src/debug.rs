use crate::config::DebugCommandConfig;
use crate::error::Result;
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugCommand {
    pub name: String,
    pub args: Vec<String>,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugCommandMatch {
    pub command: DebugCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugResponse {
    pub message: String,
}

#[async_trait]
pub trait DebugCommandHandler: Send + Sync {
    async fn handle_debug_command(&self, command: DebugCommand) -> Result<DebugResponse>;
}

impl DebugCommand {
    pub fn parse(message: &str, config: &DebugCommandConfig) -> Option<DebugCommandMatch> {
        if !config.enabled {
            return None;
        }

        let trimmed = message.trim();
        let prefix = config.prefixes.iter().find(|prefix| {
            trimmed
                .strip_prefix(prefix.as_str())
                .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
        })?;
        let rest = trimmed[prefix.len()..].trim();
        if rest.is_empty() {
            return None;
        }

        let mut parts = rest.split_whitespace();
        let name = parts.next()?.to_string();
        let args = parts.map(ToString::to_string).collect();

        Some(DebugCommandMatch {
            command: DebugCommand {
                name,
                args,
                raw: trimmed.to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_prefixed_debug_command() {
        let config = DebugCommandConfig::default();
        let matched = DebugCommand::parse("!support state abc123", &config).unwrap();

        assert_eq!(matched.command.name, "state");
        assert_eq!(matched.command.args, vec!["abc123"]);
    }

    #[test]
    fn ignores_non_debug_messages() {
        let config = DebugCommandConfig::default();

        assert!(DebugCommand::parse("normal engineer discussion", &config).is_none());
    }

    #[test]
    fn does_not_match_longer_prefix_words() {
        let config = DebugCommandConfig::default();

        assert!(DebugCommand::parse("!supportive state", &config).is_none());
    }
}
