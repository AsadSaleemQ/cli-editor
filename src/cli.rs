use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum CliKind {
    Codex,
    Claude,
}

#[derive(Debug, Parser)]
#[command(
    name = "cli-editor",
    version = concat!(env!("CARGO_PKG_VERSION"), " (unofficial; not affiliated with OpenAI, Anthropic, or Microsoft)"),
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
    Install {
        #[arg(long)]
        dry_run: bool,
    },
    Default {
        #[arg(value_enum)]
        target: DefaultTarget,
        #[arg(long, conflicts_with = "no_strict")]
        strict: bool,
        #[arg(long, conflicts_with = "strict")]
        no_strict: bool,
    },
    Restore {
        #[arg(value_enum)]
        target: DefaultTarget,
    },
    Status,
    Doctor {
        #[arg(long)]
        json: bool,
    },
    Update {
        #[arg(long, value_name = "DIRECTORY")]
        bundle: std::path::PathBuf,
    },
    Repair {
        #[arg(long, value_enum)]
        adopt_native: Option<CliKind>,
    },
    Rollback {
        #[arg(long, value_name = "RELEASE")]
        release: Option<String>,
    },
    Uninstall,
    Run {
        #[arg(value_enum)]
        target: CliKind,
        #[arg(last = true, trailing_var_arg = true)]
        args: Vec<std::ffi::OsString>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum DefaultTarget {
    Codex,
    Claude,
    All,
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
