use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

struct CodeCli {
    executable: PathBuf,
    script: PathBuf,
}

use crate::error::CliEditorError;
use crate::error::Result;

pub(crate) const EXTENSION_ID: &str = "asadsaleemq.cli-editor";
const LEGACY_EXTENSION_ID: &str = "asadsaleemq.cli-editor-vscode";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstallOutcome {
    Added,
    Preexisting,
    Unavailable,
}

pub(crate) fn install(vsix: &Path) -> Result<InstallOutcome> {
    let Some(code) = discover_code_executable() else {
        return Ok(InstallOutcome::Unavailable);
    };
    let listed = run_code(&code, &["--list-extensions", "--show-versions"])?;
    let current_is_installed = contains_extension(&listed, EXTENSION_ID);
    let legacy_is_installed = contains_extension(&listed, LEGACY_EXTENSION_ID);
    if current_is_installed {
        if legacy_is_installed {
            run_code(&code, &["--uninstall-extension", LEGACY_EXTENSION_ID])?;
        }
        return Ok(InstallOutcome::Preexisting);
    }
    let vsix = vsix
        .to_str()
        .ok_or_else(|| CliEditorError::VscodeBridge("VSIX path is not valid Unicode".into()))?;
    run_code(&code, &["--install-extension", vsix, "--force"])?;
    if legacy_is_installed {
        run_code(&code, &["--uninstall-extension", LEGACY_EXTENSION_ID])?;
    }
    Ok(InstallOutcome::Added)
}

pub(crate) fn update_if_owned(vsix: &Path, owned: bool) -> Result<()> {
    if !owned {
        return Ok(());
    }
    let code = discover_code_executable().ok_or_else(|| {
        CliEditorError::VscodeBridge(
            "VS Code was removed or moved; its owned CLI Editor extension could not be updated"
                .into(),
        )
    })?;
    let vsix = vsix
        .to_str()
        .ok_or_else(|| CliEditorError::VscodeBridge("VSIX path is not valid Unicode".into()))?;
    run_code(&code, &["--install-extension", vsix, "--force"])?;
    let listed = run_code(&code, &["--list-extensions", "--show-versions"])?;
    if contains_extension(&listed, LEGACY_EXTENSION_ID) {
        run_code(&code, &["--uninstall-extension", LEGACY_EXTENSION_ID])?;
    }
    Ok(())
}

pub(crate) fn uninstall_if_owned(owned: bool) -> Result<()> {
    if !owned {
        return Ok(());
    }
    let code = discover_code_executable().ok_or_else(|| {
        CliEditorError::VscodeBridge(
            "VS Code was removed or moved; uninstall its CLI Editor extension manually".into(),
        )
    })?;
    let listed = run_code(&code, &["--list-extensions", "--show-versions"])?;
    for id in [EXTENSION_ID, LEGACY_EXTENSION_ID] {
        if contains_extension(&listed, id) {
            run_code(&code, &["--uninstall-extension", id])?;
        }
    }
    Ok(())
}

fn contains_extension(listed: &str, extension_id: &str) -> bool {
    listed.lines().any(|line| {
        line.split('@')
            .next()
            .is_some_and(|id| id.eq_ignore_ascii_case(extension_id))
    })
}

fn run_code(code: &CodeCli, args: &[&str]) -> Result<String> {
    let output = Command::new(&code.executable)
        .arg(&code.script)
        .args(args)
        .env("ELECTRON_RUN_AS_NODE", "1")
        .env("VSCODE_DEV", "")
        .output()
        .map_err(|source| CliEditorError::io(&code.executable, source))?;
    if !output.status.success() {
        return Err(CliEditorError::VscodeBridge(format!(
            "VS Code command failed (exit {:?}): {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn discover_code_executable() -> Option<CodeCli> {
    let path = std::env::var_os("PATH")?;
    discover_code_executable_in(&path)
}

fn discover_code_executable_in(path: &OsStr) -> Option<CodeCli> {
    for directory in std::env::split_paths(path) {
        for launcher in ["code.cmd", "code"] {
            let candidate = directory.join(launcher);
            if !candidate.is_file() {
                continue;
            }
            let Some(bin) = candidate.parent() else {
                continue;
            };
            let Some(root) = bin.parent() else {
                continue;
            };
            let executable = root.join("Code.exe");
            if !executable.is_file() {
                continue;
            }
            let mut scripts = std::fs::read_dir(root)
                .ok()?
                .filter_map(std::result::Result::ok)
                .map(|entry| {
                    entry
                        .path()
                        .join("resources")
                        .join("app")
                        .join("out")
                        .join("cli.js")
                })
                .filter(|path| path.is_file())
                .collect::<Vec<_>>();
            scripts.sort();
            if let Some(script) = scripts.pop() {
                return Some(CodeCli {
                    executable: executable.canonicalize().ok()?,
                    script,
                });
            }
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::EXTENSION_ID;
    use super::contains_extension;
    use super::discover_code_executable_in;

    #[test]
    fn extension_identity_is_stable_and_lowercase() {
        assert_eq!(EXTENSION_ID, EXTENSION_ID.to_ascii_lowercase());
        assert_eq!(EXTENSION_ID.split('.').count(), 2);
    }

    #[test]
    fn extension_listing_matches_identity_without_confusing_versions() {
        let listed = "publisher.other@1.0.0\nasadsaleemq.cli-editor@0.3.0\n";
        assert!(contains_extension(listed, EXTENSION_ID));
        assert!(!contains_extension(listed, "asadsaleemq.cli-editor-vscode"));
    }

    #[cfg(windows)]
    #[test]
    fn code_cli_script_path_avoids_windows_verbatim_prefix() {
        let root =
            std::env::temp_dir().join(format!("cli-editor-discovery-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let bin = root.join("bin");
        let script = root
            .join("1.134.0")
            .join("resources")
            .join("app")
            .join("out")
            .join("cli.js");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::create_dir_all(script.parent().unwrap()).unwrap();
        std::fs::write(root.join("Code.exe"), []).unwrap();
        std::fs::write(bin.join("code.cmd"), []).unwrap();
        std::fs::write(&script, []).unwrap();

        let path = std::env::join_paths([&bin]).unwrap();
        let discovered = discover_code_executable_in(&path).unwrap();
        let discovered_script = discovered.script;
        std::fs::remove_dir_all(&root).unwrap();

        assert_eq!(discovered_script, script);
        assert!(!discovered_script.to_string_lossy().starts_with(r"\\?\"));
    }
}
