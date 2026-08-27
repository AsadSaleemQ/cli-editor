use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::CliKind;
use crate::compatibility::{Freshness, VerifiedManifest, verify_manifest};
use crate::discovery::{sha256_file, validate_recorded_target_identity};
use crate::error::{CliEditorError, Result};
use crate::process::run_native;
use crate::state::{NativeTarget, State, StateStore};
use crate::version::normalized_version;

pub(crate) fn invocation_kind(executable: &Path) -> Option<CliKind> {
    let stem = executable.file_stem()?.to_str()?;
    if stem.eq_ignore_ascii_case("codex") {
        Some(CliKind::Codex)
    } else if stem.eq_ignore_ascii_case("claude") {
        Some(CliKind::Claude)
    } else {
        None
    }
}

pub(crate) fn run_shim(kind: CliKind, args: Vec<OsString>) -> Result<i32> {
    let (explicit, args) = parse_shim_args(args);
    run_managed(kind, args, explicit)
}

fn parse_shim_args(mut args: Vec<OsString>) -> (bool, Vec<OsString>) {
    let explicit = args
        .first()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("cli-editor"));
    if explicit {
        args.remove(0);
        if args.first().is_some_and(|value| value == "--") {
            args.remove(0);
        }
    }
    (explicit, args)
}
pub(crate) fn run_managed(kind: CliKind, args: Vec<OsString>, explicit: bool) -> Result<i32> {
    let store = StateStore::for_current_user()?;
    let state = store.load()?.ok_or(CliEditorError::NotInstalled)?;
    let (state, native_validated, force_native) =
        revalidate_native_target(&store, state, kind, explicit)?;
    let enhanced_allowed = !force_native && compatibility_allows(&state, kind, explicit)?;
    let target = select_target(&state, kind, explicit, enhanced_allowed)?;
    let target = resolve_validated_target(&state, kind, target, explicit, native_validated)?;
    reject_shim_target(&state, &target)?;
    run_native(&target, &args)
}

fn revalidate_native_target(
    store: &StateStore,
    state: State,
    kind: CliKind,
    explicit: bool,
) -> Result<(State, bool, bool)> {
    revalidate_native_target_with(
        store,
        state,
        kind,
        explicit,
        crate::installer::adopt_in_place,
    )
}

fn revalidate_native_target_with<F>(
    store: &StateStore,
    state: State,
    kind: CliKind,
    explicit: bool,
    adopt: F,
) -> Result<(State, bool, bool)>
where
    F: FnOnce(CliKind, &NativeTarget) -> Result<NativeTarget>,
{
    let Some(recorded) = state.native_targets.get(&kind).cloned() else {
        return Ok((state, false, false));
    };
    match validate_native_metadata(kind, &recorded) {
        Ok(()) => Ok((state, true, false)),
        Err(CliEditorError::TargetChanged(_)) => match adopt(kind, &recorded) {
            Ok(_) => Ok((
                store.load()?.ok_or(CliEditorError::NotInstalled)?,
                true,
                false,
            )),
            Err(CliEditorError::StateChangedDuringOperation) => {
                let refreshed = store.load()?.ok_or(CliEditorError::NotInstalled)?;
                let refreshed_target = refreshed
                    .native_targets
                    .get(&kind)
                    .ok_or(CliEditorError::TargetNotFound(kind))?;
                validate_native_metadata(kind, refreshed_target)?;
                Ok((refreshed, true, false))
            }
            Err(error @ CliEditorError::VersionProbeTimedOut { .. })
                if native_timeout_can_fallback(kind, explicit) =>
            {
                validate_recorded_target_identity(kind, &recorded)?;
                eprintln!(
                    "warning: native {} update version probe timed out; launching the identity-validated native target: {error}",
                    kind.as_str()
                );
                Ok((state, true, true))
            }
            Err(error) => Err(error),
        },
        Err(error) => Err(error),
    }
}

fn native_timeout_can_fallback(kind: CliKind, explicit: bool) -> bool {
    kind == CliKind::Claude || !explicit
}

fn reject_shim_target(state: &State, target: &Path) -> Result<()> {
    let canonical_target = target
        .canonicalize()
        .map_err(|source| CliEditorError::io(target, source))?;
    if let Some(shim_directory) = &state.shim_directory
        && let Ok(canonical_shims) = shim_directory.canonicalize()
        && canonical_target.starts_with(canonical_shims)
    {
        return Err(CliEditorError::RecursionDetected);
    }
    if let Ok(current) = std::env::current_exe().and_then(|path| path.canonicalize())
        && canonical_target == current
    {
        return Err(CliEditorError::RecursionDetected);
    }
    Ok(())
}
fn compatibility_allows(state: &State, kind: CliKind, explicit: bool) -> Result<bool> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let host_version = std::env::var("TERM_PROGRAM_VERSION").ok();
    compatibility_allows_at(state, kind, explicit, now, host_version.as_deref())
}

fn compatibility_allows_at(
    state: &State,
    kind: CliKind,
    explicit: bool,
    now: u64,
    host_version: Option<&str>,
) -> Result<bool> {
    let codex_requested = kind == CliKind::Codex && (explicit || state.defaults.codex_enhanced);
    let claude_managed = kind == CliKind::Claude && (explicit || state.defaults.claude_managed);
    let claude_strict = claude_managed && state.defaults.claude_strict;
    if !codex_requested && !claude_managed {
        return Ok(false);
    }
    let verified = match cached_manifest_at(state, now) {
        Ok(manifest) => manifest,
        Err(error) if codex_requested && !explicit => {
            eprintln!("warning: enhanced Codex disabled; {error}");
            return Ok(false);
        }
        Err(error) if claude_managed && !claude_strict => {
            eprintln!(
                "warning: managed Claude compatibility check unavailable; launching verified native Claude: {error}"
            );
            return Ok(false);
        }
        Err(error) => return Err(error),
    };
    if verified.freshness == Freshness::Expired {
        if codex_requested && !explicit {
            eprintln!("warning: enhanced Codex manifest expired; launching verified native Codex");
            return Ok(false);
        }
        if claude_managed && !claude_strict {
            eprintln!("warning: managed Claude manifest expired; launching verified native Claude");
            return Ok(false);
        }
        return Err(CliEditorError::ManifestExpired);
    }
    if matches!(verified.freshness, Freshness::Grace { .. }) {
        eprintln!("warning: CLI Editor compatibility manifest is stale but within grace");
    }
    if codex_requested {
        let native_version = state
            .native_targets
            .get(&CliKind::Codex)
            .map(|target| normalized_version(&target.version))
            .ok_or(CliEditorError::TargetNotFound(CliKind::Codex))?;
        if !verified.manifest.supports_codex(native_version) {
            if !explicit {
                eprintln!(
                    "warning: enhanced Codex is not validated with native Codex {native_version}; launching verified native Codex"
                );
                return Ok(false);
            }
            return Err(CliEditorError::UnsupportedCodexVersion(
                native_version.into(),
            ));
        }
    }
    if claude_managed {
        let version = state
            .native_targets
            .get(&kind)
            .map(|target| normalized_version(&target.version))
            .unwrap_or("unknown");
        if !verified.manifest.supports_claude(version) {
            if claude_strict {
                return Err(CliEditorError::StrictClaudeUnvalidated(version.into()));
            }
            eprintln!(
                "warning: Claude {version} is not in the signed compatibility manifest; launching verified native Claude"
            );
        }
        return Ok(false);
    }
    let release = state
        .active_release
        .as_ref()
        .ok_or(CliEditorError::EnhancedUnavailable)?;
    if !verified.manifest.supports_codex(&release.codex_version) {
        if explicit {
            return Err(CliEditorError::UnsupportedCodexVersion(
                release.codex_version.clone(),
            ));
        }
        eprintln!(
            "warning: enhanced Codex is not compatible with this Codex version; launching native Codex"
        );
        return Ok(false);
    }
    if let Some(vscode) = host_version
        && !verified.manifest.supports(&release.codex_version, vscode)
    {
        eprintln!(
            "warning: VS Code {vscode} is newer than the signed host validation set; continuing because host drift is non-fatal"
        );
    }
    Ok(true)
}

fn cached_manifest_at(state: &State, now: u64) -> Result<VerifiedManifest> {
    let cache = state
        .manifest_cache
        .as_ref()
        .ok_or(CliEditorError::EnhancedUnavailable)?;
    let bytes = std::fs::read(&cache.manifest_path)
        .map_err(|source| CliEditorError::io(&cache.manifest_path, source))?;
    let signature = std::fs::read_to_string(&cache.signature_path)
        .map_err(|source| CliEditorError::io(&cache.signature_path, source))?;
    if cache.sequence > state.highest_manifest_sequence {
        return Err(CliEditorError::ManifestCacheMismatch);
    }
    let verified = verify_manifest(&bytes, &signature, cache.sequence, now)?;
    if verified.manifest.sequence != cache.sequence {
        return Err(CliEditorError::ManifestCacheMismatch);
    }
    Ok(verified)
}

fn select_target(
    state: &State,
    kind: CliKind,
    explicit: bool,
    enhanced_allowed: bool,
) -> Result<PathBuf> {
    let enhanced =
        kind == CliKind::Codex && (explicit || state.defaults.codex_enhanced) && enhanced_allowed;
    if enhanced {
        let release = state
            .active_release
            .as_ref()
            .ok_or(CliEditorError::EnhancedUnavailable)?;
        return Ok(release.directory.join("codex.exe"));
    }
    state
        .native_targets
        .get(&kind)
        .map(|target| target.path.clone())
        .ok_or(CliEditorError::TargetNotFound(kind))
}

fn resolve_validated_target(
    state: &State,
    kind: CliKind,
    path: PathBuf,
    explicit: bool,
    native_validated: bool,
) -> Result<PathBuf> {
    let native = state.native_targets.get(&kind).map(|target| &target.path);
    if native_validated && native == Some(&path) {
        return Ok(path);
    }
    match validate_launch_target(state, kind, &path) {
        Ok(()) => Ok(path),
        Err(CliEditorError::ArtifactVerificationFailed(path)) if !explicit => {
            eprintln!(
                "warning: enhanced Codex failed integrity validation at {}; launching verified native Codex",
                path.display()
            );
            let native = state
                .native_targets
                .get(&kind)
                .map(|target| target.path.clone())
                .ok_or(CliEditorError::TargetNotFound(kind))?;
            if native_validated {
                Ok(native)
            } else {
                validate_launch_target(state, kind, &native)?;
                Ok(native)
            }
        }
        Err(error) => Err(error),
    }
}
fn validate_launch_target(state: &State, kind: CliKind, path: &Path) -> Result<()> {
    if kind == CliKind::Codex
        && let Some(release) = state.active_release.as_ref()
    {
        let enhanced_path = release.directory.join("codex.exe");
        if path == enhanced_path {
            let metadata = path
                .metadata()
                .map_err(|_| CliEditorError::ArtifactVerificationFailed(path.to_path_buf()))?;
            let modified = metadata_modified_unix_ms(&metadata);
            if release.file_size != 0
                && metadata.len() == release.file_size
                && modified == release.modified_unix_ms
            {
                return Ok(());
            }
            if sha256_file(path).ok().as_deref() == Some(&release.sha256) {
                return Ok(());
            }
            return Err(CliEditorError::ArtifactVerificationFailed(
                path.to_path_buf(),
            ));
        }
    }
    let target = state
        .native_targets
        .get(&kind)
        .ok_or(CliEditorError::TargetNotFound(kind))?;
    validate_native_metadata(kind, target)
}

fn metadata_modified_unix_ms(metadata: &std::fs::Metadata) -> u128 {
    metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |value| value.as_millis())
}
pub(crate) fn validate_native_metadata(kind: CliKind, target: &NativeTarget) -> Result<()> {
    let path = target.path.canonicalize().map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            CliEditorError::NativeTargetMissing {
                kind,
                path: target.path.clone(),
            }
        } else {
            CliEditorError::io(&target.path, source)
        }
    })?;
    let root = target
        .package_root
        .canonicalize()
        .map_err(|source| CliEditorError::io(&target.package_root, source))?;
    if path != target.path || !path.starts_with(root) || !has_exe_extension(&path) {
        return Err(CliEditorError::TargetChanged(target.path.clone()));
    }
    let metadata = path
        .metadata()
        .map_err(|source| CliEditorError::io(&path, source))?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |value| value.as_millis());
    if metadata.len() != target.file_size || modified != target.modified_unix_ms {
        return Err(CliEditorError::TargetChanged(path));
    }
    Ok(())
}

fn has_exe_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("exe"))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use ed25519_dalek::{Signer, SigningKey};

    use crate::CliKind;
    use crate::compatibility::{CompatibilityEntry, CompatibilityManifest};
    use crate::state::{
        DefaultSelections, ManifestCacheRecord, NativeTarget, ReleaseRecord, State,
    };

    use super::{
        compatibility_allows_at, invocation_kind, parse_shim_args, reject_shim_target,
        select_target,
    };

    fn state() -> State {
        State {
            schema_version: 1,
            install_id: "test".into(),
            installed_version: "0.1.0".into(),
            pre_install_user_path: None,
            shim_directory: None,
            path_entry_added: false,
            vscode_extension_added: false,
            defaults: DefaultSelections::default(),
            native_targets: BTreeMap::new(),
            active_release: Some(ReleaseRecord {
                version: "0.1.0".into(),
                directory: PathBuf::from(r"C:\release"),
                codex_version: "0.148.0".into(),
                sha256: "abc".into(),
                file_size: 0,
                modified_unix_ms: 0,
            }),
            highest_manifest_sequence: 0,
            manifest_cache: None,
            adoption_history: Vec::new(),
        }
    }

    #[test]
    fn detects_shim_name_case_insensitively() {
        assert_eq!(
            invocation_kind(Path::new(r"C:\shim\CODEX.exe")),
            Some(CliKind::Codex)
        );
        assert_eq!(
            invocation_kind(Path::new(r"C:\shim\claude.exe")),
            Some(CliKind::Claude)
        );
        assert_eq!(invocation_kind(Path::new(r"C:\shim\cli-editor.exe")), None);
    }

    #[test]
    fn recursion_guard_rejects_only_owned_shim_targets() {
        let directory = crate::test_support::TempDir::new();
        let shim_directory = directory.path().join("shims");
        let native_directory = directory.path().join("native");
        std::fs::create_dir(&shim_directory).unwrap();
        std::fs::create_dir(&native_directory).unwrap();
        let shim = shim_directory.join("codex.exe");
        let native = native_directory.join("codex.exe");
        std::fs::write(&shim, b"shim").unwrap();
        std::fs::write(&native, b"native").unwrap();
        let mut test_state = state();
        test_state.shim_directory = Some(shim_directory.canonicalize().unwrap());

        assert!(reject_shim_target(&test_state, &shim).is_err());
        assert!(reject_shim_target(&test_state, &native).is_ok());
    }
    #[test]
    fn explicit_codex_selects_enhanced_release() {
        assert_eq!(
            select_target(&state(), CliKind::Codex, true, true).unwrap(),
            PathBuf::from(r"C:\release\codex.exe")
        );
    }

    #[test]
    fn corrupt_enhanced_release_falls_back_only_for_default_mode() {
        let directory = crate::test_support::TempDir::new();
        let root = directory.path().canonicalize().unwrap();
        let native_path = root.join("native-codex.exe");
        std::fs::write(&native_path, b"verified native").unwrap();
        let metadata = native_path.metadata().unwrap();
        let modified = metadata
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let release_dir = root.join("release");
        std::fs::create_dir(&release_dir).unwrap();
        let enhanced_path = release_dir.join("codex.exe");
        std::fs::write(&enhanced_path, b"original enhanced").unwrap();
        let expected_hash = crate::discovery::sha256_file(&enhanced_path).unwrap();

        let mut test_state = state();
        test_state.active_release = Some(ReleaseRecord {
            version: "0.1.0".into(),
            directory: release_dir,
            codex_version: "0.148.0".into(),
            sha256: expected_hash,
            file_size: 0,
            modified_unix_ms: 0,
        });
        test_state.native_targets.insert(
            CliKind::Codex,
            crate::NativeTarget {
                path: native_path.clone(),
                package_root: root,
                package_identity: "codex:@openai/codex@0.148.0".into(),
                version: "codex-cli 0.148.0".into(),
                sha256: crate::discovery::sha256_file(&native_path).unwrap(),
                file_size: metadata.len(),
                modified_unix_ms: modified,
            },
        );
        std::fs::write(&enhanced_path, b"corrupt enhanced").unwrap();

        assert_eq!(
            super::resolve_validated_target(
                &test_state,
                CliKind::Codex,
                enhanced_path.clone(),
                false,
                true,
            )
            .unwrap(),
            native_path
        );
        assert!(matches!(
            super::resolve_validated_target(
                &test_state,
                CliKind::Codex,
                enhanced_path.clone(),
                true,
                true,
            ),
            Err(crate::CliEditorError::ArtifactVerificationFailed(path)) if path == enhanced_path
        ));
    }
    #[test]
    fn changed_native_target_falls_back_after_probe_timeout_without_reprobing() {
        let directory = crate::test_support::TempDir::new();
        let package_root = directory.path().canonicalize().unwrap();
        let native_path = package_root.join("claude.exe");
        std::fs::write(&native_path, b"updated native").unwrap();
        let metadata = native_path.metadata().unwrap();
        let mut test_state = state();
        test_state.native_targets.insert(
            CliKind::Claude,
            NativeTarget {
                path: native_path.clone(),
                package_root,
                package_identity: "claude:native-executable".into(),
                version: "2.1.240 (Claude Code)".into(),
                sha256: "stale".into(),
                file_size: metadata.len() + 1,
                modified_unix_ms: 0,
            },
        );
        let store = crate::state::StateStore::new(directory.path().join("unused-store"));

        let (returned, native_validated, force_native) = super::revalidate_native_target_with(
            &store,
            test_state,
            CliKind::Claude,
            true,
            |_, target| {
                Err(crate::CliEditorError::VersionProbeTimedOut {
                    path: target.path.clone(),
                    timeout_seconds: 60,
                })
            },
        )
        .unwrap();

        assert!(native_validated);
        assert!(force_native);
        assert_eq!(returned.native_targets[&CliKind::Claude].path, native_path);
    }

    #[test]
    fn missing_native_target_uses_the_actionable_launcher_error() {
        let directory = crate::test_support::TempDir::new();
        let package_root = directory.path().canonicalize().unwrap();
        let missing = package_root.join("claude.exe");
        let target = NativeTarget {
            path: missing.clone(),
            package_root,
            package_identity: "claude:native-executable".into(),
            version: "2.1.240 (Claude Code)".into(),
            sha256: "missing".into(),
            file_size: 0,
            modified_unix_ms: 0,
        };

        assert!(matches!(
            super::validate_native_metadata(CliKind::Claude, &target),
            Err(crate::CliEditorError::NativeTargetMissing {
                kind: CliKind::Claude,
                path,
            }) if path == missing
        ));
    }

    #[test]
    fn concurrent_native_adoption_reloads_the_winning_state() {
        let directory = crate::test_support::TempDir::new();
        let package_root = directory.path().join("native");
        std::fs::create_dir(&package_root).unwrap();
        let package_root = package_root.canonicalize().unwrap();
        let native_path = package_root.join("claude.exe");
        std::fs::write(&native_path, b"updated native").unwrap();
        let metadata = native_path.metadata().unwrap();
        let modified_unix_ms = metadata
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let adopted = NativeTarget {
            path: native_path.clone(),
            package_root,
            package_identity: "claude:native-executable".into(),
            version: "2.1.241 (Claude Code)".into(),
            sha256: crate::discovery::sha256_file(&native_path).unwrap(),
            file_size: metadata.len(),
            modified_unix_ms,
        };
        let mut stale = state();
        stale.active_release = None;
        stale.native_targets.insert(
            CliKind::Claude,
            NativeTarget {
                file_size: adopted.file_size + 1,
                ..adopted.clone()
            },
        );
        let mut winning = stale.clone();
        winning
            .native_targets
            .insert(CliKind::Claude, adopted.clone());
        let store = crate::state::StateStore::new(directory.path().join("store"));
        store.save(&winning).unwrap();

        let (returned, native_validated, force_native) =
            super::revalidate_native_target_with(&store, stale, CliKind::Claude, true, |_, _| {
                Err(crate::CliEditorError::StateChangedDuringOperation)
            })
            .unwrap();

        assert!(native_validated);
        assert!(!force_native);
        assert_eq!(returned.native_targets[&CliKind::Claude], adopted);
    }

    #[test]
    fn native_probe_timeout_fallback_preserves_explicit_codex_contract() {
        assert!(super::native_timeout_can_fallback(CliKind::Codex, false));
        assert!(super::native_timeout_can_fallback(CliKind::Claude, false));
        assert!(super::native_timeout_can_fallback(CliKind::Claude, true));
        assert!(!super::native_timeout_can_fallback(CliKind::Codex, true));
    }
    fn state_with_manifest(
        compatibility: Vec<CompatibilityEntry>,
        expires_unix: u64,
    ) -> (crate::test_support::TempDir, State) {
        let directory = crate::test_support::TempDir::new();
        let manifest_path = directory.path().join("manifest.json");
        let signature_path = directory.path().join("manifest.sig");
        let manifest = CompatibilityManifest {
            schema_version: 1,
            sequence: 1,
            issued_unix: 100,
            expires_unix,
            minimum_dispatcher_version: "0.1.0".into(),
            compatibility,
            artifacts: Vec::new(),
        };
        let bytes = serde_json::to_vec(&manifest).unwrap();
        let seed: [u8; 32] =
            hex::decode("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
                .unwrap()
                .try_into()
                .unwrap();
        let signature = SigningKey::from_bytes(&seed).sign(&bytes);
        std::fs::write(&manifest_path, bytes).unwrap();
        std::fs::write(&signature_path, hex::encode(signature.to_bytes())).unwrap();

        let mut test_state = state();
        test_state.highest_manifest_sequence = 1;
        test_state.manifest_cache = Some(ManifestCacheRecord {
            manifest_path,
            signature_path,
            sequence: 1,
            expires_unix,
        });
        for (kind, version) in [
            (CliKind::Codex, "codex-cli 0.148.0"),
            (CliKind::Claude, "2.1.240 (Claude Code)"),
        ] {
            test_state.native_targets.insert(
                kind,
                NativeTarget {
                    path: PathBuf::from(format!(r"C:\native\{}.exe", kind.as_str())),
                    package_root: PathBuf::from(r"C:\native"),
                    package_identity: format!("{}:official", kind.as_str()),
                    version: version.into(),
                    sha256: "00".repeat(32),
                    file_size: 1,
                    modified_unix_ms: 1,
                },
            );
        }
        (directory, test_state)
    }

    #[test]
    fn shim_token_parsing_preserves_arguments_and_literal_escape() {
        let (explicit, args) = parse_shim_args(vec![
            OsString::from("CLI-EDITOR"),
            OsString::from("--"),
            OsString::from("prompt"),
        ]);
        assert!(explicit);
        assert_eq!(args, vec![OsString::from("prompt")]);

        let escaped = vec![OsString::from("--"), OsString::from("cli-editor")];
        let (explicit, args) = parse_shim_args(escaped.clone());
        assert!(!explicit);
        assert_eq!(args, escaped);
    }

    #[test]
    fn explicit_claude_warns_and_forwards_when_validation_is_unavailable() {
        let entry = CompatibilityEntry {
            codex: "0.148.0".into(),
            vscode: vec!["1.134.0".into()],
            claude: vec!["2.1.240".into()],
        };
        let (_directory, state) = state_with_manifest(vec![entry], 500);
        std::fs::write(
            &state.manifest_cache.as_ref().unwrap().signature_path,
            "invalid",
        )
        .unwrap();

        assert!(!compatibility_allows_at(&state, CliKind::Claude, false, 200, None).unwrap());
        assert!(!compatibility_allows_at(&state, CliKind::Claude, true, 200, None).unwrap());
    }

    #[test]
    fn explicit_claude_warns_and_forwards_an_unlisted_version() {
        let entry = CompatibilityEntry {
            codex: "0.148.0".into(),
            vscode: vec!["1.134.0".into()],
            claude: vec!["2.0.0".into()],
        };
        let (_directory, state) = state_with_manifest(vec![entry], 500);

        assert!(!compatibility_allows_at(&state, CliKind::Claude, true, 200, None).unwrap());
    }

    #[test]
    fn strict_claude_rejects_unlisted_version() {
        let entry = CompatibilityEntry {
            codex: "0.148.0".into(),
            vscode: vec!["1.134.0".into()],
            claude: vec!["2.0.0".into()],
        };
        let (_directory, mut state) = state_with_manifest(vec![entry], 500);
        state.defaults.claude_managed = true;
        state.defaults.claude_strict = true;

        assert!(matches!(
            compatibility_allows_at(&state, CliKind::Claude, true, 200, None),
            Err(crate::CliEditorError::StrictClaudeUnvalidated(_))
        ));
    }

    #[test]
    fn vscode_drift_warns_but_does_not_disable_explicit_codex() {
        let entry = CompatibilityEntry {
            codex: "0.148.0".into(),
            vscode: vec!["1.134.0".into()],
            claude: vec!["2.1.240".into()],
        };
        let (_directory, state) = state_with_manifest(vec![entry], 500);

        assert!(
            compatibility_allows_at(&state, CliKind::Codex, true, 200, Some("1.200.0")).unwrap()
        );
    }

    #[test]
    fn unsupported_codex_falls_back_only_for_default_mode() {
        let entry = CompatibilityEntry {
            codex: "0.147.0".into(),
            vscode: vec!["1.134.0".into()],
            claude: vec!["2.1.240".into()],
        };
        let (_directory, mut state) = state_with_manifest(vec![entry], 500);
        state.defaults.codex_enhanced = true;

        assert!(!compatibility_allows_at(&state, CliKind::Codex, false, 200, None).unwrap());
        assert!(matches!(
            compatibility_allows_at(&state, CliKind::Codex, true, 200, None),
            Err(crate::CliEditorError::UnsupportedCodexVersion(_))
        ));
    }

    #[test]
    fn stale_manifest_within_grace_still_allows_enhanced_codex() {
        let entry = CompatibilityEntry {
            codex: "0.148.0".into(),
            vscode: vec!["1.134.0".into()],
            claude: vec!["2.1.240".into()],
        };
        let (_directory, state) = state_with_manifest(vec![entry], 199);

        assert!(compatibility_allows_at(&state, CliKind::Codex, true, 200, None).unwrap());
    }
}
