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
