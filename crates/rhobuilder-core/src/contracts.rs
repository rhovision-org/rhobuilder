//! Generic contract patterns for a design without a worked example.
//!
//! The worked `bookable-service` example is hand-authored. Every other design gets a
//! real root, one linear cell per app resource, one cell per `new` component, and an
//! adapter per `reuse`/`extend` component. The generated `.rhobuilder-contracts.json`
//! describes install order, URI captures and required Root keys for the deploy planner.
//!
//! These are scaffolds, not implementations of the design's unique protocol. The flow
//! manifest keeps their arms `owed` until the author writes the model.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

use crate::design::{Component, Design, Resource};
use crate::scaffold::ScaffoldError;

#[derive(Debug, Clone)]
pub struct PrimitiveInfo {
    pub component: String,
    pub owner: String,
    pub resolve_arity: usize,
}

#[derive(Debug, Clone)]
pub struct GeneratedContracts {
    pub files: BTreeMap<String, Vec<u8>>,
    pub manifest: Value,
}

fn pascal(value: &str) -> String {
    let mut out = String::new();
    for part in value.split(|ch: char| !ch.is_ascii_alphanumeric()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.extend(chars);
        }
    }
    if out.is_empty() {
        "Component".to_string()
    } else {
        out
    }
}

fn snake(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_').to_string();
    if trimmed.is_empty() {
        "component".to_string()
    } else {
        trimmed
    }
}

fn upper(value: &str) -> String {
    snake(value).to_ascii_uppercase()
}

fn owner_root(owner: &str) -> Option<&'static str> {
    match owner {
        "r-cash" => Some("__RCASH_ROOT_URI__"),
        "r-gen" => Some("__RGEN_ROOT_URI__"),
        "r-share" => Some("__RSHARE_ROOT_URI__"),
        "r-sure" => Some("__RSURE_ROOT_URI__"),
        "r-print" => Some("__RPRINT_ROOT_URI__"),
        _ => None,
    }
}

fn resource_term(resource: &Resource) -> String {
    let contract = pascal(&resource.id);
    let lifecycle = serde_json::to_string(&resource.lifecycle).unwrap_or_else(|_| "[]".into());
    let template = r#"new rs(`rho:registry:insertArbitrary`), deployId(`rho:rchain:deployId`) in {
  new __CONTRACT__, state, uriCh in {
    state!({"id": "__ID__", "lifecycle": __LIFECYCLE__, "value": Nil}) |
    contract __CONTRACT__(@"get", ret) = {
      for (@s <<- state) { ret!(s) }
    } |
    contract __CONTRACT__(@"set", @value, ret) = {
      for (@s <- state) {
        state!(s.set("value", value)) | ret!(true)
      }
    } |
    contract __CONTRACT__(@"health", ret) = {
      ret!((true, "scaffold"))
    } |
    rs!(bundle+{*__CONTRACT__}, *uriCh) |
    for (@uri <- uriCh) { deployId!([uri]) }
  }
}
"#;
    template
        .replace("__CONTRACT__", &contract)
        .replace("__ID__", &resource.id)
        .replace("__LIFECYCLE__", &lifecycle)
}

fn new_component_term(component: &Component) -> String {
    let contract = pascal(&component.id);
    let arms = serde_json::to_string(&component.arms).unwrap_or_else(|_| "[]".into());
    let template = r#"new rs(`rho:registry:insertArbitrary`), deployId(`rho:rchain:deployId`) in {
  new __CONTRACT__, state, uriCh in {
    state!({"id": "__ID__", "arms": __ARMS__, "value": Nil}) |
    contract __CONTRACT__(@"get", ret) = {
      for (@s <<- state) { ret!(s) }
    } |
    contract __CONTRACT__(@"set", @value, ret) = {
      for (@s <- state) {
        state!(s.set("value", value)) | ret!(true)
      }
    } |
    contract __CONTRACT__(@"health", ret) = {
      ret!((true, "scaffold"))
    } |
    rs!(bundle+{*__CONTRACT__}, *uriCh) |
    for (@uri <- uriCh) { deployId!([uri]) }
  }
}
"#;
    template
        .replace("__CONTRACT__", &contract)
        .replace("__ID__", &component.id)
        .replace("__ARMS__", &arms)
}

fn adapter_term(component: &Component, primitive: &PrimitiveInfo) -> Result<String, ScaffoldError> {
    let Some(root) = owner_root(&primitive.owner) else {
        return Err(ScaffoldError::Unresolved(format!(
            "no adapter root pattern for owner {}",
            primitive.owner
        )));
    };
    let contract = format!("{}Adapter", pascal(&component.id));
    let resolve = if primitive.resolve_arity >= 3 {
        "Root!(\"resolve\", key, 1, *entryCh)"
    } else {
        "Root!(\"resolve\", key, *entryCh)"
    };
    let template = r#"new rl(`rho:registry:lookup`), rs(`rho:registry:insertArbitrary`),
    deployId(`rho:rchain:deployId`), __CONTRACT__, uriCh, resolveKey in {
  contract resolveKey(@key, ret) = {
    new rootCh, entryCh in {
      rl!(`__ROOT__`, *rootCh) |
      for (Root <- rootCh) {
        __RESOLVE__ |
        for (@entry <- entryCh) { ret!(entry) }
      }
    }
  } |
  contract __CONTRACT__(@"health", ret) = {
    resolveKey!("__COMPONENT__", *ret)
  } |
  rs!(bundle+{*__CONTRACT__}, *uriCh) |
  for (@uri <- uriCh) { deployId!([uri]) }
}
"#;
    Ok(template
        .replace("__CONTRACT__", &contract)
        .replace("__ROOT__", root)
        .replace("__RESOLVE__", resolve)
        .replace("__COMPONENT__", &primitive.component))
}

fn root_term(entries: &Value) -> String {
    let entries = serde_json::to_string(entries).unwrap_or_else(|_| "{}".into());
    let template = r#"new rs(`rho:registry:insertArbitrary`), deployId(`rho:rchain:deployId`) in {
  new Root, state, uriCh in {
    state!({"entries": __ENTRIES__}) |
    contract Root(@"resolve", @key, @minVersion, ret) = {
      for (@s <<- state) {
        match s.get("entries").get(key) {
          Nil => { ret!((false, "unknown component")) }
          e => {
            if (e.get("version") < minVersion) {
              ret!((false, "stale"))
            } else {
              ret!((true, e.get("uri"), e.get("version")))
            }
          }
        }
      }
    } |
    contract Root(@"get", @key, ret) = {
      for (@s <<- state) {
        match s.get("entries").get(key) {
          Nil => { ret!((false, "unknown component")) }
          e => { ret!((true, e)) }
        }
      }
    } |
    contract Root(@"keys", ret) = {
      for (@s <<- state) { ret!((true, s.get("entries").keys().toList())) }
    } |
    rs!(bundle+{*Root}, *uriCh) |
    for (@rootUri <- uriCh) {
      for (@s <- state) {
        state!(s.set("entries", s.get("entries").set("component:root",
          {"uri": rootUri, "version": 1, "kind": "root"})))
      } |
      deployId!([rootUri])
    }
  }
}
"#;
    template.replace("__ENTRIES__", &entries)
}

/// Generate the generic contract set for a design. The caller should skip this when a
/// worked example directory exists for the slug.
pub fn generate(
    design: &Design,
    primitives: &BTreeMap<String, PrimitiveInfo>,
) -> Result<GeneratedContracts, ScaffoldError> {
    let mut files = BTreeMap::new();
    let mut entries = serde_json::Map::new();
    let mut required_keys = BTreeSet::new();
    let mut dependencies = BTreeSet::new();
    let mut steps = Vec::new();
    let mut used_keys = BTreeSet::new();

    for resource in &design.resources {
        let key = format!("component:{}", resource.id);
        if !used_keys.insert(key.clone()) {
            return Err(ScaffoldError::Unresolved(format!(
                "duplicate component key {key}"
            )));
        }
        let placeholder = format!("__{}_URI__", upper(&resource.id));
        let file = format!("10_{}.rho", snake(&resource.id));
        files.insert(
            format!("contracts/src/{file}"),
            resource_term(resource).into_bytes(),
        );
        entries.insert(
            key.clone(),
            json!({"uri": placeholder, "version": 1, "kind": "resource"}),
        );
        required_keys.insert(key);
        steps.push(json!({
            "name": file.trim_end_matches(".rho"),
            "term": file,
            "captures": [{"placeholder": placeholder, "from": "uris", "index": 0}],
        }));
    }

    for component in &design.components {
        match component.tag.as_str() {
            "reuse" | "extend" => {
                let primitive_id = component.primitive.as_deref().ok_or_else(|| {
                    ScaffoldError::Unresolved(format!(
                        "{} has no primitive for an adapter",
                        component.id
                    ))
                })?;
                let primitive = primitives.get(primitive_id).ok_or_else(|| {
                    ScaffoldError::Unresolved(format!(
                        "{} reuses {primitive_id}, but the registry has no component key",
                        component.id
                    ))
                })?;
                let key = format!("component:{}-adapter", component.id);
                if !used_keys.insert(key.clone()) {
                    return Err(ScaffoldError::Unresolved(format!(
                        "duplicate component key {key}"
                    )));
                }
                let placeholder = format!("__{}_URI__", upper(&component.id));
                let file = format!("20_{}.rho", snake(&component.id));
                files.insert(
                    format!("contracts/src/{file}"),
                    adapter_term(component, primitive)?.into_bytes(),
                );
                entries.insert(
                    key.clone(),
                    json!({"uri": placeholder, "version": 1, "kind": "adapter"}),
                );
                required_keys.insert(key);
                dependencies.insert(primitive.owner.clone());
                steps.push(json!({
                    "name": file.trim_end_matches(".rho"),
                    "term": file,
                    "captures": [{"placeholder": placeholder, "from": "uris", "index": 0}],
                }));
            }
            "new" => {
                let key = format!("component:{}", component.id);
                if !used_keys.insert(key.clone()) {
                    return Err(ScaffoldError::Unresolved(format!(
                        "duplicate component key {key}"
                    )));
                }
                let placeholder = format!("__{}_URI__", upper(&component.id));
                let file = format!("30_{}.rho", snake(&component.id));
                files.insert(
                    format!("contracts/src/{file}"),
                    new_component_term(component).into_bytes(),
                );
                entries.insert(
                    key.clone(),
                    json!({"uri": placeholder, "version": 1, "kind": "scaffold"}),
                );
                required_keys.insert(key);
                steps.push(json!({
                    "name": file.trim_end_matches(".rho"),
                    "term": file,
                    "captures": [{"placeholder": placeholder, "from": "uris", "index": 0}],
                }));
            }
            _ => {
                return Err(ScaffoldError::Unresolved(format!(
                    "{} has unknown tag {}",
                    component.id, component.tag
                )));
            }
        }
    }

    let root_file = "00_root.rho".to_string();
    files.insert(
        format!("contracts/src/{root_file}"),
        root_term(&Value::Object(entries)).into_bytes(),
    );
    steps.push(json!({
        "name": "00_root",
        "term": root_file,
        "root": true,
        "captures": [],
    }));
    required_keys.insert("component:root".to_string());

    let manifest = json!({
        "schema": "rhobuilder.contracts/v1",
        "rootKey": "component:root",
        "dependencies": dependencies.into_iter().collect::<Vec<_>>(),
        "requiredKeys": required_keys.into_iter().collect::<Vec<_>>(),
        "steps": steps,
        "note": "generic scaffold; design arms remain owed until the author writes the protocol",
    });
    files.insert(
        ".rhobuilder-contracts.json".to_string(),
        serde_json::to_vec_pretty(&manifest)
            .map_err(|error| ScaffoldError::Unresolved(error.to_string()))?,
    );
    Ok(GeneratedContracts { files, manifest })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design::{Component, Design, DesignGraph, Resource};

    fn design() -> Design {
        Design {
            schema: "rhobuilder.design/v1".into(),
            app: crate::design::App {
                name: "Ledger".into(),
                slug: "ledger".into(),
                summary: "test".into(),
                networks: vec!["devnet".into()],
            },
            unique_value: "notes".into(),
            actors: vec![],
            resources: vec![Resource {
                id: "ledger".into(),
                owner: "app".into(),
                state_cell: "ledgerState".into(),
                lifecycle: vec!["open".into(), "closed".into()],
            }],
            components: vec![
                Component {
                    id: "payments".into(),
                    tag: "reuse".into(),
                    primitive: Some("rcash.account-factory".into()),
                    adapter: Some("payments-adapter.rho".into()),
                    mode: None,
                    kind: None,
                    why_unique: None,
                    why_extend: None,
                    no_registered_alternative: vec![],
                    arms: vec![],
                    invariants: vec![],
                    lean_status: None,
                    lean: None,
                    override_: None,
                },
                Component {
                    id: "ledger-notes".into(),
                    tag: "new".into(),
                    primitive: None,
                    adapter: None,
                    mode: None,
                    kind: Some("custom".into()),
                    why_unique: Some("notes".into()),
                    why_extend: None,
                    no_registered_alternative: vec!["rcash.account-factory".into()],
                    arms: vec!["record".into()],
                    invariants: vec![],
                    lean_status: None,
                    lean: None,
                    override_: None,
                },
            ],
            graph: DesignGraph::default(),
            conservation: vec![],
            concurrency: vec![],
            remedies: vec![],
            acceptance: vec![],
        }
    }

    #[test]
    fn generates_root_resource_and_adapters() {
        let mut primitives = BTreeMap::new();
        primitives.insert(
            "rcash.account-factory".into(),
            PrimitiveInfo {
                component: "component:account-factory".into(),
                owner: "r-cash".into(),
                resolve_arity: 3,
            },
        );
        let generated = generate(&design(), &primitives).unwrap_or_else(|_| panic!("generate"));
        assert!(generated.files.contains_key("contracts/src/00_root.rho"));
        assert!(generated.files.contains_key("contracts/src/10_ledger.rho"));
        assert!(generated
            .files
            .contains_key("contracts/src/20_payments.rho"));
        assert!(generated
            .files
            .contains_key("contracts/src/30_ledger_notes.rho"));
        assert_eq!(generated.manifest["dependencies"][0], "r-cash");
    }

    #[test]
    fn unknown_primitive_is_refused() {
        let error = generate(&design(), &BTreeMap::new());
        assert!(matches!(error, Err(ScaffoldError::Unresolved(_))));
    }
}
