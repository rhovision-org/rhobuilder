//! The reuse gate: refuse a `new` component that duplicates a registered commodity
//! primitive, and validate that reuse/extension references resolve.
//!
//! `extend` is adapter-only or a sibling proposal. Deployed Rholang is immutable; this
//! gate never treats an extension as permission to edit a sibling repository.

use std::collections::BTreeSet;

use rhobuilder_core::design::{Component, Design};
use serde::Serialize;

use crate::{Edge, Registry, Vertex};

const CORE_OWNERS: [&str; 6] = [
    "r-cash", "r-sure", "r-gen", "r-share", "r-print", "r-bridge",
];
const PROTECTED_KINDS: [&str; 10] = [
    "accounts",
    "payments",
    "escrow",
    "insurance-claim",
    "knowledge-claim",
    "provenance",
    "identity",
    "booking",
    "publication",
    "bridge",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReuseCheck {
    pub id: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    pub component: String,
    pub primitive: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReuseReport {
    pub schema: String,
    pub verdict: String,
    pub ok: bool,
    pub checks: Vec<ReuseCheck>,
    pub reuse: Vec<String>,
    pub extend: Vec<String>,
    pub new: Vec<String>,
    pub conflicts: Vec<Conflict>,
    pub missing: Vec<String>,
    pub graph: ReuseGraph,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReuseGraph {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
}

struct Gate {
    checks: Vec<ReuseCheck>,
    conflicts: Vec<Conflict>,
    missing: Vec<String>,
}

impl Gate {
    fn new() -> Self {
        Self {
            checks: Vec::new(),
            conflicts: Vec::new(),
            missing: Vec::new(),
        }
    }

    fn ok(&mut self, id: &str, detail: impl Into<String>) {
        self.checks.push(ReuseCheck {
            id: id.to_string(),
            status: "ok".into(),
            detail: detail.into(),
        });
    }

    fn warn(&mut self, id: &str, detail: impl Into<String>) {
        self.checks.push(ReuseCheck {
            id: id.to_string(),
            status: "warn".into(),
            detail: detail.into(),
        });
    }

    fn fail(&mut self, id: &str, detail: impl Into<String>) {
        self.checks.push(ReuseCheck {
            id: id.to_string(),
            status: "fail".into(),
            detail: detail.into(),
        });
    }

    fn conflict(&mut self, component: &str, primitive: &str, detail: impl Into<String>) {
        self.conflicts.push(Conflict {
            component: component.to_string(),
            primitive: primitive.to_string(),
            detail: detail.into(),
        });
    }
}

fn has_override(component: &Component) -> bool {
    component.override_.as_ref().is_some_and(|override_| {
        !override_.reason.trim().is_empty()
            && !override_.approved_by.trim().is_empty()
            && !override_.approved_at.trim().is_empty()
    })
}

fn check_reuse(gate: &mut Gate, design: &Design, registry: &Registry, component: &Component) {
    let Some(id) = component.primitive.as_deref() else {
        gate.fail("reuse", format!("{} names no primitive", component.id));
        return;
    };
    let primitive = match registry.get(id) {
        Ok(primitive) => primitive,
        Err(_) => {
            gate.missing.push(id.to_string());
            gate.fail(
                "reuse",
                format!("{} reuses {id}, which is not registered", component.id),
            );
            return;
        }
    };
    match primitive
        .policy
        .as_ref()
        .and_then(|policy| policy.reuse.as_deref())
    {
        Some("forbidden") => gate.fail(
            "reuse",
            format!("{} reuses {id}, whose policy forbids reuse", component.id),
        ),
        Some("discouraged") => gate.warn(
            "reuse",
            format!(
                "{} reuses {id}, whose policy discourages reuse",
                component.id
            ),
        ),
        _ => gate.ok("reuse", format!("{} -> {id}", component.id)),
    }
    for network in &design.app.networks {
        match primitive.networks.get(network) {
            Some(entry) if entry.verified == "missing" => gate.warn(
                "network",
                format!("{id} has no verified root on {network}; the app cannot deploy there yet"),
            ),
            Some(entry) if entry.verified == "unverified" => gate.warn(
                "network",
                format!("{id} is unverified on {network}; verify before a deploy plan"),
            ),
            Some(_) => {}
            None => gate.warn(
                "network",
                format!("{id} has no catalog entry for {network}"),
            ),
        }
    }
}

fn check_extend(gate: &mut Gate, registry: &Registry, component: &Component) {
    let Some(id) = component.primitive.as_deref() else {
        gate.fail("extend", format!("{} names no primitive", component.id));
        return;
    };
    let primitive = match registry.get(id) {
        Ok(primitive) => primitive,
        Err(_) => {
            gate.missing.push(id.to_string());
            gate.fail(
                "extend",
                format!("{} extends {id}, which is not registered", component.id),
            );
            return;
        }
    };
    let mode = component.mode.as_deref().unwrap_or("adapter-only");
    if mode == "new-version-proposal" {
        gate.warn(
            "extend",
            format!(
                "{} proposes a new version of {id}; this is not a deployed adapter and does not modify the sibling",
                component.id
            ),
        );
        return;
    }
    let adapter_allowed = primitive
        .policy
        .as_ref()
        .is_some_and(|policy| policy.extension.iter().any(|item| item == "adapter-only"));
    if adapter_allowed {
        gate.ok(
            "extend",
            format!("{} adapts {id} without modifying the sibling", component.id),
        );
    } else {
        gate.fail(
            "extend",
            format!("{id} does not allow an adapter-only extension"),
        );
    }
}

fn check_new(gate: &mut Gate, registry: &Registry, component: &Component) {
    let kind = component.kind.as_deref().unwrap_or("");
    let mut existing: Vec<&str> = registry
        .primitives
        .iter()
        .filter(|primitive| primitive.kind == kind)
        .filter(|primitive| {
            primitive
                .policy
                .as_ref()
                .and_then(|policy| policy.reuse.as_deref())
                == Some("preferred")
        })
        .map(|primitive| primitive.id.as_str())
        .collect();
    for alternative in &component.no_registered_alternative {
        match registry.get(alternative) {
            Ok(primitive) => {
                if primitive.kind == kind {
                    existing.push(&primitive.id);
                }
            }
            Err(_) => {
                gate.missing.push(alternative.clone());
                gate.fail(
                    "new",
                    format!(
                        "{} lists {alternative} as a checked alternative, but it is not registered",
                        component.id
                    ),
                );
            }
        }
    }
    if !PROTECTED_KINDS.contains(&kind) {
        gate.warn(
            "new",
            format!(
                "{} has kind {:?}; the protected-domain check cannot classify it",
                component.id, kind
            ),
        );
        return;
    }
    existing.sort_unstable();
    existing.dedup();
    if existing.is_empty() {
        gate.ok(
            "new",
            format!(
                "{} has no preferred registered alternative of kind {kind}",
                component.id
            ),
        );
        return;
    }
    if has_override(component) {
        gate.warn(
            "new",
            format!(
                "{} overrides existing {}; the override is recorded",
                component.id,
                existing.join(", ")
            ),
        );
    } else {
        for primitive in existing {
            gate.conflict(
                &component.id,
                primitive,
                format!(
                    "{} is a new {kind} primitive but {primitive} already owns that domain; reuse or extend it, or record an approved override",
                    component.id
                ),
            );
        }
        gate.fail(
            "new",
            format!(
                "{} duplicates a registered commodity primitive",
                component.id
            ),
        );
    }
}

/// Run the gate. The report is a refusal if any check fails or any conflict remains.
pub fn check(design: &Design, registry: &Registry) -> ReuseReport {
    let mut gate = Gate::new();
    let mut reuse = Vec::new();
    let mut extend = Vec::new();
    let mut new = Vec::new();

    let mut known_vertices: BTreeSet<String> = registry
        .vertices
        .iter()
        .map(|vertex| vertex.id.clone())
        .collect();
    for vertex in &design.graph.vertices {
        known_vertices.insert(vertex.id.clone());
    }
    let owner_of = |id: &str| -> Option<&str> {
        design
            .graph
            .vertices
            .iter()
            .find(|vertex| vertex.id == id)
            .map(|vertex| vertex.owner.as_str())
            .or_else(|| {
                registry
                    .vertices
                    .iter()
                    .find(|vertex| vertex.id == id)
                    .map(|vertex| vertex.owner.as_str())
            })
    };

    for component in &design.components {
        match component.tag.as_str() {
            "reuse" => {
                reuse.push(component.id.clone());
                check_reuse(&mut gate, design, registry, component);
            }
            "extend" => {
                extend.push(component.id.clone());
                check_extend(&mut gate, registry, component);
            }
            "new" => {
                new.push(component.id.clone());
                check_new(&mut gate, registry, component);
            }
            other => gate.fail("tag", format!("{} has unknown tag {other:?}", component.id)),
        }
    }

    for edge in &design.graph.edges {
        if !known_vertices.contains(&edge.from) || !known_vertices.contains(&edge.to) {
            gate.fail(
                "graph",
                format!(
                    "edge {} names an unknown vertex ({} -> {})",
                    edge.id, edge.from, edge.to
                ),
            );
            continue;
        }
        let from_owner = owner_of(&edge.from).unwrap_or("app");
        let to_owner = owner_of(&edge.to).unwrap_or("app");
        let core_to_app = CORE_OWNERS.contains(&from_owner) && !CORE_OWNERS.contains(&to_owner);
        if core_to_app && !edge.optional {
            gate.fail(
                "graph",
                format!(
                    "edge {} goes from a core primitive ({from_owner}) to an app vertex ({to_owner}); it must be optional",
                    edge.id
                ),
            );
        }
    }

    let ok = !gate.checks.iter().any(|check| check.status == "fail") && gate.conflicts.is_empty();
    ReuseReport {
        schema: "rhobuilder.reuse-report/v1".to_string(),
        verdict: if ok { "pass" } else { "refuse" }.to_string(),
        ok,
        checks: gate.checks,
        reuse,
        extend,
        new,
        conflicts: gate.conflicts,
        missing: gate.missing,
        graph: ReuseGraph {
            vertices: registry.vertices.clone(),
            edges: registry.edges.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhobuilder_core::design::Design;

    fn registry() -> Registry {
        Registry::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../registry"))
            .expect("registry")
    }

    fn design(components: serde_json::Value) -> Design {
        serde_json::from_value(serde_json::json!({
            "schema": "rhobuilder.design/v1",
            "app": {"name": "Demo", "slug": "demo", "summary": "A demo", "networks": ["testnet"]},
            "uniqueValue": "A unique coordination protocol",
            "components": components,
            "acceptance": [{"id": "a1", "given": "g", "when": "w", "then": "t"}]
        }))
        .expect("design")
    }

    #[test]
    fn reusing_a_registered_primitive_passes() {
        let design = design(serde_json::json!([
            {"id": "payments", "tag": "reuse", "primitive": "rcash.account-factory"}
        ]));
        let report = check(&design, &registry());
        assert!(report.ok, "{report:?}");
        assert_eq!(report.verdict, "pass");
    }

    #[test]
    fn a_new_escrow_without_an_override_is_refused() {
        let design = design(serde_json::json!([
            {"id": "my-escrow", "tag": "new", "kind": "escrow",
             "whyUnique": "unique", "noRegisteredAlternative": ["rsure.policy"],
             "arms": ["fund"]}
        ]));
        let report = check(&design, &registry());
        assert!(!report.ok, "{report:?}");
        assert!(report
            .conflicts
            .iter()
            .any(|conflict| conflict.primitive == "rsure.policy"));
    }

    #[test]
    fn a_new_escrow_with_an_override_passes_with_a_warning() {
        let design = design(serde_json::json!([
            {"id": "my-escrow", "tag": "new", "kind": "escrow",
             "whyUnique": "unique", "noRegisteredAlternative": ["rsure.policy"],
             "arms": ["fund"],
             "override": {"reason": "different remedy", "approvedBy": "PatrickMockridge",
                          "approvedAt": "2026-11-25", "scope": "local-app-only"}}
        ]));
        let report = check(&design, &registry());
        assert!(report.ok, "{report:?}");
        assert!(report
            .checks
            .iter()
            .any(|check| check.id == "new" && check.status == "warn"));
    }

    #[test]
    fn a_missing_reuse_target_is_refused() {
        let design = design(serde_json::json!([
            {"id": "payments", "tag": "reuse", "primitive": "no.such.primitive"}
        ]));
        let report = check(&design, &registry());
        assert!(!report.ok);
        assert!(report.missing.contains(&"no.such.primitive".to_string()));
    }
}
