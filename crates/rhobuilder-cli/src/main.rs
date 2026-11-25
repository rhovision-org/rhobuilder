//! `rhobuilder` — stable JSON CLI over the deterministic core.
//!
//! Exit codes: 0 success, 1 refusal or check failure, 2 usage/config/identity,
//! 3 missing dependency/pin, 4 node unavailable, 5 gate failure, 6 unsafe key.
//! Human output never leaks into a `--json` envelope.

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

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Version => version(cli.json),
        Command::Chain { command } => match command {
            ChainCommand::Probe { network } => chain_probe(cli.json, &cli.registry, &network).await,
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
    fn version_envelope_is_versioned() {
        let envelope = json!({
            "schema": rhobuilder_core::schema("version"),
            "ok": true,
        });
        assert_eq!(envelope["schema"], "rhobuilder.version/v1");
        assert_eq!(envelope["ok"], true);
    }
}
