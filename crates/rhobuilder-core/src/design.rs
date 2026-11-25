//! The `design.json` artifact and its schema-level validation.
//!
//! This module checks the document on its own terms. Whether a `new` component
//! duplicates a registered primitive is the reuse gate's job, because that needs the
//! registry; see `rhobuilder-registry::reuse`.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const DESIGN_SCHEMA: &str = "rhobuilder.design/v1";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Design {
    #[serde(default)]
    pub schema: String,
    pub app: App,
    #[serde(default)]
    pub unique_value: String,
    #[serde(default)]
    pub actors: Vec<Actor>,
    #[serde(default)]
    pub resources: Vec<Resource>,
    #[serde(default)]
    pub components: Vec<Component>,
    #[serde(default)]
    pub graph: DesignGraph,
    #[serde(default)]
    pub conservation: Vec<Conservation>,
    #[serde(default)]
    pub concurrency: Vec<Concurrency>,
    #[serde(default)]
    pub remedies: Vec<Value>,
    #[serde(default)]
    pub acceptance: Vec<Acceptance>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub networks: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub state_cell: String,
    #[serde(default)]
    pub lifecycle: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    #[serde(default)]
    pub id: String,
    /// `reuse`, `extend` or `new`.
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub primitive: Option<String>,
    #[serde(default)]
    pub adapter: Option<String>,
    /// `adapter-only` or `new-version-proposal` for an extension.
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub why_unique: Option<String>,
    #[serde(default)]
    pub why_extend: Option<String>,
    #[serde(default)]
    pub no_registered_alternative: Vec<String>,
    #[serde(default)]
    pub arms: Vec<String>,
    #[serde(default)]
    pub invariants: Vec<String>,
    #[serde(default, rename = "override")]
    pub override_: Option<Override>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Override {
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub approved_by: String,
    #[serde(default)]
    pub approved_at: String,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignGraph {
    #[serde(default)]
    pub vertices: Vec<DesignVertex>,
    #[serde(default)]
    pub edges: Vec<DesignEdge>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignVertex {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignEdge {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub verb: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conservation {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub assertion: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Concurrency {
    #[serde(default)]
    pub risk: String,
    #[serde(default)]
    pub mechanism: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Acceptance {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub given: String,
    #[serde(default)]
    pub when: String,
    #[serde(default)]
    pub then: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignCheck {
    pub id: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignValidation {
    pub schema: String,
    pub ok: bool,
    pub checks: Vec<DesignCheck>,
}

#[derive(Debug, Error)]
pub enum DesignError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("invalid design JSON: {0}")]
    Json(String),
}

pub fn load(path: &Path) -> Result<Design, DesignError> {
    let text = std::fs::read_to_string(path).map_err(|source| DesignError::Io {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|error| DesignError::Json(error.to_string()))
}

fn is_kebab(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

struct Checks(Vec<DesignCheck>);

impl Checks {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn push(&mut self, id: &str, status: &str, detail: impl Into<String>) {
        self.0.push(DesignCheck {
            id: id.to_string(),
            status: status.to_string(),
            detail: detail.into(),
        });
    }

    fn ok(&mut self, id: &str, detail: impl Into<String>) {
        self.push(id, "ok", detail);
    }

    fn warn(&mut self, id: &str, detail: impl Into<String>) {
        self.push(id, "warn", detail);
    }

    fn fail(&mut self, id: &str, detail: impl Into<String>) {
        self.push(id, "fail", detail);
    }

    fn finish(self) -> DesignValidation {
        let ok = !self.0.iter().any(|check| check.status == "fail");
        DesignValidation {
            schema: "rhobuilder.design-validation/v1".to_string(),
            ok,
            checks: self.0,
        }
    }
}

impl Design {
    pub fn validate(&self) -> DesignValidation {
        let mut checks = Checks::new();
        if self.schema == DESIGN_SCHEMA {
            checks.ok("schema", DESIGN_SCHEMA);
        } else {
            checks.fail(
                "schema",
                format!("expected {DESIGN_SCHEMA}, found {:?}", self.schema),
            );
        }
        if self.app.name.trim().is_empty() {
            checks.fail("app.name", "name is required");
        } else {
            checks.ok("app.name", self.app.name.clone());
        }
        if is_kebab(&self.app.slug) {
            checks.ok("app.slug", self.app.slug.clone());
        } else {
            checks.fail("app.slug", "slug must be kebab-case");
        }
        if self.app.summary.trim().is_empty() {
            checks.fail("app.summary", "summary is required");
        }
        if self.app.networks.is_empty() {
            checks.fail("app.networks", "at least one target network is required");
        } else {
            checks.ok("app.networks", self.app.networks.join(", "));
        }
        if self.unique_value.trim().is_empty() {
            checks.fail(
                "uniqueValue",
                "state the unique value; a commodity app should reuse an existing one",
            );
        } else {
            checks.ok("uniqueValue", "stated");
        }

        let mut actor_ids = std::collections::BTreeSet::new();
        for actor in &self.actors {
            if actor.id.trim().is_empty() {
                checks.fail("actors", "actor id is required");
            } else if !actor_ids.insert(actor.id.clone()) {
                checks.fail("actors", format!("duplicate actor {}", actor.id));
            }
        }
        if self.actors.is_empty() {
            checks.warn("actors", "no actors declared");
        }

        let mut resource_ids = std::collections::BTreeSet::new();
        for resource in &self.resources {
            if resource.id.trim().is_empty() {
                checks.fail("resources", "resource id is required");
            } else if !resource_ids.insert(resource.id.clone()) {
                checks.fail("resources", format!("duplicate resource {}", resource.id));
            }
            if resource.state_cell.trim().is_empty() {
                checks.fail("resources", format!("{} has no stateCell", resource.id));
            }
        }

        let mut component_ids = std::collections::BTreeSet::new();
        for component in &self.components {
            if component.id.trim().is_empty() {
                checks.fail("components", "component id is required");
                continue;
            }
            if !component_ids.insert(component.id.clone()) {
                checks.fail(
                    "components",
                    format!("duplicate component {}", component.id),
                );
                continue;
            }
            match component.tag.as_str() {
                "reuse" => {
                    if component.primitive.as_deref().unwrap_or("").is_empty() {
                        checks.fail(
                            "components",
                            format!("{} is reuse but names no primitive", component.id),
                        );
                    } else {
                        checks.ok(
                            "components",
                            format!(
                                "{} reuses {}",
                                component.id,
                                component.primitive.as_deref().unwrap_or("")
                            ),
                        );
                    }
                }
                "extend" => {
                    if component.primitive.as_deref().unwrap_or("").is_empty() {
                        checks.fail(
                            "components",
                            format!("{} is extend but names no primitive", component.id),
                        );
                    }
                    if component.adapter.as_deref().unwrap_or("").is_empty() {
                        checks.fail(
                            "components",
                            format!("{} is extend but names no adapter", component.id),
                        );
                    }
                    if component
                        .why_extend
                        .as_deref()
                        .unwrap_or("")
                        .trim()
                        .is_empty()
                    {
                        checks.fail(
                            "components",
                            format!("{} is extend but has no whyExtend", component.id),
                        );
                    }
                    let mode = component.mode.as_deref().unwrap_or("adapter-only");
                    if !["adapter-only", "new-version-proposal"].contains(&mode) {
                        checks.fail(
                            "components",
                            format!("{} has unknown extend mode {mode:?}", component.id),
                        );
                    }
                }
                "new" => {
                    if component
                        .why_unique
                        .as_deref()
                        .unwrap_or("")
                        .trim()
                        .is_empty()
                    {
                        checks.fail(
                            "components",
                            format!("{} is new but has no whyUnique", component.id),
                        );
                    }
                    if component.no_registered_alternative.is_empty() {
                        checks.fail(
                            "components",
                            format!("{} is new but checked no alternatives", component.id),
                        );
                    }
                    if component.arms.is_empty() {
                        checks.fail(
                            "components",
                            format!("{} is new but declares no arms", component.id),
                        );
                    }
                }
                other => checks.fail(
                    "components",
                    format!("{} has unknown tag {other:?}", component.id),
                ),
            }
            if let Some(override_) = &component.override_ {
                if override_.reason.trim().is_empty()
                    || override_.approved_by.trim().is_empty()
                    || override_.approved_at.trim().is_empty()
                {
                    checks.fail(
                        "components",
                        format!(
                            "{} override needs reason, approvedBy and approvedAt",
                            component.id
                        ),
                    );
                } else {
                    checks.ok(
                        "components",
                        format!("{} carries an approved override", component.id),
                    );
                }
            }
        }
        if self.components.is_empty() {
            checks.fail("components", "at least one component is required");
        }

        let mut vertex_ids = std::collections::BTreeSet::new();
        for vertex in &self.graph.vertices {
            if vertex.id.trim().is_empty() {
                checks.fail("graph.vertices", "vertex id is required");
            } else if !vertex_ids.insert(vertex.id.clone()) {
                checks.fail("graph.vertices", format!("duplicate vertex {}", vertex.id));
            }
        }
        let mut edge_ids = std::collections::BTreeSet::new();
        for edge in &self.graph.edges {
            if edge.id.trim().is_empty() {
                checks.fail("graph.edges", "edge id is required");
            } else if !edge_ids.insert(edge.id.clone()) {
                checks.fail("graph.edges", format!("duplicate edge {}", edge.id));
            }
            if edge.from.trim().is_empty() || edge.to.trim().is_empty() {
                checks.fail("graph.edges", format!("{} needs from and to", edge.id));
            }
        }

        for item in &self.conservation {
            if item.path.trim().is_empty() || item.assertion.trim().is_empty() {
                checks.fail("conservation", "each path needs an assertion");
            }
        }
        if self.conservation.is_empty() {
            checks.warn(
                "conservation",
                "no value-conservation assertions; state them if any arm moves value",
            );
        }
        for item in &self.concurrency {
            if item.risk.trim().is_empty() || item.mechanism.trim().is_empty() {
                checks.fail("concurrency", "each risk needs a mechanism");
            }
        }
        for item in &self.acceptance {
            if item.given.trim().is_empty()
                || item.when.trim().is_empty()
                || item.then.trim().is_empty()
            {
                checks.fail("acceptance", format!("{} is incomplete", item.id));
            }
        }
        if self.acceptance.is_empty() {
            checks.fail("acceptance", "at least one given/when/then is required");
        }

        checks.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal() -> Design {
        serde_json::from_value(serde_json::json!({
            "schema": DESIGN_SCHEMA,
            "app": {"name": "Demo", "slug": "demo", "summary": "A demo", "networks": ["devnet"]},
            "uniqueValue": "A unique coordination protocol",
            "components": [{
                "id": "payments", "tag": "reuse", "primitive": "rcash.account-factory"
            }],
            "acceptance": [{"id": "a1", "given": "g", "when": "w", "then": "t"}]
        }))
        .expect("design")
    }

    #[test]
    fn a_minimal_design_passes() {
        let report = minimal().validate();
        assert!(report.ok, "{report:?}");
    }

    #[test]
    fn a_bad_slug_fails() {
        let mut design = minimal();
        design.app.slug = "Not_Kebab".into();
        assert!(!design.validate().ok);
    }

    #[test]
    fn a_new_component_without_alternatives_fails() {
        let mut design = minimal();
        design.components.push(Component {
            id: "escrow".into(),
            tag: "new".into(),
            primitive: None,
            adapter: None,
            mode: None,
            kind: Some("escrow".into()),
            why_unique: Some("unique".into()),
            why_extend: None,
            no_registered_alternative: vec![],
            arms: vec!["fund".into()],
            invariants: vec![],
            override_: None,
        });
        assert!(!design.validate().ok);
    }

    #[test]
    fn an_extend_needs_an_adapter_and_reason() {
        let mut design = minimal();
        design.components.push(Component {
            id: "cover".into(),
            tag: "extend".into(),
            primitive: Some("rsure.policy".into()),
            adapter: None,
            mode: None,
            kind: None,
            why_unique: None,
            why_extend: None,
            no_registered_alternative: vec![],
            arms: vec![],
            invariants: vec![],
            override_: None,
        });
        assert!(!design.validate().ok);
    }
}
