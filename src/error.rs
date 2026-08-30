use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, CliEditorError>;

#[derive(Debug, thiserror::Error)]
pub enum CliEditorError {
    #[error("CLI Editor state directory is unavailable")]
    StateDirectoryUnavailable,
    #[error("another CLI Editor operation is in progress")]
    LockTimeout,
    #[error("state file is unsupported: expected schema {expected}, found {found}")]
    UnsupportedStateSchema { expected: u32, found: u32 },
    #[error(
        "native target was not found for {0:?}; after installing it, run `cli-editor repair --adopt-native codex`"
    )]
    TargetNotFound(crate::CliKind),
    #[error(
        "native target is missing for {kind:?} at {path}; reinstall Codex, then run `cli-editor repair --adopt-native codex`, or run `cli-editor uninstall`"
    )]
    NativeTargetMissing { kind: crate::CliKind, path: PathBuf },
    #[error("version probe failed for {0}")]
    VersionProbeFailed(PathBuf),
    #[error("version probe timed out after {timeout_seconds} seconds for {path}")]
    VersionProbeTimedOut { path: PathBuf, timeout_seconds: u64 },
    #[error("native target is unsafe or unsupported: {0}")]
    UnsafeTarget(PathBuf),
    #[error(
        "CLI Editor cannot safely manage launcher {0}; install the official Codex package or remove/reorder this launcher, then rerun install"
    )]
    UnsupportedLauncher(PathBuf),
    #[error("argument contains a NUL character")]
    ArgumentNul,
    #[error("Windows command line exceeds the 32,767 UTF-16 code-unit limit")]
    CommandLineTooLong,
    #[error("Windows API {api} failed: {source}")]
    WindowsApi {
        api: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("CLI Editor is not installed")]
    NotInstalled,
    #[error("enhanced Codex is unavailable or not compatible")]
    EnhancedUnavailable,
    #[error(
        "recorded target changed and must be revalidated: {0}; run `cli-editor repair --adopt-native codex`"
    )]
    TargetChanged(PathBuf),
    #[error("CLI Editor shim recursion was detected")]
    RecursionDetected,
    #[error("Windows registry API {api} failed: {source}")]
    RegistryApi {
        api: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("the user PATH registry value has an unsupported type or encoding")]
    UnsupportedUserPath,
    #[error("a supported native Codex CLI was not found")]
    NoSupportedCliFound,
    #[error("required release artifact is missing: {0}")]
    MissingReleaseArtifact(PathBuf),
    #[error("release bundle contains retired Claude compatibility metadata")]
    LegacyClaudeReleaseUnsupported,
    #[error("embedded manifest public key is invalid")]
    InvalidManifestKey,
    #[error("compatibility manifest signature is invalid")]
    InvalidManifestSignature,
    #[error("compatibility manifest schema is unsupported: {0}")]
    UnsupportedManifestSchema(u32),
    #[error("compatibility manifest sequence rollback: highest {highest}, received {received}")]
    ManifestRollback { highest: u64, received: u64 },
    #[error("compatibility manifest issue/expiry window is invalid")]
    InvalidManifestWindow,
    #[error("release build still embeds the public development signing key")]
    DevelopmentKeyReleaseBlocked,
    #[error("cached compatibility manifest does not match the active state record")]
    ManifestCacheMismatch,
    #[error("no retained signed release is available for rollback")]
    NoRollbackAvailable,
    #[error("compatibility manifest is expired beyond its grace period")]
    ManifestExpired,
    #[error("release manifest does not support Codex version {0}")]
    UnsupportedCodexVersion(String),
    #[error("release manifest requires dispatcher {required}, current is {current}")]
    DispatcherTooOld { required: String, current: String },
    #[error("release manifest does not contain required artifact {0}")]
    ArtifactNotDeclared(String),
    #[error("release artifact verification failed: {0}")]
    ArtifactVerificationFailed(PathBuf),
    #[error("repair requires --adopt-native codex")]
    RepairTargetRequired,
    #[error("CLI Editor state changed while the operation was being prepared; retry")]
    StateChangedDuringOperation,
    #[error("the release bundle does not advance the accepted manifest sequence")]
    NoUpdateAvailable,
    #[error("run the cli-editor.exe from the new external release bundle to update the dispatcher")]
    ExternalUpdaterRequired,
    #[error("VS Code extension error: {0}")]
    VscodeBridge(String),
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Cli(#[from] clap::Error),
}

impl CliEditorError {
    /// Uses shell-reserved launcher codes so wrapper failures are distinguishable from child exits.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::TargetNotFound(_)
            | Self::NativeTargetMissing { .. }
            | Self::VersionProbeFailed(_)
            | Self::VersionProbeTimedOut { .. }
            | Self::UnsafeTarget(_)
            | Self::UnsupportedLauncher(_)
            | Self::TargetChanged(_)
            | Self::RecursionDetected
            | Self::NoSupportedCliFound
            | Self::ArtifactVerificationFailed(_) => 126,
            _ => 125,
        }
    }

    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::CliEditorError;

    #[test]
    fn launcher_errors_use_reserved_exit_codes() {
        assert_eq!(
            CliEditorError::TargetNotFound(crate::CliKind::Codex).exit_code(),
            126
        );
        assert_eq!(
            CliEditorError::NativeTargetMissing {
                kind: crate::CliKind::Codex,
                path: PathBuf::from(r"C:\missing\codex.exe"),
            }
            .exit_code(),
            126
        );
        assert_eq!(CliEditorError::NotInstalled.exit_code(), 125);
    }
}
