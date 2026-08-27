use serde::Serialize;

use crate::discovery::sha256_file;
use crate::dispatcher::validate_native_metadata;
use crate::error::Result;
use crate::registry::{prepend_shim, read_user_path};
use crate::{AdoptionRecord, CliKind, StateStore};

#[derive(Serialize)]
struct DoctorReport {
    installed: bool,
    healthy: bool,
    checks: Vec<Check>,
    adoption_history: Vec<AdoptionRecord>,
}

#[derive(Serialize)]
struct Check {
    name: String,
    ok: bool,
    detail: String,
}

pub(crate) fn status() -> Result<()> {
    let store = StateStore::for_current_user()?;
    let Some(state) = store.load()? else {
        println!("CLI Editor is not installed.");
        return Ok(());
    };
    println!("CLI Editor {}", state.installed_version);
    println!(
        "  Codex default: {}",
        if state.defaults.codex_enhanced {
            "enhanced"
        } else {
            "native"
        }
    );
    println!(
        "  Claude default: {}{}",
        if state.defaults.claude_managed {
            "managed native"
        } else {
            "native"
        },
        if state.defaults.claude_strict {
            " (strict)"
        } else {
            ""
        }
    );
    for kind in [CliKind::Codex, CliKind::Claude] {
        if let Some(target) = state.native_targets.get(&kind) {
            println!(
                "  {} native: {} [{}]",
                kind.as_str(),
                target.path.display(),
                target.version
            );
        }
    }
    if let Some(release) = &state.active_release {
        println!(
            "  enhanced Codex: {} [Codex {}]",
            release.directory.display(),
            release.codex_version
        );
    }
    println!(
        "  native adoptions recorded: {}",
        state.adoption_history.len()
    );
    for kind in [CliKind::Codex, CliKind::Claude] {
        if let Some(record) = state
            .adoption_history
            .iter()
            .rev()
            .find(|record| record.cli == kind)
        {
            println!(
                "  latest {} adoption: {} -> {} at {} ms [{}]",
                kind.as_str(),
                record.old_version,
                record.new_version,
                record.timestamp_unix_ms,
                record.package_root.display()
            );
        }
    }
    Ok(())
}

pub(crate) fn doctor(json: bool) -> Result<i32> {
    let store = StateStore::for_current_user()?;
    let Some(state) = store.load()? else {
        let report = DoctorReport {
            installed: false,
            healthy: false,
            checks: vec![Check {
                name: "installation".into(),
                ok: false,
                detail: "CLI Editor is not installed".into(),
            }],
            adoption_history: Vec::new(),
        };
        print_report(&report, json)?;
        return Ok(1);
    };

    let mut checks = Vec::new();
    for kind in [CliKind::Codex, CliKind::Claude] {
        let (ok, detail) = match state.native_targets.get(&kind) {
            Some(target) => match validate_native_metadata(kind, target) {
                Ok(()) => (
                    true,
                    format!("{} [{}]", target.path.display(), target.version),
                ),
                Err(error) => (false, error.to_string()),
            },
            None => (true, "not installed at CLI Editor setup time".into()),
        };
        checks.push(Check {
            name: format!("{} native target", kind.as_str()),
            ok,
            detail,
        });
    }
    if let Some(release) = &state.active_release {
        let path = release.directory.join("codex.exe");
        let metadata_unchanged = path.metadata().ok().is_some_and(|metadata| {
            let modified = metadata
                .modified()
                .ok()
                .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |value| value.as_millis());
            release.file_size != 0
                && metadata.len() == release.file_size
                && modified == release.modified_unix_ms
        });
        let actual = if metadata_unchanged {
            Ok(release.sha256.clone())
        } else {
            sha256_file(&path)
        };
        checks.push(Check {
            name: "enhanced Codex artifact".into(),
            ok: actual.as_ref().is_ok_and(|hash| hash == &release.sha256),
            detail: actual.map_or_else(
                |error| error.to_string(),
                |hash| format!("{} sha256={hash}", path.display()),
            ),
        });
    }
    if let Some(shim) = &state.shim_directory {
        let mut expected_shims = vec![("cli-editor", "cli-editor.exe")];
        if state.native_targets.contains_key(&CliKind::Codex) {
            expected_shims.push(("codex", "codex.exe"));
        }
        if state.native_targets.contains_key(&CliKind::Claude) {
            expected_shims.push(("claude", "claude.exe"));
        }
        for (command, name) in expected_shims {
            let path = shim.join(name);
            checks.push(Check {
                name: format!("{name} shim"),
                ok: path.is_file(),
                detail: path.display().to_string(),
            });
            if command != "cli-editor" {
                let expected = path.canonicalize().ok();
                let resolved = first_command_path(command);
                checks.push(Check {
                    name: format!("{command} command precedence"),
                    ok: expected.is_some() && resolved == expected,
                    detail: resolved.map_or_else(
                        || "not found on current PATH".into(),
                        |path| path.display().to_string(),
                    ),
                });
            }
        }
        let current = read_user_path()?;
        let expected = prepend_shim(&current, shim)?;
        checks.push(Check {
            name: "user PATH entry present".into(),
            ok: expected == current.data,
            detail: shim.display().to_string(),
        });
    }
    let healthy = checks.iter().all(|check| check.ok);
    let report = DoctorReport {
        installed: true,
        healthy,
        checks,
        adoption_history: state.adoption_history.clone(),
    };
    print_report(&report, json)?;
    Ok(if healthy { 0 } else { 1 })
}

fn first_command_path(command: &str) -> Option<std::path::PathBuf> {
    let extensions = std::env::var_os("PATHEXT")
        .map(|value| {
            value
                .to_string_lossy()
                .split(';')
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| vec![".COM".into(), ".EXE".into(), ".BAT".into(), ".CMD".into()]);
    let path = std::env::var_os("PATH").unwrap_or_default();
    let directories = std::env::split_paths(&path);
    for directory in directories {
        for extension in &extensions {
            let candidate = directory.join(format!("{command}{extension}"));
            if candidate.is_file() {
                return candidate.canonicalize().ok();
            }
        }
    }
    None
}

fn print_report(report: &DoctorReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!(
            "CLI Editor doctor: {}",
            if report.healthy {
                "healthy"
            } else {
                "attention required"
            }
        );
        for check in &report.checks {
            println!(
                "  [{}] {}: {}",
                if check.ok { "ok" } else { "FAIL" },
                check.name,
                check.detail
            );
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::{Check, DoctorReport};
    use crate::{AdoptionRecord, CliKind};

    #[test]
    fn json_report_exposes_durable_adoption_history() {
        let report = DoctorReport {
            installed: true,
            healthy: true,
            checks: vec![Check {
                name: "state".into(),
                ok: true,
                detail: "valid".into(),
            }],
            adoption_history: vec![AdoptionRecord {
                timestamp_unix_ms: 123,
                cli: CliKind::Claude,
                package_root: r"C:\native".into(),
                old_version: "1.0.0".into(),
                new_version: "1.0.1".into(),
                old_sha256: "aa".repeat(32),
                new_sha256: "bb".repeat(32),
            }],
        };

        let json = serde_json::to_value(report).expect("doctor report");
        let adoption = &json["adoption_history"][0];
        assert_eq!(adoption["cli"], "claude");
        assert_eq!(adoption["old_version"], "1.0.0");
        assert_eq!(adoption["new_version"], "1.0.1");
        assert_eq!(adoption["timestamp_unix_ms"], 123);
    }
}
