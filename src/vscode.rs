use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

struct CodeCli {
    executable: PathBuf,
    script: PathBuf,
}

use crate::error::CliEditorError;
use crate::error::Result;

pub(crate) const EXTENSION_ID: &str = "asadsaleemq.cli-editor-vscode";

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
    if listed.lines().any(|line| {
        line.split('@')
            .next()
            .is_some_and(|id| id.eq_ignore_ascii_case(EXTENSION_ID))
    }) {
        return Ok(InstallOutcome::Preexisting);
    }
    let vsix = vsix
        .to_str()
        .ok_or_else(|| CliEditorError::VscodeBridge("VSIX path is not valid Unicode".into()))?;
    run_code(&code, &["--install-extension", vsix, "--force"])?;
    Ok(InstallOutcome::Added)
}

pub(crate) fn update_if_owned(vsix: &Path, owned: bool) -> Result<()> {
    if !owned {
        return Ok(());
    }
    let code = discover_code_executable().ok_or_else(|| {
        CliEditorError::VscodeBridge(
            "VS Code was removed or moved; its owned terminal bridge could not be updated".into(),
        )
    })?;
    let vsix = vsix
        .to_str()
        .ok_or_else(|| CliEditorError::VscodeBridge("VSIX path is not valid Unicode".into()))?;
    run_code(&code, &["--install-extension", vsix, "--force"])?;
    Ok(())
}

pub(crate) fn uninstall_if_owned(owned: bool) -> Result<()> {
    if !owned {
        return Ok(());
    }
    let code = discover_code_executable().ok_or_else(|| {
        CliEditorError::VscodeBridge(
            "VS Code was removed or moved; uninstall its CLI Editor Terminal Bridge extension manually"
                .into(),
        )
    })?;
    run_code(&code, &["--uninstall-extension", EXTENSION_ID])?;
    Ok(())
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
    for directory in std::env::split_paths(&path) {
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
                    script: script.canonicalize().ok()?,
                });
            }
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::EXTENSION_ID;

    #[test]
    fn extension_identity_is_stable_and_lowercase() {
        assert_eq!(EXTENSION_ID, EXTENSION_ID.to_ascii_lowercase());
        assert_eq!(EXTENSION_ID.split('.').count(), 2);
    }
}
