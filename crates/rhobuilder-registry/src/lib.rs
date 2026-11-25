//! The on-chain primitive registry: catalog, graph, pins and pinned-commit drift.
//!
//! The registry is curated. Drift reads the sibling repository at the pinned commit with
//! `git show`/`git cat-file`, never a dirty worktree. A pin that is not available locally
//! is reported as `pin-unavailable`, not as drift and not as success.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub mod reuse;

/// A committed primitive (`registry/primitives.json`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Primitive {
    pub id: String,
    pub owner: String,
    pub kind: String,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub arms: Vec<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub pinned_commit: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub policy: Option<Policy>,
    #[serde(default)]
    pub networks: BTreeMap<String, NetworkEntry>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Policy {
    #[serde(default)]
    pub reuse: Option<String>,
    #[serde(default)]
    pub extension: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkEntry {
    pub verified: String,
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub uri: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Vertex {
    pub id: String,
    pub owner: String,
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Edge {
    pub id: String,
    pub verb: String,
    pub from: String,
    pub to: String,
    pub kind: String,
    #[serde(default)]
    pub arms: Vec<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Registry {
    pub schema: String,
    #[serde(default)]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub pins: Value,
    pub primitives: Vec<Primitive>,
    #[serde(default)]
    pub vertices: Vec<Vertex>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub invariants: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pins {
    pub schema: String,
    #[serde(default)]
    pub rust: Value,
    #[serde(default)]
    pub node: Value,
    #[serde(default)]
    pub crypto: Value,
    #[serde(default)]
    pub siblings: BTreeMap<String, SiblingPin>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiblingPin {
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub rev: Option<String>,
    #[serde(default)]
    pub consumer_pins: BTreeMap<String, String>,
    #[serde(default)]
    pub local_head: Option<String>,
    #[serde(default)]
    pub authoritative: Option<bool>,
    #[serde(default)]
    pub consumer: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphView {
    pub root: String,
    pub depth: usize,
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilteredPrimitive {
    #[serde(flatten)]
    pub primitive: Primitive,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkEntry>,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("invalid JSON in {path}: {source}")]
    Json {
        path: String,
        source: serde_json::Error,
    },
    #[error("unknown primitive: {0}")]
    UnknownPrimitive(String),
    #[error("unknown graph root: {0}")]
    UnknownVertex(String),
    #[error("git failed in {dir}: {detail}")]
    Git { dir: String, detail: String },
    #[error("pins are not a rhobuilder.pins/v1 document")]
    BadPins,
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, RegistryError> {
    let text = std::fs::read_to_string(path).map_err(|source| RegistryError::Io {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| RegistryError::Json {
        path: path.display().to_string(),
        source,
    })
}

impl Registry {
    pub fn load(root: &Path) -> Result<Self, RegistryError> {
        read_json(&root.join("primitives.json"))
    }

    pub fn get(&self, id: &str) -> Result<&Primitive, RegistryError> {
        self.primitives
            .iter()
            .find(|primitive| primitive.id == id)
            .ok_or_else(|| RegistryError::UnknownPrimitive(id.to_string()))
    }

    /// Filter the catalog. A network filter selects primitives that have an entry for it.
    pub fn list(
        &self,
        network: Option<&str>,
        owner: Option<&str>,
        kind: Option<&str>,
    ) -> Vec<FilteredPrimitive> {
        self.primitives
            .iter()
            .filter(|primitive| owner.is_none_or(|value| primitive.owner == value))
            .filter(|primitive| kind.is_none_or(|value| primitive.kind == value))
            .filter(|primitive| network.is_none_or(|value| primitive.networks.contains_key(value)))
            .map(|primitive| FilteredPrimitive {
                primitive: primitive.clone(),
                network: network.and_then(|value| primitive.networks.get(value).cloned()),
            })
            .collect()
    }

    /// Breadth-first view over the named graph, root included.
    pub fn graph(&self, root: &str, depth: usize) -> Result<GraphView, RegistryError> {
        if !self.vertices.iter().any(|vertex| vertex.id == root) {
            return Err(RegistryError::UnknownVertex(root.to_string()));
        }
        let mut queue: VecDeque<(String, usize)> = VecDeque::from([(root.to_string(), 0)]);
        let mut seen_vertices: BTreeSet<String> = BTreeSet::new();
        let mut seen_edges: BTreeSet<String> = BTreeSet::new();
        let mut edges: Vec<Edge> = Vec::new();
        while let Some((vertex, distance)) = queue.pop_front() {
            if distance > depth || !seen_vertices.insert(vertex.clone()) {
                continue;
            }
            if distance >= depth {
                continue;
            }
            for edge in self
                .edges
                .iter()
                .filter(|edge| edge.from == vertex || edge.to == vertex)
            {
                if seen_edges.insert(edge.id.clone()) {
                    edges.push(edge.clone());
                    for next in [&edge.from, &edge.to] {
                        if next != &vertex {
                            queue.push_back((next.clone(), distance + 1));
                        }
                    }
                }
            }
        }
        Ok(GraphView {
            root: root.to_string(),
            depth,
            vertices: self
                .vertices
                .iter()
                .filter(|vertex| seen_vertices.contains(&vertex.id))
                .cloned()
                .collect(),
            edges,
        })
    }
}

pub fn load_pins(root: &Path) -> Result<Pins, RegistryError> {
    let pins: Pins = read_json(&root.join("pins.json"))?;
    if pins.schema != "rhobuilder.pins/v1" {
        return Err(RegistryError::BadPins);
    }
    Ok(pins)
}

fn is_hex40(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// The local checkout for a sibling repository. Override with
/// `RHOBUILDER_DIR_<NAME>` where `<NAME>` is upper-case with `-` replaced by `_`.
pub fn checkout(source_root: &Path, name: &str) -> PathBuf {
    let env_name = format!("RHOBUILDER_DIR_{}", name.replace('-', "_").to_uppercase());
    if let Ok(value) = std::env::var(&env_name) {
        if !value.is_empty() {
            return PathBuf::from(value);
        }
    }
    let directory = match name {
        "r-cash" => "Rholphin",
        "r-gen" => "RGen",
        "r-sure" => "RSure",
        "r-share" => "RShare",
        "r-print" => "RPrint",
        "r-bridge" => "RBridge",
        "rholo" => "rholo-fixed",
        other => other,
    };
    source_root.join(directory)
}

fn git(dir: &Path, args: &[&str]) -> Result<std::process::Output, RegistryError> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|error| RegistryError::Git {
            dir: dir.display().to_string(),
            detail: error.to_string(),
        })
}

fn git_ok(dir: &Path, args: &[&str]) -> bool {
    git(dir, args)
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn git_text(dir: &Path, args: &[&str]) -> Option<String> {
    git(dir, args)
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
}

fn pin_after(text: &str, dependency: &str) -> Option<String> {
    for line in text.lines() {
        if !line.contains("git =") || !line.contains(dependency) {
            continue;
        }
        let rev = line.find("rev = \"")?;
        let rest = &line[rev + 7..];
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    None
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftCheck {
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftSummary {
    pub checked: usize,
    pub ok: usize,
    pub drift: usize,
    pub pin_unavailable: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftReport {
    pub schema: String,
    pub ok: bool,
    pub source_root: String,
    pub checks: Vec<DriftCheck>,
    pub summary: DriftSummary,
}

fn push_check(
    checks: &mut Vec<DriftCheck>,
    repo: &str,
    rev: Option<&str>,
    id: Option<&str>,
    path: Option<&str>,
    status: &str,
    detail: impl Into<String>,
) {
    checks.push(DriftCheck {
        repo: repo.to_string(),
        rev: rev.map(str::to_string),
        id: id.map(str::to_string),
        path: path.map(str::to_string),
        status: status.to_string(),
        detail: detail.into(),
    });
}

/// Compare the catalog with the sibling repositories at their pinned commits.
pub fn drift(root: &Path, source_root: &Path) -> Result<DriftReport, RegistryError> {
    let registry = Registry::load(root)?;
    let pins = load_pins(root)?;
    let mut checks: Vec<DriftCheck> = Vec::new();

    for primitive in &registry.primitives {
        let rev = primitive.pinned_commit.as_str();
        if !is_hex40(rev) {
            push_check(
                &mut checks,
                &primitive.owner,
                Some(rev),
                Some(&primitive.id),
                Some(&primitive.source),
                "skipped",
                "pin is TBD-review-required",
            );
            continue;
        }
        let dir = checkout(source_root, &primitive.owner);
        if !dir.is_dir() {
            push_check(
                &mut checks,
                &primitive.owner,
                Some(rev),
                Some(&primitive.id),
                Some(&primitive.source),
                "pin-unavailable",
                format!("checkout missing: {}", dir.display()),
            );
            continue;
        }
        let commit = format!("{rev}^{{commit}}");
        if !git_ok(&dir, &["cat-file", "-e", &commit]) {
            push_check(
                &mut checks,
                &primitive.owner,
                Some(rev),
                Some(&primitive.id),
                Some(&primitive.source),
                "pin-unavailable",
                format!("commit {rev} is not in {}", dir.display()),
            );
            continue;
        }
        let spec = format!("{rev}:{}", primitive.source);
        if !git_ok(&dir, &["cat-file", "-e", &spec]) {
            push_check(
                &mut checks,
                &primitive.owner,
                Some(rev),
                Some(&primitive.id),
                Some(&primitive.source),
                "drift",
                "source path does not exist at the pin",
            );
            continue;
        }
        if let Some(component) = &primitive.component {
            let found = git_ok(
                &dir,
                &[
                    "grep",
                    "-F",
                    "-q",
                    "--",
                    component,
                    rev,
                    "--",
                    "contracts/src",
                ],
            );
            if !found {
                push_check(
                    &mut checks,
                    &primitive.owner,
                    Some(rev),
                    Some(&primitive.id),
                    Some(&primitive.source),
                    "drift",
                    format!("component {component} is not under contracts/src at the pin"),
                );
                continue;
            }
        }
        push_check(
            &mut checks,
            &primitive.owner,
            Some(rev),
            Some(&primitive.id),
            Some(&primitive.source),
            "ok",
            "source and component found at the pin",
        );
    }

    // Rust dependency pins recorded in each sibling's Cargo.toml.
    let expected_r_cash = pins
        .siblings
        .get("r-cash")
        .and_then(|pin| pin.rev.as_deref())
        .unwrap_or("");
    let expected_rholo = pins.crypto.get("rev").and_then(Value::as_str).unwrap_or("");
    for (name, pin) in &pins.siblings {
        let Some(rev) = pin.rev.as_deref().filter(|value| is_hex40(value)) else {
            continue;
        };
        let dir = checkout(source_root, name);
        if !dir.is_dir() {
            continue;
        }
        let Some(cargo) = git_text(&dir, &["show", &format!("{rev}:Cargo.toml")]) else {
            continue;
        };
        for (dependency, expected) in [
            ("rhovision-org/r-cash", expected_r_cash),
            ("rhovision-org/rholo", expected_rholo),
        ] {
            if expected.is_empty() {
                continue;
            }
            if name == "r-cash" && dependency == "rhovision-org/r-cash" {
                continue;
            }
            let actual = pin_after(&cargo, dependency).unwrap_or_default();
            let status = if actual == expected { "ok" } else { "drift" };
            push_check(
                &mut checks,
                name,
                Some(rev),
                None,
                Some("Cargo.toml"),
                status,
                format!("{dependency}: expected {expected}, found {actual}"),
            );
        }
    }

    // Consumer configuration pins (configs/*.json).
    let config_expectations: [(&str, &str, &str); 6] = [
        ("r-print", "configs/rcash.json", "r-cash"),
        ("r-print", "configs/rsure.json", "r-sure"),
        ("r-print", "configs/rgen.json", "r-gen"),
        ("r-sure", "configs/rcash.json", "r-cash"),
        ("r-sure", "configs/rgen.json", "r-gen"),
        ("r-share", "configs/rcash.json", "r-cash"),
    ];
    for (consumer, path, source) in config_expectations {
        let Some(pin) = pins.siblings.get(consumer) else {
            continue;
        };
        let Some(rev) = pin.rev.as_deref().filter(|value| is_hex40(value)) else {
            continue;
        };
        let dir = checkout(source_root, consumer);
        if !dir.is_dir() {
            continue;
        }
        let Some(text) = git_text(&dir, &["show", &format!("{rev}:{path}")]) else {
            push_check(
                &mut checks,
                consumer,
                Some(rev),
                None,
                Some(path),
                "drift",
                "config path does not exist at the pin",
            );
            continue;
        };
        let Ok(config) = serde_json::from_str::<Value>(&text) else {
            push_check(
                &mut checks,
                consumer,
                Some(rev),
                None,
                Some(path),
                "drift",
                "config is not JSON",
            );
            continue;
        };
        let actual = config
            .get("pinned_commit")
            .and_then(Value::as_str)
            .unwrap_or("");
        let expected = if source == "r-gen" {
            pins.siblings
                .get("r-gen")
                .and_then(|pin| pin.consumer_pins.get(consumer))
                .map(String::as_str)
                .unwrap_or("")
        } else {
            pins.siblings
                .get(source)
                .and_then(|pin| pin.rev.as_deref())
                .unwrap_or("")
        };
        let status = if expected.is_empty() {
            "skipped"
        } else if actual == expected {
            "ok"
        } else {
            "drift"
        };
        push_check(
            &mut checks,
            consumer,
            Some(rev),
            None,
            Some(path),
            status,
            format!("{source}: expected {expected}, found {actual}"),
        );
    }

    let ok = checks.iter().filter(|check| check.status == "ok").count();
    let drift_count = checks
        .iter()
        .filter(|check| check.status == "drift")
        .count();
    let unavailable = checks
        .iter()
        .filter(|check| check.status == "pin-unavailable")
        .count();
    Ok(DriftReport {
        schema: "rhobuilder.registry-drift/v1".to_string(),
        ok: drift_count == 0 && unavailable == 0,
        source_root: source_root.display().to_string(),
        checks,
        summary: DriftSummary {
            checked: ok + drift_count + unavailable,
            ok,
            drift: drift_count,
            pin_unavailable: unavailable,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir)
            .status()
            .expect("git");
        assert!(status.success(), "git {args:?}");
    }

    fn fixture() -> (tempfile::TempDir, tempfile::TempDir) {
        let registry_root = tempfile::tempdir().expect("registry");
        let source = tempfile::tempdir().expect("source");
        let repo = source.path().join("Rholphin");
        std::fs::create_dir_all(repo.join("contracts/src/core")).expect("dirs");
        std::fs::write(
            repo.join("contracts/src/core/AccountFactory.rho"),
            "// component:account-factory\nnew AccountFactory in { AccountFactory!(\"getOrCreate\") }\n",
        )
        .expect("source");
        git(&repo, &["init", "-b", "main"]);
        git(&repo, &["config", "user.email", "test@example.com"]);
        git(&repo, &["config", "user.name", "test"]);
        git(&repo, &["add", "-A"]);
        git(&repo, &["commit", "-m", "fixture"]);
        let rev = git_text(&repo, &["rev-parse", "HEAD"]).expect("rev");
        let rev = rev.trim().to_string();

        std::fs::write(
            registry_root.path().join("pins.json"),
            format!(
                r#"{{"schema":"rhobuilder.pins/v1","crypto":{{"rev":"{}"}},
                "siblings":{{"r-cash":{{"repo":"rhovision-org/r-cash","rev":"{rev}"}}}}}}"#,
                "841cb842876369deaa63e3b3e26c20d60e6c74c6"
            ),
        )
        .expect("pins");
        std::fs::write(
            registry_root.path().join("primitives.json"),
            format!(
                r#"{{"schema":"rhobuilder.registry/v1","primitives":[{{
                  "id":"rcash.account-factory","owner":"r-cash","kind":"accounts",
                  "component":"component:account-factory",
                  "source":"contracts/src/core/AccountFactory.rho",
                  "pinnedCommit":"{rev}","status":"tested","arms":["getOrCreate"],
                  "networks":{{"devnet":{{"verified":"unverified"}}}}}}],
                  "vertices":[],"edges":[],"invariants":[]}}"#
            ),
        )
        .expect("registry");
        (registry_root, source)
    }

    #[test]
    fn drift_reads_the_pinned_commit_and_finds_the_component() {
        let (registry_root, source) = fixture();
        let report = drift(registry_root.path(), source.path()).expect("drift");
        assert!(report.ok, "{report:?}");
        assert!(report
            .checks
            .iter()
            .any(|check| check.id.as_deref() == Some("rcash.account-factory")
                && check.status == "ok"));
    }

    #[test]
    fn a_missing_source_is_drift() {
        let (registry_root, source) = fixture();
        let text = std::fs::read_to_string(registry_root.path().join("primitives.json"))
            .expect("read")
            .replace("AccountFactory.rho", "Missing.rho");
        std::fs::write(registry_root.path().join("primitives.json"), text).expect("write");
        let report = drift(registry_root.path(), source.path()).expect("drift");
        assert!(!report.ok);
        assert!(report.checks.iter().any(|check| check.status == "drift"));
    }

    #[test]
    fn a_missing_pin_is_not_reported_as_drift() {
        let (registry_root, source) = fixture();
        let path = registry_root.path().join("primitives.json");
        let mut document: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("read")).expect("json");
        document["primitives"][0]["pinnedCommit"] = Value::String("0".repeat(40));
        std::fs::write(&path, document.to_string()).expect("write");
        let report = drift(registry_root.path(), source.path()).expect("drift");
        assert_eq!(report.summary.drift, 0);
        assert!(report.summary.pin_unavailable > 0, "{report:?}");
    }

    #[test]
    fn graph_is_breadth_first_and_bounded() {
        let registry = Registry {
            schema: "rhobuilder.registry/v1".into(),
            generated_at: None,
            pins: Value::Null,
            primitives: vec![],
            vertices: vec![
                Vertex {
                    id: "a".into(),
                    owner: "x".into(),
                    kind: "cell".into(),
                },
                Vertex {
                    id: "b".into(),
                    owner: "x".into(),
                    kind: "cell".into(),
                },
                Vertex {
                    id: "c".into(),
                    owner: "x".into(),
                    kind: "cell".into(),
                },
            ],
            edges: vec![
                Edge {
                    id: "ab".into(),
                    verb: "calls".into(),
                    from: "a".into(),
                    to: "b".into(),
                    kind: "call".into(),
                    arms: vec![],
                    source: String::new(),
                    status: String::new(),
                    optional: false,
                },
                Edge {
                    id: "bc".into(),
                    verb: "calls".into(),
                    from: "b".into(),
                    to: "c".into(),
                    kind: "call".into(),
                    arms: vec![],
                    source: String::new(),
                    status: String::new(),
                    optional: false,
                },
            ],
            invariants: vec![],
        };
        let view = registry.graph("a", 1).expect("graph");
        assert_eq!(view.vertices.len(), 2);
        assert_eq!(view.edges.len(), 1);
    }
}
