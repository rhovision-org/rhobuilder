//! `rhobuilder` — stable JSON CLI over the deterministic core.
//!
//! Exit codes: 0 success, 1 refusal/drift, 2 usage/config/identity, 3 missing pin or
//! dependency, 4 node unavailable, 5 gate failure, 6 unsafe key. Human output never leaks
//! into a `--json` envelope.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use serde_json::{json, Value};

#[derive(Debug, Parser)]
#[command(name = "rhobuilder", version, about = "Build rhovision dApps on rholo")]
struct Cli {
    /// Emit one JSON envelope on stdout.
    #[arg(long, global = true)]
    json: bool,

    /// Registry root (default: ./registry).
    #[arg(long, global = true, default_value = "registry")]
    registry: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the product and version.
    Version,
    /// Read-only chain operations.
    Chain {
        #[command(subcommand)]
        command: ChainCommand,
    },
    /// Registry queries and pinned-commit drift.
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ChainCommand {
    /// Probe network identity and capabilities without signing or exploratory deploys.
    Probe {
        /// Network profile name under the registry (`devnet`, `testnet`, `mainnet`).
        #[arg(long, default_value = "testnet")]
        network: String,
    },
}

#[derive(Debug, Subcommand)]
enum RegistryCommand {
    /// List primitives, optionally filtered by network, owner or kind.
    List {
        #[arg(long)]
        network: Option<String>,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        kind: Option<String>,
    },
    /// Show one primitive by id.
    Get {
        #[arg(long)]
        id: String,
    },
    /// Show the reuse graph around a vertex.
    Graph {
        #[arg(long)]
        root: String,
        #[arg(long, default_value_t = 2)]
        depth: usize,
    },
    /// Compare the catalog with sibling repositories at their pinned commits.
    Drift {
        #[arg(long, default_value = ".")]
        source_root: PathBuf,
        /// Treat an unavailable pin or checkout as a failure.
        #[arg(long)]
        strict: bool,
    },
    /// Probe network identity and check the catalog's network claims.
    Verify {
        #[arg(long, default_value = "testnet")]
        network: String,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Version => version(cli.json),
        Command::Chain { command } => match command {
            ChainCommand::Probe { network } => chain_probe(cli.json, &cli.registry, &network).await,
        },
        Command::Registry { command } => match command {
            RegistryCommand::List {
                network,
                owner,
                kind,
            } => registry_list(cli.json, &cli.registry, network, owner, kind),
            RegistryCommand::Get { id } => registry_get(cli.json, &cli.registry, &id),
            RegistryCommand::Graph { root, depth } => {
                registry_graph(cli.json, &cli.registry, &root, depth)
            }
            RegistryCommand::Drift {
                source_root,
                strict,
            } => registry_drift(cli.json, &cli.registry, &source_root, strict),
            RegistryCommand::Verify { network } => {
                registry_verify(cli.json, &cli.registry, &network).await
            }
        },
    }
}

fn version(json: bool) -> ExitCode {
    if json {
        print_json(json!({
            "schema": rhobuilder_core::schema("version"),
            "ok": true,
            "product": rhobuilder_core::PRODUCT,
            "version": rhobuilder_core::VERSION,
        }))
    } else {
        println!("{} {}", rhobuilder_core::PRODUCT, rhobuilder_core::VERSION);
        ExitCode::SUCCESS
    }
}

fn registry_list(
    json: bool,
    registry: &Path,
    network: Option<String>,
    owner: Option<String>,
    kind: Option<String>,
) -> ExitCode {
    let registry = match rhobuilder_registry::Registry::load(registry) {
        Ok(registry) => registry,
        Err(error) => return fail(json, "RegistryError", None, &error.to_string(), 2),
    };
    let items = registry.list(network.as_deref(), owner.as_deref(), kind.as_deref());
    if json {
        return print_json(json!({
            "schema": rhobuilder_core::schema("registry-list"),
            "ok": true,
            "count": items.len(),
            "items": items,
        }));
    }
    println!("registry list: {} item(s)", items.len());
    for item in items {
        println!(
            "  {} ({}, {})",
            item.primitive.id, item.primitive.owner, item.primitive.kind
        );
    }
    ExitCode::SUCCESS
}

fn registry_get(json: bool, registry: &Path, id: &str) -> ExitCode {
    let registry = match rhobuilder_registry::Registry::load(registry) {
        Ok(registry) => registry,
        Err(error) => return fail(json, "RegistryError", None, &error.to_string(), 2),
    };
    let primitive = match registry.get(id) {
        Ok(primitive) => primitive,
        Err(error) => return fail(json, "UnknownPrimitive", Some("id"), &error.to_string(), 2),
    };
    if json {
        return print_json(json!({
            "schema": rhobuilder_core::schema("registry-get"),
            "ok": true,
            "primitive": primitive,
        }));
    }
    println!("{} ({}, {})", primitive.id, primitive.owner, primitive.kind);
    if let Some(component) = &primitive.component {
        println!("  component: {component}");
    }
    println!(
        "  source: {} @ {}",
        primitive.source, primitive.pinned_commit
    );
    println!("  arms: {}", primitive.arms.join(", "));
    for (network, entry) in &primitive.networks {
        println!("  {network}: {} {:?}", entry.verified, entry.uri);
    }
    ExitCode::SUCCESS
}

fn registry_graph(json: bool, registry: &Path, root: &str, depth: usize) -> ExitCode {
    let registry = match rhobuilder_registry::Registry::load(registry) {
        Ok(registry) => registry,
        Err(error) => return fail(json, "RegistryError", None, &error.to_string(), 2),
    };
    let view = match registry.graph(root, depth) {
        Ok(view) => view,
        Err(error) => return fail(json, "UnknownVertex", Some("root"), &error.to_string(), 2),
    };
    if json {
        return print_json(json!({
            "schema": rhobuilder_core::schema("registry-graph"),
            "ok": true,
            "graph": view,
        }));
    }
    println!(
        "graph {root} depth {depth}: {} vertices, {} edges",
        view.vertices.len(),
        view.edges.len()
    );
    for edge in view.edges {
        println!("  {} -{}-> {}", edge.from, edge.verb, edge.to);
    }
    ExitCode::SUCCESS
}

fn registry_drift(json: bool, registry: &Path, source_root: &Path, strict: bool) -> ExitCode {
    let report = match rhobuilder_registry::drift(registry, source_root) {
        Ok(report) => report,
        Err(error) => return fail(json, "RegistryError", None, &error.to_string(), 2),
    };
    if json {
        if let Err(error) = emit_json(&json!({
            "schema": rhobuilder_core::schema("registry-drift"),
            "ok": report.ok,
            "report": &report,
        })) {
            eprintln!("rhobuilder: cannot encode JSON: {error}");
            return ExitCode::from(1);
        }
    } else {
        println!(
            "registry drift: {} checked, {} ok, {} drift, {} pin-unavailable",
            report.summary.checked,
            report.summary.ok,
            report.summary.drift,
            report.summary.pin_unavailable
        );
        for check in &report.checks {
            println!(
                "  {:16} {}/{}: {}",
                check.status,
                check.repo,
                check.id.as_deref().unwrap_or("-"),
                check.detail
            );
        }
    }
    if report.summary.drift > 0 {
        ExitCode::from(1)
    } else if report.summary.pin_unavailable > 0 && strict {
        ExitCode::from(3)
    } else {
        ExitCode::SUCCESS
    }
}

async fn registry_verify(json: bool, registry_path: &Path, network: &str) -> ExitCode {
    let profile_path = registry_path
        .join("networks")
        .join(format!("{network}.json"));
    let profile = match rhobuilder_chain::load_profile(&profile_path) {
        Ok(profile) => profile,
        Err(error) => {
            return fail(
                json,
                "ProfileError",
                None,
                &format!("{}: {error}", profile_path.display()),
                2,
            )
        }
    };
    let observed = match rhobuilder_chain::probe(&profile).await {
        Ok(observed) => observed,
        Err(error) => {
            let (kind, code) = match &error {
                rhobuilder_chain::ProbeError::Gated { .. } => ("NetworkGated", 2),
                rhobuilder_chain::ProbeError::NoEndpoint { .. } => ("NoEndpoint", 2),
                rhobuilder_chain::ProbeError::Http(_) => ("NodeUnavailable", 4),
                rhobuilder_chain::ProbeError::BadResponse { .. } => ("BadResponse", 4),
                rhobuilder_chain::ProbeError::IdentityMismatch { .. } => ("IdentityMismatch", 2),
                rhobuilder_chain::ProbeError::Io(_) | rhobuilder_chain::ProbeError::Json(_) => {
                    ("ProfileError", 2)
                }
            };
            return fail(json, kind, None, &error.to_string(), code);
        }
    };
    let checks = rhobuilder_chain::compare(&profile, &observed);
    let identity_ok = rhobuilder_chain::first_failure(&checks).is_none();

    let registry = match rhobuilder_registry::Registry::load(registry_path) {
        Ok(registry) => registry,
        Err(error) => return fail(json, "RegistryError", None, &error.to_string(), 2),
    };
    let mut claims: BTreeMap<String, usize> = BTreeMap::new();
    let mut errors: Vec<String> = Vec::new();
    for primitive in &registry.primitives {
        let Some(entry) = primitive.networks.get(network) else {
            continue;
        };
        *claims.entry(entry.verified.clone()).or_insert(0) += 1;
        if entry.verified == "lookup" && (entry.root.is_none() || entry.uri.is_none()) {
            errors.push(format!(
                "{} claims verified=lookup without a root and uri",
                primitive.id
            ));
        }
        if entry.verified == "candidate" && entry.root.is_none() {
            errors.push(format!(
                "{} claims verified=candidate without a root",
                primitive.id
            ));
        }
    }
    let ok = identity_ok && errors.is_empty();
    let envelope = json!({
        "schema": rhobuilder_core::schema("registry-verify"),
        "ok": ok,
        "network": network,
        "identity": {"observed": observed, "checks": checks},
        "claims": claims,
        "errors": errors,
        "note": "identity plus manifest claims; a lookup is not a code-identity proof",
    });
    if json {
        if let Err(error) = emit_json(&envelope) {
            eprintln!("rhobuilder: cannot encode JSON: {error}");
            return ExitCode::from(1);
        }
        return if ok {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(2)
        };
    }
    println!(
        "registry verify {network}: {}",
        if ok { "ok" } else { "failed" }
    );
    for (status, count) in claims {
        println!("  {status}: {count}");
    }
    for error in errors {
        println!("  error: {error}");
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

async fn chain_probe(json: bool, registry: &Path, network: &str) -> ExitCode {
    let path = registry.join("networks").join(format!("{network}.json"));
    let profile = match rhobuilder_chain::load_profile(&path) {
        Ok(profile) => profile,
        Err(error) => {
            return fail(
                json,
                "ProfileError",
                None,
                &format!("{}: {error}", path.display()),
                2,
            )
        }
    };
    let observed = match rhobuilder_chain::probe(&profile).await {
        Ok(observed) => observed,
        Err(error) => {
            let (kind, code) = match &error {
                rhobuilder_chain::ProbeError::Gated { .. } => ("NetworkGated", 2),
                rhobuilder_chain::ProbeError::NoEndpoint { .. } => ("NoEndpoint", 2),
                rhobuilder_chain::ProbeError::Http(_) => ("NodeUnavailable", 4),
                rhobuilder_chain::ProbeError::BadResponse { .. } => ("BadResponse", 4),
                rhobuilder_chain::ProbeError::IdentityMismatch { .. } => ("IdentityMismatch", 2),
                rhobuilder_chain::ProbeError::Io(_) | rhobuilder_chain::ProbeError::Json(_) => {
                    ("ProfileError", 2)
                }
            };
            return fail(json, kind, None, &error.to_string(), code);
        }
    };
    let checks = rhobuilder_chain::compare(&profile, &observed);
    if let Some(failure) = rhobuilder_chain::first_failure(&checks) {
        let envelope = json!({
            "schema": rhobuilder_core::schema("chain-probe"),
            "ok": false,
            "kind": "error",
            "network": network,
            "observed": observed,
            "checks": checks,
            "error": {
                "type": "IdentityMismatch",
                "field": failure.name,
                "message": failure.detail,
            },
        });
        if json {
            if let Err(error) = emit_json(&envelope) {
                eprintln!("rhobuilder: cannot encode JSON: {error}");
                return ExitCode::from(1);
            }
        } else {
            eprintln!(
                "chain probe {network}: identity mismatch at {}: {}",
                failure.name, failure.detail
            );
        }
        return ExitCode::from(2);
    }

    if json {
        return print_json(json!({
            "schema": rhobuilder_core::schema("chain-probe"),
            "ok": true,
            "network": network,
            "observed": observed,
            "checks": checks,
        }));
    }
    println!("chain probe: {network} ok");
    if let Some(id) = &observed.network_id {
        println!("  networkId: {id}");
    }
    if let Some(shard) = &observed.shard_id {
        println!("  shardId: {shard}");
    }
    if let Some(genesis) = &observed.genesis_block_hash {
        println!("  genesisBlockHash: {genesis}");
    }
    if let Some(height) = observed.height {
        println!("  height: {height}");
    }
    for check in checks {
        println!("  {:4} {} {}", check.status, check.name, check.detail);
    }
    ExitCode::SUCCESS
}

fn fail(json: bool, kind: &str, field: Option<&str>, message: &str, code: u8) -> ExitCode {
    if json {
        let mut error = json!({"type": kind, "message": message});
        if let Some(field) = field {
            error["field"] = Value::String(field.to_string());
        }
        let envelope = json!({
            "schema": rhobuilder_core::schema("error"),
            "ok": false,
            "kind": "error",
            "error": error,
        });
        if let Err(error) = emit_json(&envelope) {
            eprintln!("rhobuilder: cannot encode JSON: {error}");
            return ExitCode::from(1);
        }
    } else {
        eprintln!("rhobuilder: {message}");
    }
    ExitCode::from(code)
}

fn emit_json(value: &Value) -> Result<(), serde_json::Error> {
    let text = serde_json::to_string_pretty(value)?;
    println!("{text}");
    Ok(())
}

fn print_json(value: Value) -> ExitCode {
    match emit_json(&value) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rhobuilder: cannot encode JSON: {error}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_version_with_json() {
        let cli = Cli::try_parse_from(["rhobuilder", "--json", "version"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn cli_parses_chain_probe() {
        let cli = Cli::try_parse_from([
            "rhobuilder",
            "--json",
            "--registry",
            "registry",
            "chain",
            "probe",
            "--network",
            "testnet",
        ])
        .expect("parse");
        assert!(cli.json);
        assert_eq!(cli.registry, PathBuf::from("registry"));
    }

    #[test]
    fn cli_parses_registry_drift() {
        let cli = Cli::try_parse_from([
            "rhobuilder",
            "--json",
            "registry",
            "drift",
            "--source-root",
            "/home/patrick",
            "--strict",
        ])
        .expect("parse");
        assert!(cli.json);
    }

    #[test]
    fn version_envelope_is_versioned() {
        let envelope = json!({
            "schema": rhobuilder_core::schema("version"),
            "ok": true,
        });
        assert_eq!(envelope["schema"], "rhobuilder.version/v1");
        assert_eq!(envelope["ok"], true);
    }
}
