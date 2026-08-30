use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;
use sha2::Digest;
use sha2::Sha256;

use crate::CliKind;
use crate::NativeTarget;
use crate::error::CliEditorError;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    pub path: std::ffi::OsString,
    pub current_dir: PathBuf,
    pub shim_dir: PathBuf,
}

impl DiscoveryOptions {
    pub fn from_environment(shim_dir: PathBuf) -> Result<Self> {
        let current_dir =
            std::env::current_dir().map_err(|source| CliEditorError::io(".", source))?;
        Ok(Self {
            path: std::env::var_os("PATH").unwrap_or_default(),
            current_dir,
            shim_dir,
        })
    }
}

pub fn discover_native(options: &DiscoveryOptions) -> Result<NativeTarget> {
    let excluded = canonical_existing_dirs([&options.current_dir, &options.shim_dir]);
    for directory in std::env::split_paths(&options.path) {
        if directory.as_os_str().is_empty()
            || !directory.is_absolute()
            || is_excluded_directory(&directory, &excluded)
        {
            continue;
        }
        for name in candidate_names() {
            let candidate = directory.join(name);
            if !candidate.is_file() {
                continue;
            }
            if let Some(resolved) = resolve_candidate(&candidate)? {
                return inspect_target(&resolved.path, &resolved.package_root, resolved.identity);
            }
        }
    }
    Err(CliEditorError::TargetNotFound(CliKind::Codex))
}

pub(crate) fn inspect_target(
    path: &Path,
    package_root: &Path,
    package_identity: String,
) -> Result<NativeTarget> {
    if !has_exe_extension(path) {
        return Err(CliEditorError::UnsafeTarget(path.to_path_buf()));
    }
    let path = path
        .canonicalize()
        .map_err(|source| CliEditorError::io(path, source))?;
    if !has_exe_extension(&path) {
        return Err(CliEditorError::UnsafeTarget(path));
    }
    let package_root = package_root
        .canonicalize()
        .map_err(|source| CliEditorError::io(package_root, source))?;
    if !path.starts_with(&package_root) {
        return Err(CliEditorError::UnsafeTarget(path));
    }

    let metadata = path
        .metadata()
        .map_err(|source| CliEditorError::io(&path, source))?;
    let modified_unix_ms = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |duration| duration.as_millis());
    let version = probe_version(&path)?;
    Ok(NativeTarget {
        sha256: sha256_file(&path)?,
        path,
        package_root,
        package_identity: format!("codex:{package_identity}"),
        version,
        file_size: metadata.len(),
        modified_unix_ms,
    })
}

pub(crate) fn refresh_recorded_target(recorded: &NativeTarget) -> Result<NativeTarget> {
    let resolved = resolve_recorded_target(recorded)?;
    let refreshed = inspect_target(&resolved.path, &resolved.package_root, resolved.identity)?;
    if !same_package_identity(&recorded.package_identity, &refreshed.package_identity) {
        return Err(CliEditorError::TargetChanged(recorded.path.clone()));
    }
    Ok(refreshed)
}

pub(crate) fn validate_recorded_target_identity(recorded: &NativeTarget) -> Result<()> {
    resolve_recorded_target(recorded).map(|_| ())
}

fn resolve_recorded_target(recorded: &NativeTarget) -> Result<ResolvedCandidate> {
    let path = recorded.path.canonicalize().map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            CliEditorError::NativeTargetMissing {
                kind: CliKind::Codex,
                path: recorded.path.clone(),
            }
        } else {
            CliEditorError::io(&recorded.path, source)
        }
    })?;
    let package_root = recorded
        .package_root
        .canonicalize()
        .map_err(|source| CliEditorError::io(&recorded.package_root, source))?;
    if path != recorded.path
        || package_root != recorded.package_root
        || !path.starts_with(&package_root)
        || !has_exe_extension(&path)
    {
        return Err(CliEditorError::TargetChanged(recorded.path.clone()));
    }

    let native_identity = "codex:native-executable";
    let identity = if recorded.package_identity == native_identity {
        "native-executable".into()
    } else if recorded
        .package_identity
        .starts_with("codex:@openai/codex@")
    {
        let package_json_path = package_root.join("package.json");
        let package_json: PackageJson = serde_json::from_slice(
            &std::fs::read(&package_json_path)
                .map_err(|source| CliEditorError::io(&package_json_path, source))?,
        )?;
        if package_json.name != "@openai/codex" || !is_expected_npm_codex_path(&path, &package_root)
        {
            return Err(CliEditorError::TargetChanged(recorded.path.clone()));
        }
        format!("@openai/codex@{}", package_json.version)
    } else {
        return Err(CliEditorError::TargetChanged(recorded.path.clone()));
    };
    Ok(ResolvedCandidate {
        path,
        package_root,
        identity,
    })
}
pub(crate) fn same_package_identity(previous: &str, next: &str) -> bool {
    previous == next
        || previous == "codex:native-executable" && next == "codex:native-executable"
        || previous.starts_with("codex:@openai/codex@") && next.starts_with("codex:@openai/codex@")
}
fn is_expected_npm_codex_path(path: &Path, package_root: &Path) -> bool {
    let relative = path.strip_prefix(package_root).ok();
    relative.is_some_and(|relative| {
        relative
            == Path::new(
                r"node_modules\@openai\codex-win32-x64\vendor\x86_64-pc-windows-msvc\bin\codex.exe",
            )
            || relative == Path::new(r"vendor\x86_64-pc-windows-msvc\bin\codex.exe")
    })
}
pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(|source| CliEditorError::io(path, source))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|source| CliEditorError::io(path, source))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(hex::encode(digest.finalize()))
}

struct ResolvedCandidate {
    path: PathBuf,
    package_root: PathBuf,
    identity: String,
}

fn resolve_candidate(candidate: &Path) -> Result<Option<ResolvedCandidate>> {
    match candidate {
        candidate if has_npm_shim_extension(candidate) => resolve_npm_codex(candidate).map(Some),
        candidate if has_script_extension(candidate) => {
            Err(CliEditorError::UnsafeTarget(candidate.to_path_buf()))
        }
        candidate if has_exe_extension(candidate) => {
            let package_root = candidate
                .parent()
                .ok_or_else(|| CliEditorError::UnsafeTarget(candidate.to_path_buf()))?
                .to_path_buf();
            Ok(Some(ResolvedCandidate {
                path: candidate.to_path_buf(),
                package_root,
                identity: "native-executable".into(),
            }))
        }
        _ => Ok(None),
    }
}

fn read_official_package_json(shim: &Path, package_json_path: &Path) -> Result<PackageJson> {
    let bytes = std::fs::read(package_json_path)
        .map_err(|_| CliEditorError::UnsupportedLauncher(shim.to_path_buf()))?;
    serde_json::from_slice(&bytes)
        .map_err(|_| CliEditorError::UnsupportedLauncher(shim.to_path_buf()))
}

fn resolve_npm_codex(shim: &Path) -> Result<ResolvedCandidate> {
    let npm_root = shim
        .parent()
        .ok_or_else(|| CliEditorError::UnsafeTarget(shim.to_path_buf()))?;
    let package_root = npm_root.join("node_modules").join("@openai").join("codex");
    let package_json_path = package_root.join("package.json");
    let package_json = read_official_package_json(shim, &package_json_path)?;
    if package_json.name != "@openai/codex" {
        return Err(CliEditorError::UnsupportedLauncher(shim.to_path_buf()));
    }

    let platform_path = package_root
        .join("node_modules")
        .join("@openai")
        .join("codex-win32-x64")
        .join("vendor")
        .join("x86_64-pc-windows-msvc")
        .join("bin")
        .join("codex.exe");
    let fallback_path = package_root
        .join("vendor")
        .join("x86_64-pc-windows-msvc")
        .join("bin")
        .join("codex.exe");
    let path = if platform_path.is_file() {
        platform_path
    } else if fallback_path.is_file() {
        fallback_path
    } else {
        return Err(CliEditorError::TargetNotFound(CliKind::Codex));
    };
    Ok(ResolvedCandidate {
        path,
        package_root,
        identity: format!("@openai/codex@{}", package_json.version),
    })
}

pub(crate) const NATIVE_VERSION_PROBE_TIMEOUT: std::time::Duration =
    std::time::Duration::from_secs(60);
pub(crate) const RELEASE_VERSION_PROBE_TIMEOUT: std::time::Duration =
    std::time::Duration::from_secs(60);

pub(crate) fn probe_version(path: &Path) -> Result<String> {
    probe_version_with_timeout(path, NATIVE_VERSION_PROBE_TIMEOUT)
}

pub(crate) fn probe_release_version(path: &Path) -> Result<String> {
    probe_version_with_timeout(path, RELEASE_VERSION_PROBE_TIMEOUT)
}

const MAX_VERSION_OUTPUT_BYTES: u64 = 64 * 1024;
static PROBE_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct ProbeOutput {
    path: PathBuf,
    file: Option<File>,
}

impl ProbeOutput {
    fn create() -> Result<Self> {
        let directory = std::env::temp_dir();
        for _ in 0..100 {
            let sequence = PROBE_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = directory.join(format!(
                "cli-editor-version-probe-{}-{sequence}.tmp",
                std::process::id()
            ));
            match std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(file) => {
                    return Ok(Self {
                        path,
                        file: Some(file),
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(source) => return Err(CliEditorError::io(&path, source)),
            }
        }
        Err(CliEditorError::VersionProbeFailed(directory))
    }

    fn child_stdout(&self) -> Result<File> {
        self.file
            .as_ref()
            .expect("probe output remains open")
            .try_clone()
            .map_err(|source| CliEditorError::io(&self.path, source))
    }

    fn read(&mut self, target: &Path) -> Result<Vec<u8>> {
        let file = self.file.as_mut().expect("probe output remains open");
        file.seek(SeekFrom::Start(0))
            .map_err(|source| CliEditorError::io(&self.path, source))?;
        let mut bytes = Vec::new();
        file.take(MAX_VERSION_OUTPUT_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|source| CliEditorError::io(&self.path, source))?;
        if bytes.len() as u64 > MAX_VERSION_OUTPUT_BYTES {
            return Err(CliEditorError::VersionProbeFailed(target.to_path_buf()));
        }
        Ok(bytes)
    }
}

impl Drop for ProbeOutput {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = std::fs::remove_file(&self.path);
    }
}

fn probe_version_with_timeout(path: &Path, timeout: std::time::Duration) -> Result<String> {
    let working_directory = path
        .parent()
        .ok_or_else(|| CliEditorError::UnsafeTarget(path.to_path_buf()))?;
    let mut output = ProbeOutput::create()?;
    let child_stdout = output.child_stdout()?;
    let mut child = Command::new(path)
        .arg("--version")
        .current_dir(working_directory)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(child_stdout))
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|source| CliEditorError::io(path, source))?;
    let started = std::time::Instant::now();
    let status = loop {
        match child
            .try_wait()
            .map_err(|source| CliEditorError::io(path, source))?
        {
            Some(status) => break status,
            None if started.elapsed() < timeout => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CliEditorError::VersionProbeTimedOut {
                    path: path.to_path_buf(),
                    timeout_seconds: timeout.as_secs(),
                });
            }
        }
    };
    if !status.success() {
        return Err(CliEditorError::VersionProbeFailed(path.to_path_buf()));
    }
    let stdout = output.read(path)?;
    let stdout = String::from_utf8_lossy(&stdout);
    let version = stdout.trim();
    if version.is_empty() {
        return Err(CliEditorError::VersionProbeFailed(path.to_path_buf()));
    }
    Ok(version.to_owned())
}
fn candidate_names() -> &'static [&'static str] {
    &["codex.exe", "codex.bat", "codex.cmd", "codex.ps1"]
}

fn canonical_existing_dirs<'a>(directories: impl IntoIterator<Item = &'a PathBuf>) -> Vec<PathBuf> {
    directories
        .into_iter()
        .filter_map(|path| path.canonicalize().ok())
        .collect()
}

fn is_excluded_directory(directory: &Path, excluded: &[PathBuf]) -> bool {
    directory
        .canonicalize()
        .ok()
        .is_some_and(|directory| excluded.iter().any(|item| item == &directory))
}

fn has_exe_extension(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
}

fn has_script_extension(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("bat")
                || extension.eq_ignore_ascii_case("cmd")
                || extension.eq_ignore_ascii_case("ps1")
        })
}

fn has_npm_shim_extension(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("cmd") || extension.eq_ignore_ascii_case("ps1")
        })
}
#[derive(Deserialize)]
struct PackageJson {
    name: String,
    version: String,
}

impl CliKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::LegacyClaude => "claude",
        }
    }
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
