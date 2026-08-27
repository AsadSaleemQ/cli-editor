use clap::Parser;
use pretty_assertions::assert_eq;

use super::Cli;
use super::Command;

#[test]
fn parses_strict_claude_default() {
    let cli = Cli::try_parse_from(["cli-editor", "default", "claude", "--strict"])
        .expect("command should parse");

    assert!(matches!(
        cli.command,
        Some(Command::Default {
            strict: true,
            no_strict: false,
            ..
        })
    ));
}

#[test]
fn rejects_conflicting_strict_flags() {
    let error = Cli::try_parse_from(["cli-editor", "default", "claude", "--strict", "--no-strict"])
        .expect_err("flags should conflict");

    assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
}
#[test]
fn update_requires_an_explicit_bundle_directory() {
    let error = Cli::try_parse_from(["cli-editor", "update"])
        .expect_err("update without a bundle should fail closed");
    assert_eq!(
        error.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );

    let cli = Cli::try_parse_from(["cli-editor", "update", "--bundle", r"C:\release"])
        .expect("explicit release bundle should parse");
    assert!(matches!(cli.command, Some(Command::Update { .. })));
}
#[test]
fn version_identifies_the_unofficial_distribution() {
    let version = Cli::command().render_version().to_string();
    assert!(version.contains(env!("CARGO_PKG_VERSION")));
    assert!(version.contains("unofficial"));
    assert!(version.contains("not affiliated with OpenAI, Anthropic, or Microsoft"));
}
