use std::io::Write;
use std::path::Path;

use pretty_assertions::assert_eq;

use super::MAX_VERSION_OUTPUT_BYTES;
use super::NATIVE_VERSION_PROBE_TIMEOUT;
use super::ProbeOutput;
use super::RELEASE_VERSION_PROBE_TIMEOUT;
use super::resolve_candidate;
use super::same_package_identity;
use super::sha256_file;
use crate::CliKind;
use crate::DiscoveryOptions;
use crate::discover_native;

#[test]
fn missing_recorded_target_uses_the_actionable_launcher_error() {
    let temp = crate::test_support::TempDir::new();
    let package_root = temp.path().canonicalize().unwrap();
    let missing = package_root.join("codex.exe");
    let recorded = crate::NativeTarget {
        path: missing.clone(),
        package_root,
        package_identity: "codex:native-executable".into(),
        version: "codex-cli 0.148.0".into(),
        sha256: "missing".into(),
        file_size: 0,
        modified_unix_ms: 0,
    };

    assert!(matches!(
        super::validate_recorded_target_identity(&recorded),
        Err(crate::CliEditorError::NativeTargetMissing {
            kind: CliKind::Codex,
            path,
        }) if path == missing
    ));
}

#[test]
fn resolves_npm_codex_shim_to_native_executable() {
    let temp = crate::test_support::TempDir::new();
    let npm_root = temp.path();
    let shim = npm_root.join("codex.cmd");
    std::fs::write(&shim, "@echo off").expect("shim");
    let package_root = npm_root.join("node_modules").join("@openai").join("codex");
    std::fs::create_dir_all(&package_root).expect("package root");
    std::fs::write(
        package_root.join("package.json"),
        r#"{"name":"@openai/codex","version":"0.148.0"}"#,
    )
    .expect("package json");
    let native = package_root
        .join("node_modules")
        .join("@openai")
        .join("codex-win32-x64")
        .join("vendor")
        .join("x86_64-pc-windows-msvc")
        .join("bin")
        .join("codex.exe");
    std::fs::create_dir_all(native.parent().expect("native parent")).expect("native parent");
    std::fs::write(&native, "binary").expect("native file");

    let resolved = resolve_candidate(&shim)
        .expect("resolution")
        .expect("candidate");

    assert_eq!(resolved.path, native);
    assert_eq!(resolved.package_root, package_root);
    assert_eq!(resolved.identity, "@openai/codex@0.148.0");
}

#[test]
fn cold_native_and_release_probe_budgets_allow_security_scanning() {
    assert_eq!(NATIVE_VERSION_PROBE_TIMEOUT.as_secs(), 60);
    assert_eq!(RELEASE_VERSION_PROBE_TIMEOUT.as_secs(), 60);
}

#[test]
fn sha256_is_stable() {
    let temp = crate::test_support::TempDir::new();
    let path = temp.path().join("sample.exe");
    std::fs::write(&path, "cli-editor").expect("sample");

    assert_eq!(
        sha256_file(Path::new(&path)).expect("hash"),
        "e0744232290a08ac20392b23ea9aaf95c59657e817aa2840cc064e78f62c3d92"
    );
}

#[test]
fn reports_actionable_error_for_npm_named_launcher_without_package() {
    let temp = crate::test_support::TempDir::new();
    let script = temp.path().join("codex.cmd");
    std::fs::write(&script, "@echo off").expect("script");

    assert!(matches!(
        resolve_candidate(&script),
        Err(crate::CliEditorError::UnsupportedLauncher(path)) if path == script
    ));
}

#[test]
fn rejects_unsafe_first_path_match_instead_of_skipping_it() {
    let temp = crate::test_support::TempDir::new();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    std::fs::create_dir(&first).expect("first directory");
    std::fs::create_dir(&second).expect("second directory");
    std::fs::write(first.join("codex.bat"), "@echo unsafe").expect("unsafe wrapper");
    std::fs::write(second.join("codex.exe"), "later executable").expect("later executable");
    let options = DiscoveryOptions {
        path: std::env::join_paths([&first, &second]).expect("PATH"),
        current_dir: temp.path().join("current"),
        shim_dir: temp.path().join("shims"),
    };

    assert!(matches!(
        discover_native(&options),
        Err(crate::CliEditorError::UnsafeTarget(path)) if path == first.join("codex.bat")
    ));
}

#[test]
fn ignores_relative_path_entries() {
    let temp = crate::test_support::TempDir::new();
    let relative = temp.path().join("relative");
    std::fs::create_dir(&relative).expect("relative directory");
    std::fs::write(relative.join("codex.exe"), "not executable").expect("candidate");
    let options = DiscoveryOptions {
        path: std::ffi::OsString::from("relative"),
        current_dir: temp.path().to_path_buf(),
        shim_dir: temp.path().join("shim"),
    };
    assert!(matches!(
        discover_native(&options),
        Err(crate::CliEditorError::TargetNotFound(CliKind::Codex))
    ));
}

#[test]
fn excludes_current_and_shim_directories() {
    let temp = crate::test_support::TempDir::new();
    let current = temp.path().join("current");
    let shim = temp.path().join("shim");
    std::fs::create_dir(&current).expect("current directory");
    std::fs::create_dir(&shim).expect("shim directory");
    std::fs::write(current.join("codex.exe"), "not executable").expect("current candidate");
    std::fs::write(shim.join("codex.exe"), "not executable").expect("shim candidate");
    let path = std::env::join_paths([&current, &shim]).expect("PATH");
    let options = DiscoveryOptions {
        path,
        current_dir: current,
        shim_dir: shim,
    };
    assert!(matches!(
        discover_native(&options),
        Err(crate::CliEditorError::TargetNotFound(CliKind::Codex))
    ));
}
#[test]
fn only_expected_package_identity_families_can_self_adopt() {
    assert!(same_package_identity(
        "codex:@openai/codex@0.148.0",
        "codex:@openai/codex@0.149.0"
    ));
    assert!(!same_package_identity(
        "codex:@openai/codex@0.148.0",
        "codex:untrusted@0.149.0"
    ));
    assert!(!same_package_identity(
        "other:native-executable",
        "codex:native-executable"
    ));
}
#[test]
fn probe_output_is_bounded_and_removed() {
    let capture_path;
    {
        let mut capture = ProbeOutput::create().expect("probe output");
        capture_path = capture.path.clone();
        capture
            .file
            .as_mut()
            .expect("probe file")
            .write_all(&vec![b'x'; MAX_VERSION_OUTPUT_BYTES as usize + 1])
            .expect("write oversized output");
        assert!(matches!(
            capture.read(Path::new("oversized.exe")),
            Err(crate::CliEditorError::VersionProbeFailed(path))
                if path == Path::new("oversized.exe")
        ));
    }
    assert!(!capture_path.exists());
}
