//! Gate orchestration over a generated project's Makefile.
//!
//! There is one gate implementation: the generated project's `make` targets. This crate
//! runs them and normalises `pass`/`fail`/`not_run`; it does not reimplement a checker.

use std::path::Path;
use std::process::Command;

use serde::Serialize;
use thiserror::Error;

pub const STAGE: &str = "P2";

#[derive(Debug, Error)]
pub enum GateError {
    #[error("profile must be static, formal or full")]
    UnknownProfile,
    #[error("cannot run make: {0}")]
    Spawn(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateResult {
    pub name: String,
    pub status: String,
    pub exit_code: i32,
    pub output: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    pub schema: String,
    pub ok: bool,
    pub profile: String,
    pub gates: Vec<GateResult>,
}

pub fn implemented() -> bool {
    true
}

/// Run the project's Makefile targets for `profile`.
pub fn check(project: &Path, profile: &str) -> Result<CheckReport, GateError> {
    let targets: &[&str] = match profile {
        "static" => &["check"],
        "formal" => &["formal-check"],
        "full" => &["check", "formal-check", "unit-all"],
        _ => return Err(GateError::UnknownProfile),
    };
    let mut gates = Vec::new();
    let mut failed = false;
    let mut not_run = false;
    for target in targets {
        let result = run_target(project, target);
        match result {
            Ok((status, code, output)) => {
                if status == "fail" {
                    failed = true;
                }
                if status == "not_run" {
                    not_run = true;
                }
                gates.push(GateResult {
                    name: (*target).to_string(),
                    status: status.to_string(),
                    exit_code: code,
                    output,
                });
            }
            Err(error) => {
                failed = true;
                gates.push(GateResult {
                    name: (*target).to_string(),
                    status: "fail".to_string(),
                    exit_code: 1,
                    output: error,
                });
            }
        }
    }
    let ok = !failed && !not_run;
    Ok(CheckReport {
        schema: "rhobuilder.check/v1".to_string(),
        ok,
        profile: profile.to_string(),
        gates,
    })
}

fn run_target(project: &Path, target: &str) -> Result<(&'static str, i32, String), String> {
    let output = Command::new("make")
        .arg("-C")
        .arg(project)
        .arg(target)
        .output()
        .map_err(|error| format!("make -C {} {target}: {error}", project.display()))?;
    let code = output.status.code().unwrap_or(1);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let status = if output.status.success() {
        "pass"
    } else if code == 2 {
        "not_run"
    } else {
        "fail"
    };
    let tail: String = text
        .chars()
        .rev()
        .take(4000)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    Ok((status, code, tail))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_passing_target_is_reported() {
        let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
        std::fs::write(
            dir.path().join("Makefile"),
            "check:\n\t@echo static-ok\nformal-check:\n\t@echo formal-ok\n",
        )
        .unwrap_or_else(|_| panic!("write"));
        let report = check(dir.path(), "static").unwrap_or_else(|_| panic!("check"));
        assert!(report.ok);
        assert_eq!(report.gates[0].status, "pass");
    }

    #[test]
    fn exit_two_is_not_run() {
        let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
        std::fs::write(
            dir.path().join("Makefile"),
            "check:\n\t@echo unavailable; exit 2\n",
        )
        .unwrap_or_else(|_| panic!("write"));
        let report = check(dir.path(), "static").unwrap_or_else(|_| panic!("check"));
        assert!(!report.ok);
        assert_eq!(report.gates[0].status, "not_run");
    }

    #[test]
    fn an_unknown_profile_is_refused() {
        let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
        assert!(matches!(
            check(dir.path(), "nope"),
            Err(GateError::UnknownProfile)
        ));
    }
}
