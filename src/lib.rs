mod cli;
mod compatibility;
mod discovery;
mod dispatcher;
mod doctor;
mod error;
mod installer;
mod process;
mod registry;
mod state;
mod version;
mod vscode;
use clap::Parser;
use cli::Cli;
use cli::Command;
use error::Result;

pub use cli::CliKind;
pub use discovery::DiscoveryOptions;
pub use discovery::discover_native;
pub use error::CodexCliEditorError;
pub use process::run_native;
pub use state::AdoptionRecord;
pub use state::ManifestCacheRecord;
pub use state::NativeTarget;
pub use state::RegistryValueSnapshot;
pub use state::State;
pub use state::StateStore;

pub fn run() -> Result<i32> {
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let executable = std::env::current_exe()
        .ok()
        .or_else(|| args.first().map(std::path::PathBuf::from))
        .ok_or(CodexCliEditorError::StateDirectoryUnavailable)?;
    if dispatcher::invocation_kind(&executable) {
        return dispatcher::run_shim(args.into_iter().skip(1).collect());
    }

    let cli = Cli::parse_from(args);
    match cli.command {
        Some(Command::Install { dry_run }) => {
            installer::install(dry_run)?;
            Ok(0)
        }
        Some(Command::Default { target }) => {
            installer::configure_default(target)?;
            Ok(0)
        }
        Some(Command::Restore { target }) => {
            installer::restore_defaults(target)?;
            Ok(0)
        }
        Some(Command::Uninstall) => {
            installer::uninstall()?;
            Ok(0)
        }
        Some(Command::Status) => {
            doctor::status()?;
            Ok(0)
        }
        Some(Command::Doctor { json }) => doctor::doctor(json),
        Some(Command::Update { bundle }) => {
            installer::update(&bundle)?;
            Ok(0)
        }
        Some(Command::Run { target: _, args }) => dispatcher::run_managed(args, true),
        Some(Command::Repair { adopt_native }) => {
            installer::repair(adopt_native)?;
            Ok(0)
        }
        Some(Command::Rollback { release }) => {
            installer::rollback(release.as_deref())?;
            Ok(0)
        }
        None => {
            Cli::command()
                .print_help()
                .map_err(|source| CodexCliEditorError::io("stdout", source))?;
            println!();
            Ok(0)
        }
    }
}

#[cfg(test)]
mod test_support {
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    pub struct TempDir(PathBuf);

    impl TempDir {
        pub fn new() -> Self {
            let unique = format!(
                "codex-cli-editor-test-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            );
            let path = std::env::temp_dir().join(unique);
            std::fs::create_dir(&path).expect("create isolated test directory");
            Self(path)
        }

        pub fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}
