//! Read-only project status aggregation.
//!
//! This module reports what the project files actually say. It never runs `make`, Docker
//! or a network call, so a missing suite result is `not_run`, not a pass.

use std::path::Path;

use serde_json::{json, Value};

use crate::design;
use crate::formal;

fn read_json(path: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn count_suites(document: &Value) -> Value {
    if let Some(results) = document.get("results").and_then(Value::as_array) {
        let mut pass = 0;
        let mut fail = 0;
        let mut not_run = 0;
        let mut suites = Vec::new();
        for result in results {
            let status = result
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("not_run");
            match status {
                "pass" => pass += 1,
                "fail" => fail += 1,
                _ => not_run += 1,
            }
            suites.push(json!({
                "name": result.get("suite").and_then(Value::as_str).unwrap_or("?"),
                "status": status,
                "detail": result.get("detail").and_then(Value::as_str).unwrap_or(""),
            }));
        }
        return json!({
            "state": if fail > 0 { "fail" } else if not_run > 0 { "not_run" } else if pass > 0 { "pass" } else { "not_run" },
            "counts": {"pass": pass, "fail": fail, "notRun": not_run},
            "suites": suites,
        });
    }
    let status = document
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("not_run");
    json!({
        "state": status,
        "counts": {
            "pass": if status == "pass" { 1 } else { 0 },
            "fail": if status == "fail" { 1 } else { 0 },
            "notRun": if status == "not_run" { 1 } else { 0 },
        },
        "suites": [{
            "name": document.get("suite").and_then(Value::as_str).unwrap_or("suite"),
            "status": status,
            "detail": document.get("checks").and_then(Value::as_str).unwrap_or(""),
        }],
    })
}

fn deployment_records(project: &Path) -> Vec<Value> {
    let directory = project.join("deployments");
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut records = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json")
            || path.file_name().and_then(|name| name.to_str()) == Some("suite-results.json")
        {
            continue;
        }
        let Some(document) = read_json(&path) else {
            continue;
        };
        let report = document.get("report").unwrap_or(&document);
        records.push(json!({
            "file": path.file_name().and_then(|name| name.to_str()).unwrap_or("?"),
            "network": report.get("network").and_then(Value::as_str).unwrap_or("?"),
            "root": report.get("rootUri").and_then(Value::as_str).unwrap_or(""),
            "status": report.get("status").and_then(Value::as_str).unwrap_or("?"),
            "verified": report.get("verified").and_then(Value::as_bool).unwrap_or(false),
            "steps": report.get("steps").and_then(Value::as_array).map_or(0, Vec::len),
        }));
    }
    records.sort_by(|left, right| {
        left.get("file")
            .and_then(Value::as_str)
            .cmp(&right.get("file").and_then(Value::as_str))
    });
    records
}

/// Produce a `rhobuilder.project-status/v1` report. Missing inputs are reported as
/// `not_run`; this function does not mutate the project.
pub fn status(project: &Path) -> Value {
    let mut not_run = Vec::new();
    let design_path = project.join("design.json");
    let design = if design_path.is_file() {
        match std::fs::read_to_string(&design_path)
            .map_err(|error| error.to_string())
            .and_then(|text| design::parse(&text).map_err(|error| error.to_string()))
        {
            Ok(design) => {
                let validation = design.validate();
                json!({
                    "present": true,
                    "ok": validation.ok,
                    "slug": design.app.slug,
                    "name": design.app.name,
                    "validation": validation,
                })
            }
            Err(error) => {
                not_run.push("design.json could not be parsed");
                json!({"present": true, "ok": false, "error": error})
            }
        }
    } else {
        not_run.push("design.json is missing");
        json!({"present": false, "ok": false})
    };

    let meta = read_json(&project.join(".rhobuilder.json"));
    let manifest = read_json(&project.join(".rhobuilder-contracts.json"));
    let scaffold = json!({
        "present": meta.is_some() || manifest.is_some(),
        "template": meta.as_ref().and_then(|value| value.get("template")).and_then(Value::as_str),
        "steps": manifest.as_ref().and_then(|value| value.get("steps")).and_then(Value::as_array).map_or(0, Vec::len),
        "requiredKeys": manifest.as_ref().and_then(|value| value.get("requiredKeys")).and_then(Value::as_array).map_or(0, Vec::len),
        "note": manifest.as_ref().and_then(|value| value.get("note")).and_then(Value::as_str),
    });

    let vendor = match formal::check(project) {
        Ok(value) => value,
        Err(error) => {
            not_run.push("formal vendor check did not run");
            json!({"ok": false, "state": "not_run", "error": error.to_string()})
        }
    };
    let obligations = match formal::obligations(project, false) {
        Ok(value) => value,
        Err(error) => {
            not_run.push("formal obligations did not run");
            json!({"ok": false, "state": "not_run", "error": error.to_string()})
        }
    };
    let owed: Vec<String> = obligations
        .get("obligations")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    matches!(
                        item.get("lean_status").and_then(Value::as_str),
                        Some("owed") | Some("missing")
                    )
                })
                .filter_map(|item| item.get("component").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let named = obligations
        .get("obligations")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    !matches!(
                        item.get("lean_status").and_then(Value::as_str),
                        Some("owed") | Some("missing")
                    )
                })
                .count()
        })
        .unwrap_or(0);
    let formal_state = if obligations.get("ok").and_then(Value::as_bool) != Some(true) {
        "not_run"
    } else if !owed.is_empty() {
        "owed"
    } else if named > 0 {
        "proved-model"
    } else {
        "not_run"
    };

    let suite_path = project.join("deployments").join("suite-results.json");
    let tested = if suite_path.is_file() {
        match read_json(&suite_path) {
            Some(value) => {
                let mut counts = count_suites(&value);
                counts["source"] = Value::String("deployments/suite-results.json".to_string());
                counts
            }
            None => {
                not_run.push("suite-results.json could not be parsed");
                json!({"state": "not_run", "counts": {"pass": 0, "fail": 0, "notRun": 0}, "suites": []})
            }
        }
    } else {
        not_run.push("no suite results recorded");
        json!({"state": "not_run", "counts": {"pass": 0, "fail": 0, "notRun": 0}, "suites": []})
    };
    let deployments = deployment_records(project);
    let verified_deployment = deployments
        .iter()
        .any(|record| record.get("verified").and_then(Value::as_bool) == Some(true));
    let suite_pass = tested
        .get("counts")
        .and_then(|counts| counts.get("pass"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let tested_state = tested
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("not_run");

    let badges = vec![
        json!({
            "label": "Scaffold",
            "state": if scaffold["present"].as_bool() == Some(true) { "ok" } else { "neutral" },
            "detail": if scaffold["present"].as_bool() == Some(true) { "generated project files are present" } else { "no generated project marker" },
        }),
        json!({
            "label": "Implemented",
            "state": if verified_deployment || suite_pass > 0 { "ok" } else { "neutral" },
            "detail": if verified_deployment { "a deployment record is verified" } else if suite_pass > 0 { "a recorded suite passed" } else { "no verified deployment or passing suite" },
        }),
        json!({
            "label": "Tested",
            "state": if tested_state == "pass" { "pass" } else if tested_state == "fail" { "fail" } else { "not_run" },
            "detail": format!("{} pass, {} fail, {} not run", tested["counts"]["pass"], tested["counts"]["fail"], tested["counts"]["notRun"]),
        }),
        json!({
            "label": "Proved model",
            "state": formal_state,
            "detail": if formal_state == "owed" { format!("{} owed component(s); this is a shallow model, not a Rholang refinement", owed.len()) } else { "bounded Lean model obligation status".to_string() },
        }),
        json!({
            "label": "Owed",
            "state": if owed.is_empty() { "neutral" } else { "warn" },
            "detail": if owed.is_empty() { "none recorded".to_string() } else { owed.join(", ") },
        }),
        json!({
            "label": "Not run",
            "state": if not_run.is_empty() { "neutral" } else { "warn" },
            "detail": if not_run.is_empty() { "no skipped checks".to_string() } else { not_run.join("; ") },
        }),
    ];

    json!({
        "schema": crate::schema("project-status"),
        "ok": true,
        "project": {
            "path": project.display().to_string(),
            "slug": design.get("slug").and_then(Value::as_str).unwrap_or(""),
            "template": scaffold.get("template").and_then(Value::as_str).unwrap_or(""),
        },
        "design": design,
        "scaffold": scaffold,
        "tested": tested,
        "provedModel": {
            "state": formal_state,
            "vendor": vendor,
            "obligations": obligations,
        },
        "owed": owed,
        "notRun": not_run,
        "deployments": deployments,
        "badges": badges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scaffold(directory: &Path, owed: bool) -> Value {
        std::fs::write(
            directory.join("design.json"),
            serde_json::json!({
                "schema": "rhobuilder.design/v1",
                "app": {"name": "Demo", "slug": "demo", "summary": "demo", "networks": ["devnet"]},
                "uniqueValue": "unique",
                "components": [{"id": "panel", "tag": "new", "kind": "custom",
                    "whyUnique": "unique", "noRegisteredAlternative": ["rcash.account-factory"],
                    "arms": ["act"], "leanStatus": "owed"}],
                "acceptance": [{"id": "a1", "given": "g", "when": "w", "then": "t"}]
            })
            .to_string(),
        )
        .expect("design");
        std::fs::write(
            directory.join(".rhobuilder.json"),
            serde_json::json!({"template": "headless"}).to_string(),
        )
        .expect("meta");
        let flows = directory.join("spec").join("flows");
        std::fs::create_dir_all(&flows).expect("flows dir");
        std::fs::write(
            flows.join("core.yaml"),
            serde_json::json!({"components": [{
                "id": "panel",
                "arms": if owed { vec!["act"] } else { Vec::<&str>::new() },
                "lean_status": if owed { "owed" } else { "proved-model" },
                "lean": if owed { "" } else { "actOnce" }
            }]})
            .to_string(),
        )
        .expect("flows");
        status(directory)
    }

    #[test]
    fn a_scaffold_reports_owed_and_not_run() {
        let directory = tempfile::tempdir().expect("tempdir");
        let value = scaffold(directory.path(), true);
        assert_eq!(value["schema"], "rhobuilder.project-status/v1");
        assert_eq!(value["design"]["ok"], true);
        assert_eq!(value["scaffold"]["present"], true);
        assert_eq!(value["owed"][0], "panel");
        let badges = value["badges"].as_array().expect("badges");
        assert!(badges.iter().any(|badge| badge["label"] == "Owed"));
        assert!(badges
            .iter()
            .any(|badge| badge["label"] == "Tested" && badge["state"] == "not_run"));
    }

    #[test]
    fn recorded_suites_change_the_tested_badge() {
        let directory = tempfile::tempdir().expect("tempdir");
        scaffold(directory.path(), true);
        std::fs::create_dir_all(directory.path().join("deployments")).expect("deployments");
        std::fs::write(
            directory
                .path()
                .join("deployments")
                .join("suite-results.json"),
            serde_json::json!({"schema": "rhobuilder.suite-results/v1", "results": [
                {"suite": "root", "status": "pass"},
                {"suite": "panel", "status": "pass"}
            ]})
            .to_string(),
        )
        .expect("suite results");
        let value = status(directory.path());
        assert_eq!(value["tested"]["state"], "pass");
        assert_eq!(value["tested"]["counts"]["pass"], 2);
        let badges = value["badges"].as_array().expect("badges");
        assert!(badges
            .iter()
            .any(|badge| badge["label"] == "Implemented" && badge["state"] == "ok"));
    }

    #[test]
    fn missing_files_are_not_run_not_an_error() {
        let directory = tempfile::tempdir().expect("tempdir");
        let value = status(directory.path());
        assert_eq!(value["ok"], true);
        assert_eq!(value["design"]["present"], false);
        let not_run = value["notRun"].as_array().expect("not run");
        assert!(!not_run.is_empty());
    }
}
