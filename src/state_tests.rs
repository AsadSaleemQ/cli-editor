use std::time::Duration;

use pretty_assertions::assert_eq;

use super::ADOPTION_HISTORY_LIMIT;
use super::AdoptionRecord;
use super::ReleaseRecord;
use super::State;
use super::StateStore;
use crate::CliEditorError;
use crate::CliKind;

#[test]
fn state_round_trips_and_keeps_backup() {
    let temp = crate::test_support::TempDir::new();
    let store = StateStore::new(temp.path().to_path_buf());
    let mut state = State::new("0.1.0");
    store.save(&state).expect("initial save");
    state.defaults.codex_enhanced = true;
    store.save(&state).expect("second save");

    assert_eq!(store.load().expect("load"), Some(state));
    assert!(temp.path().join("state.backup.json").exists());
}

#[test]
fn adoption_history_is_bounded() {
    let mut state = State::new("0.1.0");
    for index in 0..(ADOPTION_HISTORY_LIMIT + 3) {
        state.record_adoption(AdoptionRecord {
            timestamp_unix_ms: index as u128,
            cli: CliKind::Codex,
            package_root: "package".into(),
            old_version: index.to_string(),
            new_version: (index + 1).to_string(),
            old_sha256: "old".into(),
            new_sha256: "new".into(),
        });
    }

    assert_eq!(state.adoption_history.len(), ADOPTION_HISTORY_LIMIT);
    assert_eq!(state.adoption_history[0].timestamp_unix_ms, 3);
}

#[test]
fn state_lock_times_out_while_held() {
    let temp = crate::test_support::TempDir::new();
    let store = StateStore::new(temp.path().to_path_buf());
    let _lock = store.lock(Duration::from_millis(50)).expect("first lock");

    let error = store
        .lock(Duration::from_millis(20))
        .expect_err("second lock should time out");
    assert!(matches!(error, CliEditorError::LockTimeout));
}

#[test]
fn transaction_reads_mutates_and_writes_under_one_lock() {
    let temp = crate::test_support::TempDir::new();
    let store = StateStore::new(temp.path().to_path_buf());
    store
        .transaction(|current| {
            assert!(current.is_none());
            Ok((State::new("0.1.0"), "created"))
        })
        .expect("create transaction");
    let value = store
        .transaction(|current| {
            let mut state = current.expect("existing state");
            state.installed_version = "0.1.1".into();
            Ok((state, 42))
        })
        .expect("update transaction");
    assert_eq!(value, 42);
    assert_eq!(store.load().unwrap().unwrap().installed_version, "0.1.1");
}

#[test]
fn rejects_unsupported_schema_version() {
    let mut state = State::new("0.1.0");
    state.schema_version = 99;
    assert!(matches!(
        state.validate(),
        Err(crate::CliEditorError::UnsupportedStateSchema {
            expected: 1,
            found: 99
        })
    ));
}

#[test]
fn serializes_cli_kind_map_keys_and_manifest_cache() {
    let temp = crate::test_support::TempDir::new();
    let store = StateStore::new(temp.path().to_path_buf());
    let mut state = State::new("0.1.0");
    state.native_targets.insert(
        crate::CliKind::Codex,
        crate::NativeTarget {
            path: std::path::PathBuf::from(r"C:\codex.exe"),
            package_root: std::path::PathBuf::from(r"C:\"),
            package_identity: "codex:test".into(),
            version: "0.148.0".into(),
            sha256: "abc".into(),
            file_size: 1,
            modified_unix_ms: 2,
        },
    );
    state.manifest_cache = Some(crate::ManifestCacheRecord {
        manifest_path: store.root().join("compatibility").join("manifest.json"),
        signature_path: store.root().join("compatibility").join("manifest.sig"),
        sequence: 7,
        expires_unix: 9,
    });
    store.save(&state).unwrap();
    assert_eq!(store.load().unwrap(), Some(state));
}
#[test]
fn rejects_tampered_owned_paths() {
    let temp = crate::test_support::TempDir::new();
    let store = StateStore::new(temp.path().join("CLIEditor"));

    let mut state = State::new("0.1.0");
    state.shim_directory = Some(temp.path().join("outside-shims"));
    assert!(matches!(
        store.save(&state),
        Err(CliEditorError::UnsafeTarget(path)) if path == temp.path().join("outside-shims")
    ));

    let mut state = State::new("0.1.0");
    state.path_entry_added = true;
    assert!(matches!(
        store.save(&state),
        Err(CliEditorError::UnsafeTarget(path)) if path == store.root().join("shims")
    ));

    let mut state = State::new("0.1.0");
    state.active_release = Some(ReleaseRecord {
        version: "release-1".into(),
        directory: temp.path().join("outside-release"),
        codex_version: "0.148.0".into(),
        sha256: "abc".into(),
        file_size: 1,
        modified_unix_ms: 2,
    });
    assert!(matches!(
        store.save(&state),
        Err(CliEditorError::UnsafeTarget(path)) if path == temp.path().join("outside-release")
    ));

    let mut state = State::new("0.1.0");
    state.manifest_cache = Some(crate::ManifestCacheRecord {
        manifest_path: temp.path().join("outside-manifest.json"),
        signature_path: store.root().join("compatibility").join("manifest.sig"),
        sequence: 1,
        expires_unix: 2,
    });
    assert!(matches!(
        store.save(&state),
        Err(CliEditorError::UnsafeTarget(path)) if path == temp.path().join("outside-manifest.json")
    ));
}
#[cfg(windows)]
#[test]
fn rejects_reparse_point_owned_directory() {
    let temp = crate::test_support::TempDir::new();
    let target = temp.path().join("junction-target");
    let link = temp.path().join("junction-link");
    std::fs::create_dir(&target).expect("junction target");
    let status = std::process::Command::new("cmd.exe")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(&link)
        .arg(&target)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("create junction");
    assert!(
        status.success(),
        "junction creation must succeed on Windows"
    );
    let result = super::ensure_not_reparse(&link);
    std::fs::remove_dir(&link).expect("remove junction without traversing it");
    assert!(matches!(
        result,
        Err(CliEditorError::UnsafeTarget(path)) if path == link
    ));
}
