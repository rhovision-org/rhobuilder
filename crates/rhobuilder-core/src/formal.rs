//! Vendoring and checking the shared Lean `RhoApp` library.
//!
//! Generated apps vendor `spec/lean/RhoApp.lean` plus `spec/lean/RhoApp/**` and record a
//! deterministic tree hash in `spec/lean/VENDORED.json`. `check` recomputes the hash; it
//! never trusts the recorded value without reading the files.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::scaffold::ScaffoldError;

pub const VENDOR_SCHEMA: &str = "rhobuilder.lean-vendor/v1";

fn read_dir_sorted(path: &Path) -> Result<Vec<std::path::PathBuf>, ScaffoldError> {
    let mut entries = std::fs::read_dir(path)
        .map_err(|source| ScaffoldError::Io {
            path: path.display().to_string(),
            source,
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort();
    Ok(entries)
}

fn collect_dir(
    base: &Path,
    prefix: &str,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), ScaffoldError> {
    walk_dir(base, base, prefix, files)
}

fn walk_dir(
    directory: &Path,
    base: &Path,
    prefix: &str,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), ScaffoldError> {
    for entry in read_dir_sorted(directory)? {
        if entry.is_dir() {
            walk_dir(&entry, base, prefix, files)?;
            continue;
        }
        let relative = entry
            .strip_prefix(base)
            .map_err(|_| ScaffoldError::MissingTemplate(entry.display().to_string()))?;
        let key = format!("{prefix}/{}", relative.to_string_lossy().replace('\\', "/"));
        let bytes = std::fs::read(&entry).map_err(|source| ScaffoldError::Io {
            path: entry.display().to_string(),
            source,
        })?;
        files.insert(key, bytes);
    }
    Ok(())
}

/// Collect the shared library in the same layout a generated app expects:
/// `spec/lean/RhoApp.lean` and `spec/lean/RhoApp/**`.
pub fn collect(source_root: &Path) -> Result<Option<BTreeMap<String, Vec<u8>>>, ScaffoldError> {
    let lean_root = source_root.join("formal").join("lean").join("RhoApp");
    if !lean_root.is_dir() {
        return Ok(None);
    }
    let mut files = BTreeMap::new();
    let root = lean_root.join("RhoApp.lean");
    if root.is_file() {
        let bytes = std::fs::read(&root).map_err(|source| ScaffoldError::Io {
            path: root.display().to_string(),
            source,
        })?;
        files.insert("spec/lean/RhoApp.lean".to_string(), bytes);
    }
    let modules = lean_root.join("RhoApp");
    if modules.is_dir() {
        collect_dir(&modules, "spec/lean/RhoApp", &mut files)?;
    }
    if files.is_empty() {
        Ok(None)
    } else {
        Ok(Some(files))
    }
}

pub fn tree_hash(files: &BTreeMap<String, Vec<u8>>) -> String {
    let mut hasher = Sha256::new();
    for (path, bytes) in files {
        hasher.update(path.as_bytes());
        hasher.update([0u8]);
        hasher.update(bytes);
        hasher.update([0u8]);
    }
    crate::hex(&hasher.finalize())
}

pub fn vendored_manifest(files: &BTreeMap<String, Vec<u8>>) -> Value {
    json!({
        "schema": VENDOR_SCHEMA,
        "version": env!("CARGO_PKG_VERSION"),
        "source": "rhobuilder",
        "sourceCommit": std::env::var("RHOBUILDER_SOURCE_COMMIT").unwrap_or_else(|_| "working-tree".to_string()),
        "treeSha256": tree_hash(files),
        "files": files.len(),
    })
}

/// Insert the vendored library and its manifest into a scaffold file map.
pub fn vendor_into(
    source_root: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<bool, ScaffoldError> {
    let Some(vendored) = collect(source_root)? else {
        return Ok(false);
    };
    let manifest = vendored_manifest(&vendored);
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| ScaffoldError::Unresolved(error.to_string()))?;
    files.extend(vendored);
    files.insert("spec/lean/VENDORED.json".to_string(), bytes);
    Ok(true)
}

/// Recompute the hash from the files in the project.
pub fn check(project: &Path) -> Result<Value, ScaffoldError> {
    let manifest_path = project.join("spec").join("lean").join("VENDORED.json");
    let text = std::fs::read_to_string(&manifest_path).map_err(|source| ScaffoldError::Io {
        path: manifest_path.display().to_string(),
        source,
    })?;
    let manifest: Value = serde_json::from_str(&text)
        .map_err(|error| ScaffoldError::Unresolved(format!("invalid VENDORED.json: {error}")))?;
    let expected = manifest
        .get("treeSha256")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let mut files = BTreeMap::new();
    let root = project.join("spec").join("lean").join("RhoApp.lean");
    if root.is_file() {
        let bytes = std::fs::read(&root).map_err(|source| ScaffoldError::Io {
            path: root.display().to_string(),
            source,
        })?;
        files.insert("spec/lean/RhoApp.lean".to_string(), bytes);
    }
    let modules = project.join("spec").join("lean").join("RhoApp");
    if modules.is_dir() {
        collect_dir(&modules, "spec/lean/RhoApp", &mut files)?;
    }
    let actual = tree_hash(&files);
    Ok(json!({
        "schema": "rhobuilder.lean-vendor-check/v1",
        "ok": !expected.is_empty() && expected == actual,
        "expected": expected,
        "actual": actual,
        "files": files.len(),
    }))
}

/// Refresh the vendored library from the rhobuilder checkout.
/// Check the per-app flow obligations and forbidden Lean escapes.
///
/// `owed` is a first-class status: the default gate accepts it and reports it; `strict`
/// fails on it. A non-`owed` component must name a theorem that exists in the app's Lean
/// tree. This is a model obligation check, never a Rholang refinement claim.
pub fn obligations(project: &Path, strict: bool) -> Result<Value, ScaffoldError> {
    let flows_path = project.join("spec").join("flows").join("core.yaml");
    let text = std::fs::read_to_string(&flows_path).map_err(|source| ScaffoldError::Io {
        path: flows_path.display().to_string(),
        source,
    })?;
    let flows: Value = serde_json::from_str(&text)
        .map_err(|error| ScaffoldError::Unresolved(format!("invalid flows JSON: {error}")))?;
    let mut obligations = Vec::new();
    let mut violations = Vec::new();
    let mut named: Vec<(String, String)> = Vec::new();
    if let Some(components) = flows.get("components").and_then(Value::as_array) {
        for component in components {
            let arms: Vec<String> = component
                .get("arms")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            if arms.is_empty() {
                continue;
            }
            let id = component
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let status = component
                .get("lean_status")
                .and_then(Value::as_str)
                .unwrap_or("missing");
            let theorem = component.get("lean").and_then(Value::as_str).unwrap_or("");
            if status == "missing" {
                violations.push(format!("{id}: mutating arms have no lean_status"));
            } else if status == "owed" {
                if strict {
                    violations.push(format!("{id}: strict mode rejects owed"));
                }
            } else if theorem.is_empty() {
                violations.push(format!("{id}: status {status} has no named theorem"));
            } else {
                named.push((id.to_string(), theorem.to_string()));
            }
            obligations.push(json!({
                "component": id,
                "arms": arms,
                "lean_status": status,
                "lean": theorem,
            }));
        }
    }

    let mut lean_files = 0;
    let mut lean_text = String::new();
    let lean_root = project.join("spec").join("lean");
    if lean_root.is_dir() {
        let mut stack = vec![lean_root];
        while let Some(directory) = stack.pop() {
            let entries = std::fs::read_dir(&directory).map_err(|source| ScaffoldError::Io {
                path: directory.display().to_string(),
                source,
            })?;
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|ext| ext.to_str()) != Some("lean") {
                    continue;
                }
                lean_files += 1;
                let text = std::fs::read_to_string(&path).map_err(|source| ScaffoldError::Io {
                    path: path.display().to_string(),
                    source,
                })?;
                for token in ["sorry", "admit", "axiom", "native_decide"] {
                    if text
                        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                        .any(|word| word == token)
                    {
                        violations.push(format!(
                            "{}: forbidden Lean token {token}",
                            path.strip_prefix(project).unwrap_or(&path).display()
                        ));
                    }
                }
                lean_text.push_str(&text);
                lean_text.push('\n');
            }
        }
    }
    for (component, theorem) in named {
        let present = lean_text.contains(&format!("theorem {theorem}"))
            || lean_text.contains(&format!("lemma {theorem}"));
        if !present {
            violations.push(format!(
                "{component}: named theorem {theorem} is not in spec/lean"
            ));
        }
    }

    Ok(json!({
        "schema": "rhobuilder.formal-obligations/v1",
        "ok": violations.is_empty(),
        "strict": strict,
        "obligations": obligations,
        "leanFiles": lean_files,
        "violations": violations,
    }))
}

pub fn sync(project: &Path, source_root: &Path) -> Result<Value, ScaffoldError> {
    let Some(vendored) = collect(source_root)? else {
        return Err(ScaffoldError::MissingTemplate(
            source_root.join("formal/lean/RhoApp").display().to_string(),
        ));
    };
    let manifest = vendored_manifest(&vendored);
    let mut written = 0;
    for (relative, bytes) in &vendored {
        let path = project.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| ScaffoldError::Io {
                path: parent.display().to_string(),
                source,
            })?;
        }
        std::fs::write(&path, bytes).map_err(|source| ScaffoldError::Io {
            path: path.display().to_string(),
            source,
        })?;
        written += 1;
    }
    let manifest_path = project.join("spec").join("lean").join("VENDORED.json");
    if let Some(parent) = manifest_path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| ScaffoldError::Io {
            path: parent.display().to_string(),
            source,
        })?;
    }
    let text = serde_json::to_string_pretty(&manifest)
        .map_err(|error| ScaffoldError::Unresolved(error.to_string()))?;
    std::fs::write(&manifest_path, text).map_err(|source| ScaffoldError::Io {
        path: manifest_path.display().to_string(),
        source,
    })?;
    Ok(json!({
        "schema": "rhobuilder.lean-vendor-sync/v1",
        "ok": true,
        "written": written,
        "treeSha256": manifest.get("treeSha256"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tree_hash_is_deterministic_and_order_independent() {
        let mut files = BTreeMap::new();
        files.insert("a".to_string(), b"one".to_vec());
        files.insert("b".to_string(), b"two".to_vec());
        let first = tree_hash(&files);
        let second = tree_hash(&files.clone());
        assert_eq!(first, second);
        files.insert("c".to_string(), b"three".to_vec());
        assert_ne!(first, tree_hash(&files));
    }

    #[test]
    fn sync_then_check_round_trips() {
        let source = tempfile::tempdir().expect("source");
        std::fs::create_dir_all(source.path().join("formal/lean/RhoApp/RhoApp")).expect("dirs");
        std::fs::write(
            source.path().join("formal/lean/RhoApp/RhoApp.lean"),
            "import RhoApp.Basic\n",
        )
        .expect("root");
        std::fs::write(
            source.path().join("formal/lean/RhoApp/RhoApp/Basic.lean"),
            "namespace RhoApp\n",
        )
        .expect("basic");
        let project = tempfile::tempdir().expect("project");
        let report = sync(project.path(), source.path()).expect("sync");
        assert_eq!(report["ok"], true);
        let first = check(project.path()).expect("check");
        assert_eq!(first["ok"], true, "{first:?}");
        std::fs::write(
            project.path().join("spec/lean/RhoApp/Basic.lean"),
            "tampered",
        )
        .expect("tamper");
        let second = check(project.path()).expect("check");
        assert_eq!(second["ok"], false, "{second:?}");
    }
}
