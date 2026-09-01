use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CliKind {
    Codex,
}

#[derive(Debug, Parser)]
#[command(
    name = "codex-cli-editor",
    version = concat!(env!("CARGO_PKG_VERSION"), " (unofficial; not affiliated with OpenAI or Microsoft)"),
    about
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

impl Cli {
    pub(crate) fn command() -> clap::Command {
        <Self as CommandFactory>::command()
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Install the Codex-only Codex CLI Editor launcher.
    Install {
        #[arg(long)]
        dry_run: bool,
    },
    /// Make enhanced Codex the default Codex route.
    Default {
        #[arg(value_enum)]
        target: DefaultTarget,
    },
    /// Restore native Codex as the default Codex route.
    Restore {
        #[arg(value_enum)]
        target: DefaultTarget,
    },
    /// Show the installed Codex routing state.
    Status,
    /// Validate the Codex installation, shim, and enhanced artifact.
    Doctor {
        #[arg(long)]
        json: bool,
    },
    /// Activate a signed Codex-only release bundle.
    Update {
        #[arg(long, value_name = "DIRECTORY")]
        bundle: std::path::PathBuf,
    },
    /// Adopt a changed native Codex installation.
    Repair {
        #[arg(long, value_enum)]
        adopt_native: Option<DefaultTarget>,
    },
    /// Activate a retained signed Codex-only release.
    Rollback {
        #[arg(long, value_name = "RELEASE")]
        release: Option<String>,
    },
    /// Remove Codex CLI Editor and restore native Codex routing.
    Uninstall,
    /// Run enhanced Codex explicitly.
    Run {
        #[arg(value_enum)]
        target: DefaultTarget,
        #[arg(last = true, trailing_var_arg = true)]
        args: Vec<std::ffi::OsString>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum DefaultTarget {
    Codex,
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
