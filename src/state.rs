use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;

use crate::CliKind;
use crate::error::CliEditorError;
use crate::error::Result;

pub const STATE_SCHEMA_VERSION: u32 = 1;
const ADOPTION_HISTORY_LIMIT: usize = 32;
const STATE_LOCK_TIMEOUT: Duration = Duration::from_secs(5);
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub schema_version: u32,
    pub install_id: String,
    pub installed_version: String,
    pub pre_install_user_path: Option<RegistryValueSnapshot>,
    pub shim_directory: Option<PathBuf>,
    pub path_entry_added: bool,
    pub defaults: DefaultSelections,
    pub native_targets: BTreeMap<CliKind, NativeTarget>,
    pub active_release: Option<ReleaseRecord>,
    pub highest_manifest_sequence: u64,
    pub manifest_cache: Option<ManifestCacheRecord>,
    pub adoption_history: Vec<AdoptionRecord>,
}

impl State {
    pub fn new(installed_version: impl Into<String>) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        Self {
            schema_version: STATE_SCHEMA_VERSION,
            install_id: format!("{nanos:032x}-{:08x}", std::process::id()),
            installed_version: installed_version.into(),
            pre_install_user_path: None,
            shim_directory: None,
            path_entry_added: false,
            defaults: DefaultSelections::default(),
            native_targets: BTreeMap::new(),
            active_release: None,
            highest_manifest_sequence: 0,
            manifest_cache: None,
            adoption_history: Vec::new(),
        }
    }

    pub fn record_adoption(&mut self, record: AdoptionRecord) {
        self.adoption_history.push(record);
        if self.adoption_history.len() > ADOPTION_HISTORY_LIMIT {
            self.adoption_history
                .drain(..self.adoption_history.len() - ADOPTION_HISTORY_LIMIT);
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != STATE_SCHEMA_VERSION {
            return Err(CliEditorError::UnsupportedStateSchema {
                expected: STATE_SCHEMA_VERSION,
                found: self.schema_version,
            });
        }
        Ok(())
    }

    fn validate_owned_paths(&self, root: &Path) -> Result<()> {
        self.validate()?;
        let expected_shims = root.join("shims");
        let versions = root.join("versions");
        let compatibility = root.join("compatibility");
        for directory in [root, &expected_shims, &versions, &compatibility] {
            ensure_not_reparse(directory)?;
        }
        if let Some(shims) = &self.shim_directory
            && !same_owned_path(shims, &expected_shims)
        {
            return Err(CliEditorError::UnsafeTarget(shims.clone()));
        }
        if self.path_entry_added
            && (self.shim_directory.is_none() || self.pre_install_user_path.is_none())
        {
            return Err(CliEditorError::UnsafeTarget(expected_shims));
        }
        if let Some(release) = &self.active_release
            && !release
                .directory
                .parent()
                .is_some_and(|parent| same_owned_path(parent, &versions))
        {
            return Err(CliEditorError::UnsafeTarget(release.directory.clone()));
        }
        if let Some(release) = &self.active_release {
            ensure_not_reparse(&release.directory)?;
        }
        if let Some(cache) = &self.manifest_cache {
            if !same_owned_path(&cache.manifest_path, &compatibility.join("manifest.json")) {
                return Err(CliEditorError::UnsafeTarget(cache.manifest_path.clone()));
            }
            if !same_owned_path(&cache.signature_path, &compatibility.join("manifest.sig")) {
                return Err(CliEditorError::UnsafeTarget(cache.signature_path.clone()));
            }
        }
        Ok(())
    }
}

pub(crate) fn ensure_not_reparse(path: &Path) -> Result<()> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => return Err(CliEditorError::io(path, source)),
    };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(CliEditorError::UnsafeTarget(path.to_path_buf()));
        }
    }
    #[cfg(not(windows))]
    if metadata.file_type().is_symlink() {
        return Err(CliEditorError::UnsafeTarget(path.to_path_buf()));
    }
    Ok(())
}
fn same_owned_path(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    {
        fn display_path(path: &Path) -> String {
            let value = path.to_string_lossy();
            if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
                return format!(r"\\{rest}");
            }
            value.strip_prefix(r"\\?\").unwrap_or(&value).to_owned()
        }
        if display_path(left).eq_ignore_ascii_case(&display_path(right)) {
            return true;
        }
        let canonical_left = left.canonicalize().unwrap_or_else(|_| left.to_path_buf());
        let canonical_right = right.canonicalize().unwrap_or_else(|_| right.to_path_buf());
        display_path(&canonical_left).eq_ignore_ascii_case(&display_path(&canonical_right))
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryValueSnapshot {
    pub existed: bool,
    pub value_type: u32,
    pub data: Vec<u8>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultSelections {
    pub codex_enhanced: bool,
    pub claude_managed: bool,
    pub claude_strict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeTarget {
    pub path: PathBuf,
    pub package_root: PathBuf,
    pub package_identity: String,
    pub version: String,
    pub sha256: String,
    pub file_size: u64,
    pub modified_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseRecord {
    pub version: String,
    pub directory: PathBuf,
    pub codex_version: String,
    pub sha256: String,
    #[serde(default)]
    pub file_size: u64,
    #[serde(default)]
    pub modified_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestCacheRecord {
    pub manifest_path: PathBuf,
    pub signature_path: PathBuf,
    pub sequence: u64,
    pub expires_unix: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdoptionRecord {
    pub timestamp_unix_ms: u128,
    pub cli: CliKind,
    pub package_root: PathBuf,
    pub old_version: String,
    pub new_version: String,
    pub old_sha256: String,
    pub new_sha256: String,
}

#[derive(Debug, Clone)]
pub struct StateStore {
    root: PathBuf,
}

impl StateStore {
    pub fn for_current_user() -> Result<Self> {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .ok_or(CliEditorError::StateDirectoryUnavailable)?;
        Ok(Self::new(base.join("CLIEditor")))
    }

    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn load(&self) -> Result<Option<State>> {
        let path = self.state_path();
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => return Err(CliEditorError::io(&path, source)),
        };
        let state: State = serde_json::from_slice(&bytes)?;
        state.validate_owned_paths(&self.root)?;
        Ok(Some(state))
    }

    pub fn save(&self, state: &State) -> Result<()> {
        let state = state.clone();
        self.transaction(|_| Ok((state, ())))
    }

    pub fn transaction<T, F>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(Option<State>) -> Result<(State, T)>,
    {
        let _lock = self.lock(STATE_LOCK_TIMEOUT)?;
        let current = self.load()?;
        let (next, output) = operation(current)?;
        self.save_locked(&next)?;
        Ok(output)
    }

    pub fn remove_with<T, F>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(&State) -> Result<T>,
    {
        let _lock = self.lock(STATE_LOCK_TIMEOUT)?;
        let state = self.load()?.ok_or(CliEditorError::NotInstalled)?;
        let output = operation(&state)?;
        for path in [self.state_path(), self.root.join("state.backup.json")] {
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
                Err(source) => return Err(CliEditorError::io(&path, source)),
            }
        }
        Ok(output)
    }
    fn save_locked(&self, state: &State) -> Result<()> {
        state.validate_owned_paths(&self.root)?;
        std::fs::create_dir_all(&self.root)
            .map_err(|source| CliEditorError::io(&self.root, source))?;
        let target = self.state_path();
        let temp = self.root.join(format!(
            "state.{}.{:032x}.{}.tmp",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let bytes = serde_json::to_vec_pretty(state)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|source| CliEditorError::io(&temp, source))?;
        if let Err(source) = file.write_all(&bytes).and_then(|()| file.sync_all()) {
            let _ = std::fs::remove_file(&temp);
            return Err(CliEditorError::io(&temp, source));
        }
        drop(file);

        if target.exists() {
            let backup = self.root.join("state.backup.json");
            std::fs::copy(&target, &backup)
                .map_err(|source| CliEditorError::io(&backup, source))?;
        }
        replace_file(&temp, &target).inspect_err(|_| {
            let _ = std::fs::remove_file(&temp);
        })
    }

    fn lock(&self, timeout: Duration) -> Result<StateLock> {
        std::fs::create_dir_all(&self.root)
            .map_err(|source| CliEditorError::io(&self.root, source))?;
        ensure_not_reparse(&self.root)?;
        let path = self.root.join("state.lock");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|source| CliEditorError::io(&path, source))?;
        let started = Instant::now();
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => return Ok(StateLock { file }),
                Err(source) if is_lock_contended(&source) => {
                    if started.elapsed() >= timeout {
                        return Err(CliEditorError::LockTimeout);
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(source) => return Err(CliEditorError::io(&path, source)),
            }
        }
    }

    fn state_path(&self) -> PathBuf {
        self.root.join("state.json")
    }
}

fn is_lock_contended(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::WouldBlock
        || cfg!(windows) && matches!(error.raw_os_error(), Some(32 | 33))
}

#[derive(Debug)]
pub struct StateLock {
    file: File,
}

impl Drop for StateLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

#[cfg(windows)]
pub(crate) fn replace_file(source: &Path, target: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winbase::MOVEFILE_REPLACE_EXISTING;
    use winapi::um::winbase::MOVEFILE_WRITE_THROUGH;
    use winapi::um::winbase::MoveFileExW;

    let source_wide: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let target_wide: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
    let result = unsafe {
        MoveFileExW(
            source_wide.as_ptr(),
            target_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        return Err(CliEditorError::io(target, std::io::Error::last_os_error()));
    }
    Ok(())
}

#[cfg(not(windows))]
pub(crate) fn replace_file(source: &Path, target: &Path) -> Result<()> {
    std::fs::rename(source, target).map_err(|source| CliEditorError::io(target, source))
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
