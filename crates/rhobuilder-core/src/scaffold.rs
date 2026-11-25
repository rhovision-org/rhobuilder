//! Deterministic dApp skeleton rendering.
//!
//! The template tree is `templates/dapp/{common,full,headless}`. `common` is always
//! applied; `full` overlays the web client; `headless` is a marker. Generated graph,
//! flow, audit and manifest files come from the design, not from free-form template text.
//! A plan writes nothing; only `apply` writes, and it refuses a non-empty target unless
//! forced.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;

use crate::design::Design;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    Full,
    Headless,
}

impl Template {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "full" => Some(Self::Full),
            "headless" => Some(Self::Headless),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Headless => "headless",
        }
    }

    fn overlay(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Headless => "headless",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScaffoldFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldPlan {
    pub schema: String,
    pub target: String,
    pub template: String,
    pub app_slug: String,
    pub files: Vec<String>,
    pub design: Value,
    pub reuse: Value,
    pub manifest: Value,
}

#[derive(Debug, Error)]
pub enum ScaffoldError {
    #[error("cannot read template {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("template directory is missing: {0}")]
    MissingTemplate(String),
    #[error("template left an unresolved placeholder: {0}")]
    Unresolved(String),
    #[error("target is not empty: {0}; pass --force to merge")]
    TargetNotEmpty(String),
    #[error("target escapes the workspace: {0}")]
    OutsideWorkspace(String),
}

fn read_dir_sorted(path: &Path) -> Result<Vec<PathBuf>, ScaffoldError> {
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

fn collect(
    root: &Path,
    base: &Path,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), ScaffoldError> {
    for entry in read_dir_sorted(root)? {
        if entry.is_dir() {
            collect(&entry, base, files)?;
            continue;
        }
        let relative = entry
            .strip_prefix(base)
            .map_err(|_| ScaffoldError::MissingTemplate(entry.display().to_string()))?;
        let key = relative.to_string_lossy().replace('\\', "/");
        let bytes = std::fs::read(&entry).map_err(|source| ScaffoldError::Io {
            path: entry.display().to_string(),
            source,
        })?;
        files.insert(key, bytes);
    }
    Ok(())
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
        "App".to_string()
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
        "resource".to_string()
    } else {
        trimmed
    }
}

fn replacements(design: &Design) -> BTreeMap<&'static str, String> {
    let slug = design.app.slug.clone();
    let resource = design
        .resources
        .first()
        .map(|resource| resource.id.clone())
        .unwrap_or_else(|| "resource".to_string());
    BTreeMap::from([
        ("__app__", slug.clone()),
        ("__App__", pascal(&slug)),
        ("__APP__", slug.to_uppercase().replace('-', "_")),
        ("__app_ident__", slug.replace('-', "_")),
        ("__app_name__", design.app.name.clone()),
        ("__summary__", design.app.summary.clone()),
        ("__unique_value__", design.unique_value.clone()),
        ("__resource__", snake(&resource)),
        ("__Resource__", pascal(&resource)),
    ])
}

const PLACEHOLDERS: &[&str] = &[
    "__app__",
    "__App__",
    "__APP__",
    "__app_ident__",
    "__app_name__",
    "__summary__",
    "__unique_value__",
    "__resource__",
    "__Resource__",
];

const ALLOWED_DUNDERS: &[&str] = &[
    "__all__",
    "__annotations__",
    "__class__",
    "__dict__",
    "__doc__",
    "__file__",
    "__future__",
    "__init__",
    "__main__",
    "__module__",
    "__name__",
    "__package__",
    "__pycache__",
    "__qualname__",
    "__slots__",
    "__version__",
];

fn unresolved(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("__") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("__") else {
            break;
        };
        let between = &after[..end];
        let is_word = !between.is_empty()
            && between
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_');
        if is_word {
            let token = format!("__{between}__");
            if !ALLOWED_DUNDERS.contains(&token.as_str())
                && !PLACEHOLDERS.contains(&token.as_str())
                && !found.contains(&token)
            {
                found.push(token);
            }
            rest = &after[end + 2..];
        } else {
            rest = after;
        }
    }
    found
}

fn render_text(text: &str, map: &BTreeMap<&'static str, String>) -> Result<String, ScaffoldError> {
    let mut out = text.to_string();
    for (key, value) in map {
        out = out.replace(key, value);
    }
    if let Some(token) = unresolved(&out).into_iter().next() {
        return Err(ScaffoldError::Unresolved(token));
    }
    Ok(out)
}

fn yaml_json(value: &Value) -> Result<Vec<u8>, ScaffoldError> {
    // JSON is valid YAML 1.2 and keeps the generated files machine-readable.
    serde_json::to_vec_pretty(value).map_err(|error| ScaffoldError::Unresolved(error.to_string()))
}

fn generated_files(
    design: &Design,
    template: Template,
    reuse: &Value,
) -> Result<BTreeMap<String, Vec<u8>>, ScaffoldError> {
    let mut files = BTreeMap::new();
    let design_json = serde_json::to_string_pretty(design)
        .map_err(|error| ScaffoldError::Unresolved(error.to_string()))?;
    files.insert("design.json".into(), design_json.into_bytes());
    let reuse_json = serde_json::to_string_pretty(reuse)
        .map_err(|error| ScaffoldError::Unresolved(error.to_string()))?;
    files.insert("reuse-report.json".into(), reuse_json.into_bytes());

    let vertices = json!({
        "schema": "rhobuilder.graph.vertices/v1",
        "vertices": design.graph.vertices,
    });
    let edges = json!({
        "schema": "rhobuilder.graph.edges/v1",
        "edges": design.graph.edges,
    });
    files.insert("spec/graph/vertices.yaml".into(), yaml_json(&vertices)?);
    files.insert("spec/graph/edges.yaml".into(), yaml_json(&edges)?);

    let components: Vec<Value> = design
        .components
        .iter()
        .map(|component| {
            json!({
                "id": component.id,
                "tag": component.tag,
                "primitive": component.primitive,
                "adapter": component.adapter,
                "kind": component.kind,
                "arms": component.arms,
                "lean_status": "owed"
            })
        })
        .collect();
    let flows = json!({
        "schema": "rhobuilder.flows/v1",
        "subsystem": "cornerstone",
        "components": components,
        "note": "lean_status is owed until the per-app model lands; proved-model is the ceiling",
    });
    files.insert("spec/flows/core.yaml".into(), yaml_json(&flows)?);

    let arms = json!({
        "schema": "rhobuilder.audit.arms/v1",
        "arms": design.components.iter().flat_map(|component| {
            component.arms.iter().map(move |arm| json!({
                "component": component.id,
                "arm": arm,
                "status": "scaffold",
            }))
        }).collect::<Vec<_>>(),
    });
    files.insert("audit/registers/arms.yaml".into(), yaml_json(&arms)?);
    files.insert(
        "audit/registers/findings.yaml".into(),
        yaml_json(&json!({"schema": "rhobuilder.audit.findings/v1", "findings": []}))?,
    );

    let manifest = json!({
        "schema": "rhobuilder.app/v1",
        "generator": "rhobuilder",
        "generatorVersion": env!("CARGO_PKG_VERSION"),
        "template": template.name(),
        "app": {"name": design.app.name, "slug": design.app.slug},
        "designSchema": design.schema,
    });
    files.insert(
        ".rhobuilder.json".into(),
        serde_json::to_vec_pretty(&manifest)
            .map_err(|error| ScaffoldError::Unresolved(error.to_string()))?,
    );
    Ok(files)
}

fn render(
    template_root: &Path,
    design: &Design,
    template: Template,
    reuse: &Value,
    target: &Path,
) -> Result<(ScaffoldPlan, BTreeMap<String, Vec<u8>>), ScaffoldError> {
    if !template_root.is_dir() {
        return Err(ScaffoldError::MissingTemplate(
            template_root.display().to_string(),
        ));
    }
    let mut raw: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let common = template_root.join("common");
    if !common.is_dir() {
        return Err(ScaffoldError::MissingTemplate(common.display().to_string()));
    }
    collect(&common, &common, &mut raw)?;
    let overlay = template_root.join(template.overlay());
    if overlay.is_dir() {
        collect(&overlay, &overlay, &mut raw)?;
    }

    let map = replacements(design);
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for (path, bytes) in raw {
        let rendered_path = render_text(&path, &map)?;
        let rendered = match String::from_utf8(bytes.clone()) {
            Ok(text) => render_text(&text, &map)?.into_bytes(),
            Err(_) => bytes,
        };
        files.insert(rendered_path, rendered);
    }
    for (path, bytes) in generated_files(design, template, reuse)? {
        files.insert(path, bytes);
    }

    // The licence is copied once from the rhobuilder checkout, not duplicated in the
    // template tree.
    if let Some(repo_root) = template_root.parent().and_then(Path::parent) {
        let license = repo_root.join("LICENSE");
        if license.is_file() {
            let bytes = std::fs::read(&license).map_err(|source| ScaffoldError::Io {
                path: license.display().to_string(),
                source,
            })?;
            files.insert("LICENSE".into(), bytes);
        }
        crate::formal::vendor_into(repo_root, &mut files)?;
    }

    let file_list = files.keys().cloned().collect::<Vec<_>>();
    let plan = ScaffoldPlan {
        schema: "rhobuilder.scaffold-plan/v1".to_string(),
        target: target.display().to_string(),
        template: template.name().to_string(),
        app_slug: design.app.slug.clone(),
        files: file_list.clone(),
        design: serde_json::to_value(design)
            .map_err(|error| ScaffoldError::Unresolved(error.to_string()))?,
        reuse: reuse.clone(),
        manifest: json!({
            "schema": "rhobuilder.scaffold/v1",
            "target": target.display().to_string(),
            "template": template.name(),
            "app": design.app.slug,
            "files": file_list,
        }),
    };
    Ok((plan, files))
}

pub fn plan(
    template_root: &Path,
    design: &Design,
    template: Template,
    reuse: &Value,
    target: &Path,
) -> Result<ScaffoldPlan, ScaffoldError> {
    render(template_root, design, template, reuse, target).map(|(plan, _)| plan)
}

/// Write the rendered plan. `force` merges into a non-empty target; it never deletes.
pub fn apply(
    template_root: &Path,
    design: &Design,
    template: Template,
    reuse: &Value,
    target: &Path,
    force: bool,
) -> Result<usize, ScaffoldError> {
    let (_, files) = render(template_root, design, template, reuse, target)?;
    if target.exists() {
        let empty = std::fs::read_dir(target)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false);
        if !empty && !force {
            return Err(ScaffoldError::TargetNotEmpty(target.display().to_string()));
        }
    }
    let mut written = 0;
    for (relative, bytes) in &files {
        let path = target.join(relative);
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
        #[cfg(unix)]
        if path.extension().and_then(|ext| ext.to_str()) == Some("sh") {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&path, permissions).map_err(|source| ScaffoldError::Io {
                path: path.display().to_string(),
                source,
            })?;
        }
        written += 1;
    }
    Ok(written)
}

/// Lexically normalize a target and reject `..`; the caller decides whether the result is
/// inside its workspace.
pub fn normalize_target(target: &Path) -> Result<PathBuf, ScaffoldError> {
    let mut out = PathBuf::new();
    for component in target.components() {
        match component {
            Component::ParentDir => {
                return Err(ScaffoldError::OutsideWorkspace(
                    target.display().to_string(),
                ))
            }
            other => out.push(other.as_os_str()),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn design() -> Design {
        serde_json::from_value(json!({
            "schema": "rhobuilder.design/v1",
            "app": {"name": "Demo App", "slug": "demo-app", "summary": "A demo", "networks": ["devnet"]},
            "uniqueValue": "unique",
            "resources": [{"id": "booking", "owner": "app", "stateCell": "bookingState", "lifecycle": []}],
            "components": [{"id": "payments", "tag": "reuse", "primitive": "rcash.account-factory"}],
            "graph": {"vertices": [], "edges": []},
            "acceptance": [{"id": "a1", "given": "g", "when": "w", "then": "t"}]
        }))
        .expect("design")
    }

    fn template() -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("template");
        let common = root.path().join("common");
        std::fs::create_dir_all(common.join("contracts")).expect("dirs");
        std::fs::write(
            common.join("README.md"),
            "# __app_name__\n\nApp: __app__ / __App__ / __resource__\n",
        )
        .expect("readme");
        std::fs::write(
            common.join("contracts/00_root.rho"),
            "// __app__\nnew ret in { ret!(true) }\n",
        )
        .expect("rho");
        std::fs::write(root.path().join("LICENSE"), "AGPL-3.0-only\n").expect("license");
        let full = root.path().join("full");
        std::fs::create_dir_all(full.join("web")).expect("web");
        std::fs::write(full.join("web/index.html"), "<h1>__app_name__</h1>\n").expect("web file");
        root
    }

    #[test]
    fn a_plan_renders_paths_and_content() {
        let root = template();
        let plan = plan(
            root.path(),
            &design(),
            Template::Full,
            &json!({"verdict": "pass"}),
            Path::new("/tmp/demo"),
        )
        .expect("plan");
        assert!(plan.files.iter().any(|path| path == "README.md"));
        assert!(plan.files.iter().any(|path| path == "design.json"));
        assert!(plan
            .files
            .iter()
            .any(|path| path == "spec/graph/vertices.yaml"));
        assert!(plan.files.iter().any(|path| path == "web/index.html"));
        assert!(!plan.files.iter().any(|path| path.contains("__")));
    }

    #[test]
    fn a_headless_plan_omits_web() {
        let root = template();
        let plan = plan(
            root.path(),
            &design(),
            Template::Headless,
            &json!({"verdict": "pass"}),
            Path::new("/tmp/demo"),
        )
        .expect("plan");
        assert!(!plan.files.iter().any(|path| path.starts_with("web/")));
    }

    #[test]
    fn apply_writes_the_tree_and_refuses_a_non_empty_target() {
        let root = template();
        let target = tempfile::tempdir().expect("target");
        let written = apply(
            root.path(),
            &design(),
            Template::Full,
            &json!({"verdict": "pass"}),
            target.path(),
            false,
        )
        .expect("apply");
        assert!(written > 0);
        assert!(target.path().join("README.md").is_file());
        assert!(target.path().join(".rhobuilder.json").is_file());
        assert!(target.path().join("web/index.html").is_file());
        let error = apply(
            root.path(),
            &design(),
            Template::Full,
            &json!({"verdict": "pass"}),
            target.path(),
            false,
        )
        .expect_err("non-empty");
        assert!(matches!(error, ScaffoldError::TargetNotEmpty(_)));
    }

    #[test]
    fn an_unresolved_placeholder_is_refused() {
        let root = template();
        std::fs::write(root.path().join("common/extra.md"), "__unknown__\n").expect("extra");
        let error = plan(
            root.path(),
            &design(),
            Template::Headless,
            &json!({"verdict": "pass"}),
            Path::new("/tmp/demo"),
        )
        .expect_err("placeholder");
        assert!(matches!(error, ScaffoldError::Unresolved(_)));
    }

    #[test]
    fn parent_components_are_rejected() {
        assert!(normalize_target(Path::new("../escape")).is_err());
        assert!(normalize_target(Path::new("/tmp/app")).is_ok());
    }
}
