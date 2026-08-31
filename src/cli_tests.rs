use clap::Parser;
use pretty_assertions::assert_eq;

use super::Cli;
use super::Command;

#[test]
fn accepts_codex_and_rejects_unknown_targets() {
    let cli = Cli::try_parse_from(["cli-editor", "default", "codex"])
        .expect("Codex default should parse");
    assert!(matches!(cli.command, Some(Command::Default { .. })));

    let error = Cli::try_parse_from(["cli-editor", "default", "other"])
        .expect_err("unknown targets must not be supported");
    assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
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
    assert!(version.contains("not affiliated with OpenAI or Microsoft"));
}

#[test]
fn help_describes_only_codex_routes() {
    let help = Cli::command().render_long_help().to_string();
    assert!(help.contains("Codex-only Codex CLI Editor launcher"));
    assert!(help.contains("signed Codex-only release bundle"));
    assert!(!help.contains("other CLI"));
}
