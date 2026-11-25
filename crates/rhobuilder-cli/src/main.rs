//! `rhobuilder` — stable JSON CLI over the deterministic core.
//!
//! Exit codes: 0 success, 1 refusal/drift, 2 usage/config/identity, 3 missing pin or
//! dependency, 4 node unavailable, 5 gate failure, 6 unsafe key. Human output never leaks
//! into a `--json` envelope.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use rcash_sdk::rcash_chain::keystore::DevnetKeys;
use rcash_sdk::rcash_chain::rho_expr::RhoValue;
use rcash_sdk::Client as ChainClient;
use rhobuilder_chain::executor::{self, Executor, ExecutorConfig, InstallPlan};
use rhobuilder_chain::funding;
use rhobuilder_chain::keystore;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Debug, Parser)]
#[command(name = "rhobuilder", version, about = "Build rhovision dApps on rholo")]
struct Cli {
    /// Emit one JSON envelope on stdout.
    #[arg(long, global = true)]
    json: bool,

    /// Registry root (default: ./registry, or RHOBUILDER_REGISTRY).
    #[arg(
        long,
        global = true,
        env = "RHOBUILDER_REGISTRY",
        default_value = "registry"
    )]
    registry: PathBuf,

    /// Template root (default: ./templates/dapp, or RHOBUILDER_TEMPLATES).
    #[arg(
        long,
        global = true,
        env = "RHOBUILDER_TEMPLATES",
        default_value = "templates/dapp"
    )]
    templates: PathBuf,

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
    /// Design validation and the reuse gate.
    Design {
        #[command(subcommand)]
        command: DesignCommand,
    },
    /// Public-testnet baseline planning (execution stays gated).
    Baseline {
        #[command(subcommand)]
        command: BaselineCommand,
    },
    /// Vendored Lean library operations.
    Formal {
        #[command(subcommand)]
        command: FormalCommand,
    },
    /// Plan or execute a generated app against a target network.
    Deploy {
        #[command(subcommand)]
        command: DeployCommand,
    },
    /// Run the generated project's gates through its Makefile.
    Check {
        #[arg(long)]
        project: PathBuf,
        #[arg(long, default_value = "static")]
        profile: String,
    },
    /// Plan or apply a new dApp skeleton.
    New {
        target: PathBuf,
        #[arg(long)]
        design: PathBuf,
        #[arg(long, default_value = "full")]
        template: String,
        /// Write nothing; print the file plan.
        #[arg(long)]
        plan: bool,
        /// Write the skeleton.
        #[arg(long)]
        apply: bool,
        /// Allow a target outside the working directory or a non-empty target.
        #[arg(long)]
        force: bool,
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
    /// Execute a signed term and return its reply (CLI-only; used by the app suites).
    Call {
        #[arg(long, default_value = "devnet")]
        network: String,
        #[arg(long)]
        keys: PathBuf,
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long)]
        term: Option<String>,
        #[arg(long)]
        execute: bool,
        #[arg(long)]
        allow_live: bool,
        #[arg(long)]
        confirm: Option<String>,
    },
    /// Read one deploy status by id.
    DeployStatus {
        #[arg(long, default_value = "devnet")]
        network: String,
        #[arg(long)]
        deploy_id: String,
    },
    /// Read the raw node status for the network.
    Status {
        #[arg(long, default_value = "devnet")]
        network: String,
    },
}

#[derive(Debug, Subcommand)]
enum DesignCommand {
    /// Validate a design and run the reuse gate.
    Check {
        #[arg(long)]
        path: PathBuf,
    },
    /// Alias for check, used by the DSH tool.
    Validate {
        #[arg(long)]
        path: PathBuf,
    },
    /// Emit the full design and reuse report (same gate, report-oriented output).
    Report {
        #[arg(long)]
        path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum BaselineCommand {
    /// Read-only plan: identity probe plus rendered vendored steps.
    Plan {
        #[arg(long, default_value = "testnet")]
        network: String,
        /// Comma-separated app subset (default: all five).
        #[arg(long)]
        apps: Option<String>,
    },
    /// Generate local ops keys (public report only; secrets are never printed).
    Keygen {
        #[arg(long, default_value = "devnet")]
        network: String,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        force: bool,
    },
    /// Fund the ops key: local validator on devnet, faucet plan/drip on testnet.
    Fund {
        #[arg(long, default_value = "devnet")]
        network: String,
        #[arg(long)]
        keys: PathBuf,
        /// Print the funding plan without moving value.
        #[arg(long)]
        plan: bool,
        /// Move value (devnet transfer; testnet requires live approval).
        #[arg(long)]
        execute: bool,
        #[arg(long)]
        allow_live: bool,
        #[arg(long)]
        confirm: Option<String>,
    },
    /// Read-only identity and catalog-claim verification.
    Verify {
        #[arg(long, default_value = "testnet")]
        network: String,
        /// Signed Root verification of every recorded app root.
        #[arg(long)]
        deep: bool,
        #[arg(long)]
        keys: Option<PathBuf>,
        /// Required for a deep verification on a live network.
        #[arg(long)]
        allow_live: bool,
        #[arg(long)]
        confirm: Option<String>,
    },
    /// Execute (or dry-run) the vendored baseline in dependency order.
    Run {
        #[arg(long, default_value = "testnet")]
        network: String,
        #[arg(long)]
        keys: Option<PathBuf>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        execute: bool,
        #[arg(long)]
        allow_live: bool,
        #[arg(long)]
        confirm: Option<String>,
        #[arg(long)]
        apps: Option<String>,
        #[arg(long)]
        op_cache: Option<PathBuf>,
        #[arg(long, default_value_t = 10)]
        min_lock_blocks: i64,
    },
}

#[derive(Debug, Subcommand)]
enum DeployCommand {
    /// Read-only plan for the generated app's own contracts.
    Plan {
        #[arg(long)]
        project: PathBuf,
        #[arg(long, default_value = "devnet")]
        network: String,
    },
    /// Execute or dry-run the generated app's own contracts.
    Execute {
        #[arg(long)]
        project: PathBuf,
        #[arg(long, default_value = "devnet")]
        network: String,
        #[arg(long)]
        plan: Option<PathBuf>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        execute: bool,
        #[arg(long)]
        keys: Option<PathBuf>,
        #[arg(long)]
        allow_live: bool,
        #[arg(long)]
        confirm: Option<String>,
        #[arg(long)]
        op_cache: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum FormalCommand {
    /// Verify the vendored RhoApp tree hash, flow obligations and Lean escapes.
    Check {
        #[arg(long)]
        project: PathBuf,
        /// Fail on an `owed` obligation.
        #[arg(long)]
        strict: bool,
    },
    /// Refresh the vendored RhoApp from a rhobuilder checkout.
    Sync {
        #[arg(long)]
        project: PathBuf,
        #[arg(long, default_value = ".")]
        source: PathBuf,
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
            ChainCommand::Call {
                network,
                keys,
                file,
                term,
                execute,
                allow_live,
                confirm,
            } => {
                chain_call(
                    cli.json,
                    &cli.registry,
                    &network,
                    &keys,
                    file.as_deref(),
                    term.as_deref(),
                    execute,
                    allow_live,
                    confirm.as_deref(),
                )
                .await
            }
            ChainCommand::DeployStatus { network, deploy_id } => {
                chain_deploy_status(cli.json, &cli.registry, &network, &deploy_id).await
            }
            ChainCommand::Status { network } => {
                chain_status(cli.json, &cli.registry, &network).await
            }
        },
        Command::Design { command } => match command {
            DesignCommand::Check { path }
            | DesignCommand::Validate { path }
            | DesignCommand::Report { path } => design_check(cli.json, &cli.registry, &path),
        },
        Command::Baseline { command } => match command {
            BaselineCommand::Plan { network, apps } => {
                baseline_plan(cli.json, &cli.registry, &network, apps.as_deref()).await
            }
            BaselineCommand::Keygen {
                network,
                out,
                force,
            } => baseline_keygen(cli.json, &network, out.as_deref(), force),
            BaselineCommand::Fund {
                network,
                keys,
                plan,
                execute,
                allow_live,
                confirm,
            } => {
                baseline_fund(
                    cli.json,
                    &cli.registry,
                    &network,
                    &keys,
                    plan,
                    execute,
                    allow_live,
                    confirm.as_deref(),
                )
                .await
            }
            BaselineCommand::Verify {
                network,
                deep,
                keys,
                allow_live,
                confirm,
            } => {
                baseline_verify(
                    cli.json,
                    &cli.registry,
                    &network,
                    deep,
                    keys.as_deref(),
                    allow_live,
                    confirm.as_deref(),
                )
                .await
            }
            BaselineCommand::Run {
                network,
                keys,
                dry_run,
                execute,
                allow_live,
                confirm,
                apps,
                op_cache,
                min_lock_blocks,
            } => {
                baseline_run(
                    cli.json,
                    &cli.registry,
                    &network,
                    keys.as_deref(),
                    dry_run,
                    execute,
                    allow_live,
                    confirm.as_deref(),
                    apps.as_deref(),
                    op_cache.as_deref(),
                    min_lock_blocks,
                )
                .await
            }
        },
        Command::Formal { command } => match command {
            FormalCommand::Check { project, strict } => formal_check(cli.json, &project, strict),
            FormalCommand::Sync { project, source } => formal_sync(cli.json, &project, &source),
        },
        Command::Check { project, profile } => check_project(cli.json, &project, &profile),
        Command::Deploy { command } => match command {
            DeployCommand::Plan { project, network } => {
                deploy_plan(cli.json, &cli.registry, &project, &network).await
            }
            DeployCommand::Execute {
                project,
                network,
                plan,
                dry_run,
                execute,
                keys,
                allow_live,
                confirm,
                op_cache,
            } => {
                deploy_execute(
                    cli.json,
                    &cli.registry,
                    &project,
                    &network,
                    plan.as_deref(),
                    dry_run,
                    execute,
                    keys.as_deref(),
                    allow_live,
                    confirm.as_deref(),
                    op_cache.as_deref(),
                )
                .await
            }
        },
        Command::New {
            target,
            design,
            template,
            plan,
            apply,
            force,
        } => scaffold_new(
            cli.json,
            &cli.registry,
            &cli.templates,
            &target,
            &design,
            &template,
            plan,
            apply,
            force,
        ),
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

const DEFAULT_APPS: [&str; 5] = ["r-cash", "r-gen", "r-share", "r-sure", "r-print"];

fn app_list(filter: Option<&str>) -> Vec<String> {
    match filter {
        Some(text) => text
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect(),
        None => DEFAULT_APPS.iter().map(|item| item.to_string()).collect(),
    }
}

fn install_index(registry: &Path) -> Result<BTreeMap<String, String>, String> {
    let path = registry.join("install").join("index.json");
    let text =
        std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let value: Value =
        serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    let apps = value
        .get("apps")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{} has no apps map", path.display()))?;
    let mut index = BTreeMap::new();
    for (app, commit) in apps {
        if let Some(commit) = commit.as_str() {
            index.insert(app.clone(), commit.to_string());
        }
    }
    Ok(index)
}

fn load_install_plan(registry: &Path, app: &str) -> Result<InstallPlan, String> {
    let index = install_index(registry)?;
    let commit = index
        .get(app)
        .ok_or_else(|| format!("no vendored install plan for {app}"))?;
    let path = registry
        .join("install")
        .join(app)
        .join(commit)
        .join("plan.json");
    let text =
        std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

fn identity_digest(
    profile: &rhobuilder_chain::NetworkProfile,
    observed: &rhobuilder_chain::NetworkObservation,
) -> String {
    let network_id = observed
        .network_id
        .as_deref()
        .or(profile.network_id.as_deref())
        .unwrap_or("");
    let shard = observed
        .shard_id
        .as_deref()
        .unwrap_or(profile.shard_id.as_str());
    let genesis = observed
        .genesis_block_hash
        .as_deref()
        .or(profile.genesis_block_hash.as_deref())
        .unwrap_or("");
    digest_text(&format!("{network_id}|{shard}|{genesis}"))
}

fn confirm_token(
    profile: &rhobuilder_chain::NetworkProfile,
    observed: &rhobuilder_chain::NetworkObservation,
    plan: Option<&str>,
) -> String {
    let network_id = observed
        .network_id
        .as_deref()
        .or(profile.network_id.as_deref())
        .unwrap_or("");
    let genesis = observed
        .genesis_block_hash
        .as_deref()
        .or(profile.genesis_block_hash.as_deref())
        .unwrap_or("");
    let mut token = format!(
        "network={},networkId={},genesis={}",
        profile.network, network_id, genesis
    );
    if let Some(plan) = plan {
        token.push_str(&format!(",plan={plan}"));
    }
    token
}

fn checked_keys_path(path: &Path, network: &str) -> Result<PathBuf, String> {
    if network == "devnet" {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    keystore::require_under(path, &cwd).map_err(|error| error.to_string())
}

fn digest_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    rhobuilder_core::hex(&hasher.finalize())
}

fn digest_json(value: &Value) -> String {
    match serde_json::to_vec(value) {
        Ok(bytes) => digest_text(&String::from_utf8_lossy(&bytes)),
        Err(_) => digest_text("encoding-error"),
    }
}

fn profile_error(json: bool, path: &Path, error: &str) -> ExitCode {
    fail(
        json,
        "ProfileError",
        None,
        &format!("{}: {error}", path.display()),
        2,
    )
}

fn probe_error(json: bool, error: rhobuilder_chain::ProbeError) -> ExitCode {
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
    fail(json, kind, None, &error.to_string(), code)
}

async fn probe_checked(
    json: bool,
    registry_path: &Path,
    network: &str,
) -> Result<
    (
        rhobuilder_chain::NetworkProfile,
        rhobuilder_chain::NetworkObservation,
        Vec<rhobuilder_chain::Check>,
    ),
    ExitCode,
> {
    let profile_path = registry_path
        .join("networks")
        .join(format!("{network}.json"));
    let profile = rhobuilder_chain::load_profile(&profile_path)
        .map_err(|error| profile_error(json, &profile_path, &error.to_string()))?;
    let observed = rhobuilder_chain::probe(&profile)
        .await
        .map_err(|error| probe_error(json, error))?;
    let checks = rhobuilder_chain::compare(&profile, &observed);
    if let Some(failure) = rhobuilder_chain::first_failure(&checks) {
        let envelope = json!({
            "schema": rhobuilder_core::schema("identity"),
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
            }
        } else {
            eprintln!(
                "baseline {network}: identity mismatch at {}: {}",
                failure.name, failure.detail
            );
        }
        return Err(ExitCode::from(2));
    }
    Ok((profile, observed, checks))
}

async fn baseline_plan(
    json: bool,
    registry_path: &Path,
    network: &str,
    apps_filter: Option<&str>,
) -> ExitCode {
    let (profile, observed, checks) = match probe_checked(json, registry_path, network).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    let apps = app_list(apps_filter);
    let mut step_plan = Vec::new();
    for app in &apps {
        let plan = match load_install_plan(registry_path, app) {
            Ok(plan) => plan,
            Err(error) => return fail(json, "InstallPlanError", Some("app"), &error, 2),
        };
        let preview = match executor::preview(&plan, registry_path, &executor::dummy_values()) {
            Ok(preview) => preview,
            Err(error) => return executor_error(json, error),
        };
        step_plan.push(json!({
            "app": app,
            "commit": plan.commit,
            "rootKey": plan.root_key,
            "dependencies": plan.dependencies,
            "steps": preview.steps.len(),
            "preview": preview,
        }));
    }
    let plan_digest = digest_json(&json!(step_plan));
    let token = confirm_token(&profile, &observed, Some(&plan_digest));
    let envelope = json!({
        "schema": rhobuilder_core::schema("baseline-plan"),
        "ok": true,
        "network": network,
        "identity": {"observed": observed, "checks": checks},
        "identityDigest": identity_digest(&profile, &observed),
        "planDigest": plan_digest,
        "confirmation": token,
        "apps": step_plan,
        "execution": if network == "devnet" { "local" } else { "gated" },
        "prerequisites": [
            "for testnet: explicit campaign approval and a funded key",
            "for testnet: --allow-live and the confirmation token with the plan digest",
            "root verification is a signed read; 429 is not_run",
            "vendored pinned terms; no sibling repo is modified",
        ],
        "note": "plan is read-only; execute is a separate gated command",
    });
    if json {
        return print_json(envelope);
    }
    println!(
        "baseline plan {network}: {}",
        if network == "devnet" {
            "local"
        } else {
            "gated"
        }
    );
    for app in &apps {
        println!("  {app}");
    }
    println!("  planDigest: {plan_digest}");
    ExitCode::SUCCESS
}

fn baseline_keygen(json: bool, network: &str, out: Option<&Path>, force: bool) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => return fail(json, "Io", None, &error.to_string(), 2),
    };
    let path = out
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(".ops").join(network).join("keys.json"));
    let path = match keystore::require_under(&path, &cwd) {
        Ok(path) => path,
        Err(error) => return fail(json, "UnsafeKeyPath", Some("out"), &error.to_string(), 2),
    };
    let keys = match keystore::generate(&path, keystore::DEFAULT_ROLES, force) {
        Ok(keys) => keys,
        Err(error) => return fail(json, "Keystore", None, &error.to_string(), 2),
    };
    let report = keystore::public_report(&keys);
    if json {
        return print_json(json!({
            "schema": rhobuilder_core::schema("baseline-keygen"),
            "ok": true,
            "network": network,
            "path": path.display().to_string(),
            "keys": report["keys"],
        }));
    }
    println!(
        "baseline keygen {network}: {} keys at {}",
        report["count"],
        path.display()
    );
    for key in report["keys"].as_array().into_iter().flatten() {
        println!(
            "  {:10} {}",
            key["name"].as_str().unwrap_or("?"),
            key["address"].as_str().unwrap_or("?")
        );
    }
    ExitCode::SUCCESS
}

#[allow(clippy::too_many_arguments)]
async fn baseline_fund(
    json: bool,
    registry_path: &Path,
    network: &str,
    keys_path: &Path,
    plan_only: bool,
    execute: bool,
    allow_live: bool,
    confirm: Option<&str>,
) -> ExitCode {
    if plan_only && execute {
        return fail(
            json,
            "Usage",
            None,
            "--plan and --execute are mutually exclusive",
            2,
        );
    }
    let (profile, observed, _) = match probe_checked(json, registry_path, network).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    let keys_path = match checked_keys_path(keys_path, network) {
        Ok(path) => path,
        Err(error) => return fail(json, "UnsafeKeyPath", Some("keys"), &error, 6),
    };
    if network != "devnet" {
        if let Err(error) = keystore::assert_safe_for(&keys_path, network) {
            return fail(json, "UnsafeKey", Some("keys"), &error.to_string(), 6);
        }
    }
    let keys = match keystore::load(&keys_path) {
        Ok(keys) => keys,
        Err(error) => return fail(json, "Keystore", Some("keys"), &error.to_string(), 6),
    };
    let Some(deployer) = keys.keys.get("deployer") else {
        return fail(
            json,
            "MissingKey",
            Some("keys"),
            "keys file has no deployer",
            6,
        );
    };
    if network == "devnet" {
        let amount = funding::devnet_amount(10);
        let plan = json!({
            "schema": "rhobuilder.fund-plan/v1",
            "network": network,
            "address": deployer.address,
            "amount": amount,
            "source": "in-container validator",
            "execute": execute,
        });
        if !execute {
            if json {
                return print_json(plan);
            }
            println!(
                "baseline fund {network}: {} drops to {} (pass --execute to transfer)",
                amount, deployer.address
            );
            return ExitCode::SUCCESS;
        }
        let node_url = profile.endpoint.clone().unwrap_or_default();
        let container =
            std::env::var("RHOBUILDER_CONTAINER").unwrap_or_else(|_| "devnet-bootstrap".into());
        let key_path = std::env::var("RHOBUILDER_KEY_PATH")
            .unwrap_or_else(|_| "/var/lib/rnode/validator.hex".into());
        let request = funding::DevnetFundRequest {
            node_url: &node_url,
            container: &container,
            shard: &profile.shard_id,
            key_path: &key_path,
            target_public: &deployer.public,
            target_address: &deployer.address,
            amount,
            timeout: std::time::Duration::from_secs(180),
        };
        return match funding::fund_devnet(request).await {
            Ok(report) => {
                if json {
                    print_json(json!({
                        "schema": rhobuilder_core::schema("baseline-fund"),
                        "ok": true,
                        "network": network,
                        "deployId": report.deploy_id,
                        "blockHash": report.block_hash,
                        "status": report.status,
                        "amount": report.amount,
                        "address": report.address,
                    }))
                } else {
                    println!(
                        "baseline fund {network}: {} drops to {} ({})",
                        report.amount, report.address, report.status
                    );
                    ExitCode::SUCCESS
                }
            }
            Err(error) => fail(json, "FundingError", None, &error.to_string(), 4),
        };
    }
    let drip = profile
        .faucet
        .as_ref()
        .and_then(|faucet| faucet.amount)
        .unwrap_or(30_000_000);
    let faucet_plan = funding::faucet_plan(&deployer.address, drip);
    if !execute {
        if json {
            return print_json(faucet_plan);
        }
        println!(
            "baseline fund {network}: one dripped {} drops to {} (--execute requires live approval)",
            drip, deployer.address
        );
        return ExitCode::SUCCESS;
    }
    let expected = confirm_token(&profile, &observed, None);
    if !allow_live {
        return fail(
            json,
            "LiveExecutionGated",
            None,
            "testnet funding needs --allow-live",
            5,
        );
    }
    if confirm != Some(expected.as_str()) {
        return fail(
            json,
            "ConfirmationMismatch",
            Some("confirm"),
            &format!("pass --confirm '{expected}'"),
            5,
        );
    }
    let node_url = profile.endpoint.clone().unwrap_or_default();
    match funding::faucet_drip(&node_url, &deployer.address).await {
        Ok(value) => print_json(json!({
            "schema": rhobuilder_core::schema("baseline-fund"),
            "ok": true,
            "network": network,
            "faucet": value,
        })),
        Err(error) => fail(json, "FundingError", None, &error.to_string(), 4),
    }
}

async fn seed_testnet_candidates(
    registry_path: &Path,
    profile: &rhobuilder_chain::NetworkProfile,
    keys: &DevnetKeys,
    plan: &InstallPlan,
    state_path: &Path,
    op_cache: &Path,
) -> Result<Option<String>, String> {
    if state_path.is_file() {
        return Ok(None);
    }
    let profile_path = registry_path.join("networks").join("testnet.json");
    let value: Value = std::fs::read_to_string(&profile_path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .ok_or_else(|| format!("{} is not JSON", profile_path.display()))?;
    let roots = value
        .get("roots")
        .and_then(|roots| roots.get("r-cash"))
        .cloned()
        .unwrap_or(Value::Null);
    let mut candidates = Vec::new();
    for key in ["committedManifest", "uncommittedCandidate", "root"] {
        if let Some(root) = roots.get(key).and_then(Value::as_str) {
            if !candidates.iter().any(|item| item == root) {
                candidates.push(root.to_string());
            }
        }
    }
    if candidates.is_empty() {
        return Ok(None);
    }
    let key = keys.key("deployer").map_err(|error| error.to_string())?;
    let client = ChainClient::new(profile.endpoint.clone().unwrap_or_default(), key)
        .map_err(|error| error.to_string())?
        .with_shard(profile.shard_id.clone())
        .with_phlo(2_000_000, 1);
    let mut partial: Option<String> = None;
    for root in candidates {
        let found = rhobuilder_chain::root::verify_keys(
            &client,
            &root,
            &plan.required_keys,
            plan.resolve_arity,
            &op_cache.join("verify"),
        )
        .await
        .map_err(|error| error.to_string())?;
        if plan.required_keys.iter().all(|key| found.contains_key(key)) {
            let state = json!({
                "schema": "rhobuilder.install-state/v1",
                "network": "testnet",
                "app": "r-cash",
                "commit": plan.commit,
                "rootUri": root,
                "captured": {},
                "steps": {},
                "status": "verified",
            });
            if let Some(parent) = state_path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            std::fs::write(
                state_path,
                serde_json::to_string_pretty(&state).map_err(|error| error.to_string())? + "\n",
            )
            .map_err(|error| error.to_string())?;
            return Ok(Some(root));
        }
        if !found.is_empty() && partial.is_none() {
            partial = Some(root);
        }
    }
    if let Some(root) = partial {
        return Err(format!(
            "candidate {root} is partial; required r-cash keys are missing. \
             Pass an explicit partial-root override or approve a full r-cash deploy."
        ));
    }
    Ok(None)
}

#[allow(clippy::too_many_arguments)]
async fn baseline_run(
    json: bool,
    registry_path: &Path,
    network: &str,
    keys_path: Option<&Path>,
    dry_run: bool,
    execute: bool,
    allow_live: bool,
    confirm: Option<&str>,
    apps_filter: Option<&str>,
    op_cache: Option<&Path>,
    min_lock_blocks: i64,
) -> ExitCode {
    if dry_run == execute {
        return fail(
            json,
            "Usage",
            None,
            "baseline run needs exactly one of --dry-run or --execute",
            2,
        );
    }
    let (profile, observed, checks) = match probe_checked(json, registry_path, network).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    let apps = app_list(apps_filter);
    let mut previews = Vec::new();
    for app in &apps {
        let plan = match load_install_plan(registry_path, app) {
            Ok(plan) => plan,
            Err(error) => return fail(json, "InstallPlanError", Some("app"), &error, 2),
        };
        let preview = match executor::preview(&plan, registry_path, &executor::dummy_values()) {
            Ok(preview) => preview,
            Err(error) => return executor_error(json, error),
        };
        previews.push(json!({"app": app, "preview": preview}));
    }
    let plan_digest = digest_json(&json!(previews));
    if dry_run {
        let envelope = json!({
            "schema": rhobuilder_core::schema("baseline-execute"),
            "ok": true,
            "network": network,
            "status": "planned",
            "dryRun": true,
            "identity": {"observed": observed, "checks": checks},
            "identityDigest": identity_digest(&profile, &observed),
            "planDigest": plan_digest,
            "confirmation": confirm_token(&profile, &observed, Some(&plan_digest)),
            "apps": previews,
            "note": "dry-run: no key loaded, no signed bytes, no POST",
        });
        if json {
            return print_json(envelope);
        }
        println!("baseline run {network}: planned (no writes)");
        println!("  planDigest: {plan_digest}");
        return ExitCode::SUCCESS;
    }
    let Some(keys_path) = keys_path else {
        return fail(
            json,
            "MissingKey",
            Some("keys"),
            "baseline run needs --keys",
            6,
        );
    };
    let keys_path = match checked_keys_path(keys_path, network) {
        Ok(path) => path,
        Err(error) => return fail(json, "UnsafeKeyPath", Some("keys"), &error, 6),
    };
    if let Err(error) = keystore::assert_safe_for(&keys_path, network) {
        return fail(json, "UnsafeKey", Some("keys"), &error.to_string(), 6);
    }
    if network != "devnet" {
        if !allow_live {
            return fail(
                json,
                "LiveExecutionGated",
                None,
                "testnet baseline needs --allow-live",
                5,
            );
        }
        let expected = confirm_token(&profile, &observed, Some(&plan_digest));
        if confirm != Some(expected.as_str()) {
            return fail(
                json,
                "ConfirmationMismatch",
                Some("confirm"),
                &format!("pass --confirm '{expected}'"),
                5,
            );
        }
    }
    let keys = match keystore::load(&keys_path) {
        Ok(keys) => keys,
        Err(error) => return fail(json, "Keystore", Some("keys"), &error.to_string(), 6),
    };
    let node_url = profile.endpoint.clone().unwrap_or_default();
    let op_cache = op_cache
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(".ops").join("ops").join(network));
    if network == "testnet" {
        let rplan = match load_install_plan(registry_path, "r-cash") {
            Ok(plan) => plan,
            Err(error) => return fail(json, "InstallPlanError", Some("app"), &error, 2),
        };
        let state_path = PathBuf::from(".ops")
            .join("state")
            .join("testnet-r-cash.json");
        match seed_testnet_candidates(
            registry_path,
            &profile,
            &keys,
            &rplan,
            &state_path,
            &op_cache,
        )
        .await
        {
            Ok(Some(root)) => {
                if !json {
                    println!("baseline run testnet: adopting r-cash candidate {root}");
                }
            }
            Ok(None) => {}
            Err(error) => return fail(json, "PartialRoot", None, &error, 1),
        }
    }
    let mut roots: BTreeMap<String, String> = BTreeMap::new();
    let mut reports = Vec::new();
    for app in &apps {
        let plan = match load_install_plan(registry_path, app) {
            Ok(plan) => plan,
            Err(error) => return fail(json, "InstallPlanError", Some("app"), &error, 2),
        };
        let state_path = PathBuf::from(".ops")
            .join("state")
            .join(format!("{network}-{app}.json"));
        let config = ExecutorConfig {
            network,
            node_url: &node_url,
            shard: &profile.shard_id,
            registry_root: registry_path,
            terms_dir: None,
            key_name: "deployer",
            admin_names: ["admin1", "admin2", "admin3"],
            issuer_name: "issuer",
            reporter_names: ["reporter1", "reporter2", "reporter3"],
            roots: &roots,
            state_path: &state_path,
            op_cache: &op_cache,
            min_lock_blocks,
            dry_run: false,
        };
        let executor = match Executor::new(&keys, config, &plan.phlo) {
            Ok(executor) => executor,
            Err(error) => return executor_error(json, error),
        };
        let report = match executor.execute(&plan).await {
            Ok(report) => report,
            Err(error) => return executor_error(json, error),
        };
        if let Some(root) = &report.root_uri {
            roots.insert(app.clone(), root.clone());
        }
        reports.push(json!({"app": app, "report": report}));
        if !json {
            println!(
                "baseline run {network}: {app} {}",
                reports.last().map(|_| "").unwrap_or("")
            );
        }
    }
    let manifest = json!({
        "schema": "rhobuilder.baseline-manifest/v1",
        "network": network,
        "networkId": observed.network_id.clone(),
        "genesisBlockHash": observed.genesis_block_hash,
        "identityDigest": identity_digest(&profile, &observed),
        "planDigest": plan_digest,
        "apps": reports,
    });
    let mut install_apps = serde_json::Map::new();
    for item in manifest["apps"].as_array().into_iter().flatten() {
        let Some(app) = item.get("app").and_then(Value::as_str) else {
            continue;
        };
        let report = item.get("report").cloned().unwrap_or(Value::Null);
        install_apps.insert(
            app.to_string(),
            json!({
                "commit": report.get("commit"),
                "root": report.get("rootUri"),
                "status": report.get("status"),
                "verified": report.get("verified"),
                "steps": report.get("steps"),
            }),
        );
    }
    let install_manifest = json!({
        "schema": "rhobuilder.install-manifest/v1",
        "network": network,
        "networkId": observed.network_id,
        "shardId": observed.shard_id,
        "apps": install_apps,
    });
    let install_path = registry_path
        .join("install")
        .join(format!("{network}.json"));
    if let Some(parent) = install_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(error) = std::fs::write(
        &install_path,
        serde_json::to_string_pretty(&install_manifest).unwrap_or_default() + "\n",
    ) {
        return fail(json, "Io", None, &error.to_string(), 2);
    }
    let out = PathBuf::from("deployments").join(format!("{network}-baseline.json"));
    if let Some(parent) = out.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            return fail(json, "Io", None, &error.to_string(), 2);
        }
    }
    if let Err(error) = std::fs::write(
        &out,
        serde_json::to_string_pretty(&manifest).unwrap_or_default() + "\n",
    ) {
        return fail(json, "Io", None, &error.to_string(), 2);
    }
    if json {
        return print_json(json!({
            "schema": rhobuilder_core::schema("baseline-execute"),
            "ok": true,
            "network": network,
            "status": "verified",
            "manifest": out.display().to_string(),
            "result": manifest,
        }));
    }
    println!("baseline run {network}: verified; wrote {}", out.display());
    ExitCode::SUCCESS
}

#[allow(clippy::too_many_arguments)]
async fn baseline_verify(
    json: bool,
    registry_path: &Path,
    network: &str,
    deep: bool,
    keys_path: Option<&Path>,
    allow_live: bool,
    confirm: Option<&str>,
) -> ExitCode {
    let (profile, observed, checks) = match probe_checked(json, registry_path, network).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    if deep && network != "devnet" {
        if !allow_live {
            return fail(
                json,
                "LiveExecutionGated",
                None,
                "testnet deep verification signs Root reads and needs --allow-live",
                5,
            );
        }
        let expected = confirm_token(&profile, &observed, None);
        if confirm != Some(expected.as_str()) {
            return fail(
                json,
                "ConfirmationMismatch",
                Some("confirm"),
                &format!("pass --confirm '{expected}'"),
                5,
            );
        }
    }
    let apps = app_list(None);
    let client = if deep {
        let Some(keys_path) = keys_path else {
            return fail(json, "MissingKey", Some("keys"), "--deep needs --keys", 6);
        };
        let keys_path = match checked_keys_path(keys_path, network) {
            Ok(path) => path,
            Err(error) => return fail(json, "UnsafeKeyPath", Some("keys"), &error, 6),
        };
        if let Err(error) = keystore::assert_safe_for(&keys_path, network) {
            return fail(json, "UnsafeKey", Some("keys"), &error.to_string(), 6);
        }
        let keys = match keystore::load(&keys_path) {
            Ok(keys) => keys,
            Err(error) => return fail(json, "Keystore", Some("keys"), &error.to_string(), 6),
        };
        let key = match keys.key("deployer") {
            Ok(key) => key,
            Err(error) => return fail(json, "MissingKey", Some("keys"), &error.to_string(), 6),
        };
        match ChainClient::new(profile.endpoint.clone().unwrap_or_default(), key) {
            Ok(client) => Some(
                client
                    .with_shard(profile.shard_id.clone())
                    .with_phlo(1_000_000, 1),
            ),
            Err(error) => return fail(json, "ChainError", None, &error.to_string(), 4),
        }
    } else {
        None
    };
    let op_cache = PathBuf::from(".ops")
        .join("ops")
        .join(network)
        .join("verify");
    let mut results = Vec::new();
    let mut ok = true;
    for app in &apps {
        let plan = match load_install_plan(registry_path, app) {
            Ok(plan) => plan,
            Err(error) => return fail(json, "InstallPlanError", Some("app"), &error, 2),
        };
        let state_path = PathBuf::from(".ops")
            .join("state")
            .join(format!("{network}-{app}.json"));
        let state: Value = match std::fs::read_to_string(&state_path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
        {
            Some(value) => value,
            None => {
                ok = false;
                results.push(json!({"app": app, "status": "missing", "state": state_path}));
                continue;
            }
        };
        let root = state.get("rootUri").and_then(Value::as_str);
        let Some(root) = root else {
            ok = false;
            results.push(json!({"app": app, "status": "missing-root", "state": state_path}));
            continue;
        };
        if let Some(client) = &client {
            match rhobuilder_chain::root::verify_keys(
                client,
                root,
                &plan.required_keys,
                plan.resolve_arity,
                &op_cache,
            )
            .await
            {
                Ok(found) => {
                    let missing: Vec<String> = plan
                        .required_keys
                        .iter()
                        .filter(|key| !found.contains_key(*key))
                        .cloned()
                        .collect();
                    if !missing.is_empty() {
                        ok = false;
                    }
                    results.push(json!({
                        "app": app,
                        "status": if missing.is_empty() { "verified" } else { "missing-keys" },
                        "root": root,
                        "missing": missing,
                    }));
                }
                Err(error) => {
                    ok = false;
                    results.push(json!({
                        "app": app,
                        "status": "not_run",
                        "root": root,
                        "error": error.to_string(),
                    }));
                }
            }
        } else {
            results.push(json!({"app": app, "status": "state-present", "root": root}));
        }
    }
    let envelope = json!({
        "schema": rhobuilder_core::schema("baseline-verify"),
        "ok": ok,
        "network": network,
        "identity": {"observed": observed, "checks": checks},
        "deep": deep,
        "results": results,
        "note": "a stored manifest is evidence, not authority; deep verification is a signed Root read",
    });
    if json {
        if let Err(error) = emit_json(&envelope) {
            eprintln!("rhobuilder: cannot encode JSON: {error}");
            return ExitCode::from(1);
        }
    } else {
        println!(
            "baseline verify {network}: {} ({} app(s), deep={deep})",
            if ok { "ok" } else { "failed" },
            results.len()
        );
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn app_slug(project: &Path) -> Result<String, String> {
    let path = project.join(".rhobuilder.json");
    let text =
        std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let value: Value =
        serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    value
        .get("app")
        .and_then(|app| app.get("slug"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("{} has no app.slug", path.display()))
}

fn app_plan(project: &Path) -> Result<(InstallPlan, PathBuf), String> {
    let slug = app_slug(project)?;
    let contracts = project.join("contracts").join("src");
    let manifest_path = project.join(".rhobuilder-contracts.json");
    if manifest_path.is_file() {
        let text = std::fs::read_to_string(&manifest_path)
            .map_err(|error| format!("{}: {error}", manifest_path.display()))?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|error| format!("{}: {error}", manifest_path.display()))?;
        let array = |key: &str| -> Vec<String> {
            value
                .get(key)
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default()
        };
        let mut steps = Vec::new();
        for item in value
            .get("steps")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let name = item
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| "contract manifest step has no name".to_string())?;
            let term = item
                .get("term")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("contract manifest step {name} has no term"))?;
            let mut captures = Vec::new();
            for capture in item
                .get("captures")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let placeholder = capture
                    .get("placeholder")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "capture has no placeholder".to_string())?;
                let from = capture
                    .get("from")
                    .and_then(Value::as_str)
                    .unwrap_or("uris");
                captures.push(executor::Capture {
                    placeholder: placeholder.to_string(),
                    from: from.to_string(),
                    key: capture
                        .get("key")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    index: capture
                        .get("index")
                        .and_then(Value::as_u64)
                        .map(|index| index as usize),
                });
            }
            steps.push(executor::PlanStep {
                name: name.to_string(),
                term: term.to_string(),
                root: item.get("root").and_then(Value::as_bool).unwrap_or(false),
                placeholders: Vec::new(),
                captures,
            });
        }
        let plan = InstallPlan {
            schema: "rhobuilder.install-plan/v1".to_string(),
            app: slug,
            repo: String::new(),
            commit: "local".to_string(),
            source_root: "contracts/src".to_string(),
            root_key: value
                .get("rootKey")
                .and_then(Value::as_str)
                .unwrap_or("component:root")
                .to_string(),
            resolve_arity: 3,
            phlo: executor::PlanPhlo {
                limit: 4_000_000,
                price: 1,
            },
            dependencies: array("dependencies"),
            required_keys: array("requiredKeys"),
            placeholders: Vec::new(),
            steps,
        };
        return Ok((plan, contracts));
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(&contracts)
        .map_err(|error| format!("{}: {error}", contracts.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rho"))
        .collect();
    files.sort();
    if files.is_empty() {
        return Err(format!("{} has no Rholang terms", contracts.display()));
    }
    if let Some(index) = files
        .iter()
        .position(|path| path.file_name().and_then(|name| name.to_str()) == Some("00_root.rho"))
    {
        let root = files.remove(index);
        files.push(root);
    }
    let mut steps = Vec::new();
    for path in &files {
        let term = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("bad term path {}", path.display()))?
            .to_string();
        let stem = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("term");
        let mut captures = Vec::new();
        let placeholder = match stem {
            "10_dispute" => Some("__DISPUTE_URI__"),
            "40_payments" => Some("__PAYMENTS_URI__"),
            "booking-adapter" => Some("__BOOKING_ADAPTER_URI__"),
            "cover-adapter" => Some("__COVER_ADAPTER_URI__"),
            _ => None,
        };
        if let Some(placeholder) = placeholder {
            captures.push(executor::Capture {
                placeholder: placeholder.to_string(),
                from: "uris".to_string(),
                key: None,
                index: Some(0),
            });
        }
        steps.push(executor::PlanStep {
            name: stem.to_string(),
            term,
            root: stem == "00_root",
            placeholders: Vec::new(),
            captures,
        });
    }
    let plan = InstallPlan {
        schema: "rhobuilder.install-plan/v1".to_string(),
        app: slug,
        repo: String::new(),
        commit: "local".to_string(),
        source_root: "contracts/src".to_string(),
        root_key: "component:root".to_string(),
        resolve_arity: 3,
        phlo: executor::PlanPhlo {
            limit: 4_000_000,
            price: 1,
        },
        dependencies: vec!["r-cash".into(), "r-share".into(), "r-sure".into()],
        required_keys: vec![
            "component:root".into(),
            "component:dispute-factory".into(),
            "component:payments".into(),
            "component:booking-adapter".into(),
            "component:cover-adapter".into(),
        ],
        placeholders: Vec::new(),
        steps,
    };
    Ok((plan, contracts))
}

fn network_roots(registry: &Path, network: &str) -> BTreeMap<String, String> {
    let mut roots = BTreeMap::new();
    let install = registry.join("install").join(format!("{network}.json"));
    if let Some(value) = std::fs::read_to_string(&install)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
    {
        if let Some(apps) = value.get("apps").and_then(Value::as_object) {
            for (app, entry) in apps {
                if let Some(root) = entry.get("root").and_then(Value::as_str) {
                    roots.insert(app.clone(), root.to_string());
                }
            }
        }
    }
    if roots.is_empty() {
        let profile = registry.join("networks").join(format!("{network}.json"));
        if let Some(value) = std::fs::read_to_string(&profile)
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        {
            if let Some(apps) = value.get("roots").and_then(Value::as_object) {
                for (app, entry) in apps {
                    if let Some(root) = entry
                        .get("root")
                        .or_else(|| entry.get("committedManifest"))
                        .and_then(Value::as_str)
                    {
                        roots.insert(app.clone(), root.to_string());
                    }
                }
            }
        }
    }
    roots
}

fn app_values(roots: &BTreeMap<String, String>, dummy: bool) -> BTreeMap<String, String> {
    let mut values = if dummy {
        executor::dummy_values()
    } else {
        BTreeMap::new()
    };
    for (app, root) in roots {
        let key = match app.as_str() {
            "r-cash" => "__RCASH_ROOT_URI__",
            "r-gen" => "__RGEN_ROOT_URI__",
            "r-share" => "__RSHARE_ROOT_URI__",
            "r-sure" => "__RSURE_ROOT_URI__",
            "r-print" => "__RPRINT_ROOT_URI__",
            _ => continue,
        };
        values.insert(key.to_string(), root.clone());
    }
    values
}

async fn deploy_plan(json: bool, registry_path: &Path, project: &Path, network: &str) -> ExitCode {
    let (profile, observed, checks) = match probe_checked(json, registry_path, network).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    let (plan, contracts) = match app_plan(project) {
        Ok(value) => value,
        Err(error) => return fail(json, "AppPlanError", None, &error, 2),
    };
    let roots = network_roots(registry_path, network);
    let values = app_values(&roots, true);
    let preview =
        match executor::preview_with_terms(&plan, registry_path, Some(&contracts), &values) {
            Ok(preview) => preview,
            Err(error) => return executor_error(json, error),
        };
    let plan_digest = digest_json(&json!(preview));
    let missing: Vec<String> = plan
        .dependencies
        .iter()
        .filter(|dependency| !roots.contains_key(*dependency))
        .cloned()
        .collect();
    let envelope = json!({
        "schema": rhobuilder_core::schema("deploy-plan"),
        "ok": true,
        "network": network,
        "app": plan.app,
        "identity": {"observed": observed, "checks": checks},
        "identityDigest": identity_digest(&profile, &observed),
        "planDigest": plan_digest,
        "confirmation": confirm_token(&profile, &observed, Some(&plan_digest)),
        "roots": roots,
        "missingDependencies": missing,
        "preview": preview,
        "execution": if network == "devnet" { "local" } else { "gated" },
    });
    if json {
        return print_json(envelope);
    }
    println!(
        "deploy plan {network}: {} step(s) for {}",
        plan.steps.len(),
        plan.app
    );
    println!("  planDigest: {plan_digest}");
    ExitCode::SUCCESS
}

#[allow(clippy::too_many_arguments)]
async fn deploy_execute(
    json: bool,
    registry_path: &Path,
    project: &Path,
    network: &str,
    expected_plan: Option<&Path>,
    dry_run: bool,
    execute: bool,
    keys_path: Option<&Path>,
    allow_live: bool,
    confirm: Option<&str>,
    op_cache: Option<&Path>,
) -> ExitCode {
    if dry_run == execute {
        return fail(
            json,
            "Usage",
            None,
            "deploy execute needs exactly one of --dry-run or --execute",
            2,
        );
    }
    let (profile, observed, checks) = match probe_checked(json, registry_path, network).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    let (plan, contracts) = match app_plan(project) {
        Ok(value) => value,
        Err(error) => return fail(json, "AppPlanError", None, &error, 2),
    };
    let roots = network_roots(registry_path, network);
    let values = app_values(&roots, true);
    let preview =
        match executor::preview_with_terms(&plan, registry_path, Some(&contracts), &values) {
            Ok(preview) => preview,
            Err(error) => return executor_error(json, error),
        };
    let plan_digest = digest_json(&json!(preview));
    if let Some(path) = expected_plan {
        let expected: Value = match std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
        {
            Some(value) => value,
            None => return fail(json, "PlanError", Some("plan"), "plan file is not JSON", 2),
        };
        let recorded = expected
            .get("planDigest")
            .and_then(Value::as_str)
            .unwrap_or("");
        if recorded != plan_digest {
            return fail(
                json,
                "PlanDigestMismatch",
                Some("plan"),
                &format!("plan digest {recorded} does not match {plan_digest}"),
                5,
            );
        }
    }
    if dry_run {
        let envelope = json!({
            "schema": rhobuilder_core::schema("deploy-execute"),
            "ok": true,
            "network": network,
            "app": plan.app,
            "status": "planned",
            "dryRun": true,
            "identity": {"observed": observed, "checks": checks},
            "identityDigest": identity_digest(&profile, &observed),
            "planDigest": plan_digest,
            "preview": preview,
        });
        if json {
            return print_json(envelope);
        }
        println!("deploy execute {network}: planned (no writes)");
        return ExitCode::SUCCESS;
    }
    let Some(keys_path) = keys_path else {
        return fail(
            json,
            "MissingKey",
            Some("keys"),
            "deploy execute needs --keys",
            6,
        );
    };
    let keys_path = match checked_keys_path(keys_path, network) {
        Ok(path) => path,
        Err(error) => return fail(json, "UnsafeKeyPath", Some("keys"), &error, 6),
    };
    if let Err(error) = keystore::assert_safe_for(&keys_path, network) {
        return fail(json, "UnsafeKey", Some("keys"), &error.to_string(), 6);
    }
    if network != "devnet" {
        if !allow_live {
            return fail(
                json,
                "LiveExecutionGated",
                None,
                "testnet deploy needs --allow-live",
                5,
            );
        }
        let expected = confirm_token(&profile, &observed, Some(&plan_digest));
        if confirm != Some(expected.as_str()) {
            return fail(
                json,
                "ConfirmationMismatch",
                Some("confirm"),
                &format!("pass --confirm '{expected}'"),
                5,
            );
        }
    }
    if roots.is_empty() {
        return fail(
            json,
            "MissingDependency",
            Some("roots"),
            "no primitive roots are recorded for this network",
            3,
        );
    }
    let missing: Vec<String> = plan
        .dependencies
        .iter()
        .filter(|dependency| !roots.contains_key(*dependency))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return fail(
            json,
            "MissingDependency",
            Some("roots"),
            &format!("missing primitive roots {missing:?}"),
            3,
        );
    }
    let keys = match keystore::load(&keys_path) {
        Ok(keys) => keys,
        Err(error) => return fail(json, "Keystore", Some("keys"), &error.to_string(), 6),
    };
    let node_url = profile.endpoint.clone().unwrap_or_default();
    let state_path = PathBuf::from(".ops")
        .join("state")
        .join(format!("{network}-{}.json", plan.app));
    let op_cache = op_cache.map(Path::to_path_buf).unwrap_or_else(|| {
        PathBuf::from(".ops")
            .join("ops")
            .join(format!("{network}-{}", plan.app))
    });
    let config = ExecutorConfig {
        network,
        node_url: &node_url,
        shard: &profile.shard_id,
        registry_root: registry_path,
        terms_dir: Some(&contracts),
        key_name: "deployer",
        admin_names: ["admin1", "admin2", "admin3"],
        issuer_name: "issuer",
        reporter_names: ["reporter1", "reporter2", "reporter3"],
        roots: &roots,
        state_path: &state_path,
        op_cache: &op_cache,
        min_lock_blocks: 10,
        dry_run: false,
    };
    let executor = match Executor::new(&keys, config, &plan.phlo) {
        Ok(executor) => executor,
        Err(error) => return executor_error(json, error),
    };
    let report = match executor.execute(&plan).await {
        Ok(report) => report,
        Err(error) => return executor_error(json, error),
    };
    let manifest = json!({
        "schema": "rhobuilder.app-deploy/v1",
        "network": network,
        "app": plan.app,
        "identityDigest": identity_digest(&profile, &observed),
        "planDigest": plan_digest,
        "report": report,
    });
    let out = PathBuf::from("deployments").join(format!("{network}-{}.json", plan.app));
    if let Some(parent) = out.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            return fail(json, "Io", None, &error.to_string(), 2);
        }
    }
    let text = serde_json::to_string_pretty(&manifest).unwrap_or_default() + "\n";
    if let Err(error) = std::fs::write(&out, &text) {
        return fail(json, "Io", None, &error.to_string(), 2);
    }
    let project_out = project
        .join("deployments")
        .join(format!("{network}-{}.json", plan.app));
    if let Some(parent) = project_out.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(error) = std::fs::write(&project_out, &text) {
        return fail(json, "Io", None, &error.to_string(), 2);
    }
    if json {
        return print_json(json!({
            "schema": rhobuilder_core::schema("deploy-execute"),
            "ok": true,
            "network": network,
            "app": plan.app,
            "status": report.status,
            "manifest": out.display().to_string(),
            "result": manifest,
        }));
    }
    println!(
        "deploy execute {network}: {} {}; wrote {}",
        plan.app,
        report.status,
        out.display()
    );
    ExitCode::SUCCESS
}

fn executor_error(json: bool, error: executor::ExecutorError) -> ExitCode {
    let (kind, code) = match &error {
        executor::ExecutorError::MissingKey(_) => ("UnsafeKey", 6),
        executor::ExecutorError::Gated(_) => ("LiveExecutionGated", 5),
        executor::ExecutorError::Unresolved { .. } => ("UnresolvedPlaceholder", 2),
        executor::ExecutorError::TermTooLarge { .. } => ("GateFailure", 5),
        executor::ExecutorError::Io { .. }
        | executor::ExecutorError::Json { .. }
        | executor::ExecutorError::State(_)
        | executor::ExecutorError::MissingPlan(_) => ("InstallError", 2),
        executor::ExecutorError::Chain(rcash_sdk::rcash_chain::ChainError::RateLimited {
            ..
        })
        | executor::ExecutorError::Chain(rcash_sdk::rcash_chain::ChainError::Unavailable {
            ..
        }) => ("NodeUnavailable", 4),
        executor::ExecutorError::Chain(_) => ("ChainError", 4),
        executor::ExecutorError::DeployFailed { .. }
        | executor::ExecutorError::NotFinalized { .. }
        | executor::ExecutorError::PartialRoot { .. }
        | executor::ExecutorError::Verification { .. } => ("InstallFailed", 1),
    };
    fail(json, kind, None, &error.to_string(), code)
}

fn formal_check(json: bool, project: &Path, strict: bool) -> ExitCode {
    let vendor = match rhobuilder_core::formal::check(project) {
        Ok(report) => report,
        Err(error) => return fail(json, "FormalError", None, &error.to_string(), 2),
    };
    let obligations = match rhobuilder_core::formal::obligations(project, strict) {
        Ok(report) => report,
        Err(error) => return fail(json, "FormalError", None, &error.to_string(), 2),
    };
    let vendor_ok = vendor.get("ok").and_then(Value::as_bool).unwrap_or(false);
    let obligations_ok = obligations
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let ok = vendor_ok && obligations_ok;
    if json {
        if let Err(error) = emit_json(&json!({
            "schema": rhobuilder_core::schema("formal-check"),
            "ok": ok,
            "vendor": &vendor,
            "obligations": &obligations,
        })) {
            eprintln!("rhobuilder: cannot encode JSON: {error}");
            return ExitCode::from(1);
        }
    } else {
        println!(
            "formal check: {} (vendor {}, {} lean file(s), {} obligation(s))",
            if ok { "ok" } else { "failed" },
            if vendor_ok { "ok" } else { "mismatch" },
            obligations
                .get("leanFiles")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            obligations
                .get("obligations")
                .and_then(Value::as_array)
                .map(|items| items.len())
                .unwrap_or(0)
        );
        for violation in obligations
            .get("violations")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            println!("  fail: {}", violation.as_str().unwrap_or("?"));
        }
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn formal_sync(json: bool, project: &Path, source: &Path) -> ExitCode {
    let report = match rhobuilder_core::formal::sync(project, source) {
        Ok(report) => report,
        Err(error) => return fail(json, "FormalError", None, &error.to_string(), 2),
    };
    if json {
        return print_json(json!({
            "schema": rhobuilder_core::schema("formal-sync"),
            "ok": true,
            "report": report,
        }));
    }
    println!(
        "formal sync: {} files ({})",
        report.get("written").and_then(Value::as_u64).unwrap_or(0),
        report
            .get("treeSha256")
            .and_then(Value::as_str)
            .unwrap_or("?")
    );
    ExitCode::SUCCESS
}

#[allow(clippy::too_many_arguments)]
fn scaffold_new(
    json: bool,
    registry_path: &Path,
    templates: &Path,
    target: &Path,
    design_path: &Path,
    template_name: &str,
    plan_only: bool,
    apply_now: bool,
    force: bool,
) -> ExitCode {
    if plan_only && apply_now {
        return fail(
            json,
            "Usage",
            None,
            "--plan and --apply are mutually exclusive",
            2,
        );
    }
    let design = match rhobuilder_core::design::load(design_path) {
        Ok(design) => design,
        Err(error) => return fail(json, "DesignError", None, &error.to_string(), 2),
    };
    let validation = design.validate();
    let registry = match rhobuilder_registry::Registry::load(registry_path) {
        Ok(registry) => registry,
        Err(error) => return fail(json, "RegistryError", None, &error.to_string(), 2),
    };
    let reuse = rhobuilder_registry::reuse::check(&design, &registry);
    let primitives: BTreeMap<String, rhobuilder_core::contracts::PrimitiveInfo> = registry
        .primitives
        .iter()
        .filter_map(|primitive| {
            primitive.component.as_ref().map(|component| {
                (
                    primitive.id.clone(),
                    rhobuilder_core::contracts::PrimitiveInfo {
                        component: component.clone(),
                        owner: primitive.owner.clone(),
                        resolve_arity: if primitive.owner == "r-gen" { 2 } else { 3 },
                    },
                )
            })
        })
        .collect();
    if !validation.ok || !reuse.ok {
        let envelope = json!({
            "schema": rhobuilder_core::schema("scaffold-refusal"),
            "ok": false,
            "verdict": &reuse.verdict,
            "validation": &validation,
            "reuse": &reuse,
        });
        if json {
            if let Err(error) = emit_json(&envelope) {
                eprintln!("rhobuilder: cannot encode JSON: {error}");
                return ExitCode::from(1);
            }
        } else {
            println!("scaffold refused: fix the design first");
            for conflict in &reuse.conflicts {
                println!(
                    "  conflict {} vs {}: {}",
                    conflict.component, conflict.primitive, conflict.detail
                );
            }
        }
        return ExitCode::from(2);
    }
    let Some(template) = rhobuilder_core::scaffold::Template::parse(template_name) else {
        return fail(json, "Usage", None, "template must be full or headless", 2);
    };
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => return fail(json, "Io", None, &error.to_string(), 2),
    };
    let normalized = match rhobuilder_core::scaffold::normalize_target(target) {
        Ok(path) => path,
        Err(error) => return fail(json, "OutsideWorkspace", None, &error.to_string(), 2),
    };
    let target_abs = if normalized.is_absolute() {
        normalized
    } else {
        cwd.join(normalized)
    };
    if !target_abs.starts_with(&cwd) && !force {
        return fail(
            json,
            "OutsideWorkspace",
            Some("target"),
            "target is outside the working directory; pass --force to allow it",
            2,
        );
    }
    let template_root = if templates.is_absolute() {
        templates.to_path_buf()
    } else {
        cwd.join(templates)
    };
    let reuse_value = match serde_json::to_value(&reuse) {
        Ok(value) => value,
        Err(error) => return fail(json, "Json", None, &error.to_string(), 2),
    };

    if apply_now {
        let written = match rhobuilder_core::scaffold::apply_with_primitives(
            &template_root,
            &design,
            template,
            &reuse_value,
            &target_abs,
            force,
            &primitives,
        ) {
            Ok(written) => written,
            Err(error) => {
                let code = match &error {
                    rhobuilder_core::scaffold::ScaffoldError::TargetNotEmpty(_) => 1,
                    _ => 2,
                };
                return fail(json, "ScaffoldError", None, &error.to_string(), code);
            }
        };
        if json {
            return print_json(json!({
                "schema": rhobuilder_core::schema("scaffold"),
                "ok": true,
                "target": target_abs.display().to_string(),
                "template": template.name(),
                "app": design.app.slug,
                "written": written,
            }));
        }
        println!(
            "scaffold: {} files written to {} ({})",
            written,
            target_abs.display(),
            template.name()
        );
        return ExitCode::SUCCESS;
    }

    let plan = match rhobuilder_core::scaffold::plan_with_primitives(
        &template_root,
        &design,
        template,
        &reuse_value,
        &target_abs,
        &primitives,
    ) {
        Ok(plan) => plan,
        Err(error) => return fail(json, "ScaffoldError", None, &error.to_string(), 2),
    };
    if json {
        return print_json(json!({
            "schema": rhobuilder_core::schema("scaffold-plan"),
            "ok": true,
            "plan": plan,
        }));
    }
    println!(
        "scaffold plan: {} files for {} at {} ({})",
        plan.files.len(),
        plan.app_slug,
        plan.target,
        plan.template
    );
    for path in plan.files.iter().take(40) {
        println!("  {path}");
    }
    if plan.files.len() > 40 {
        println!("  ... {} more", plan.files.len() - 40);
    }
    ExitCode::SUCCESS
}

fn design_check(json: bool, registry_path: &Path, path: &Path) -> ExitCode {
    let design = match rhobuilder_core::design::load(path) {
        Ok(design) => design,
        Err(error) => return fail(json, "DesignError", None, &error.to_string(), 2),
    };
    let validation = design.validate();
    let registry = match rhobuilder_registry::Registry::load(registry_path) {
        Ok(registry) => registry,
        Err(error) => return fail(json, "RegistryError", None, &error.to_string(), 2),
    };
    let reuse = rhobuilder_registry::reuse::check(&design, &registry);
    let ok = validation.ok && reuse.ok;
    let envelope = json!({
        "schema": rhobuilder_core::schema("design-check"),
        "ok": ok,
        "verdict": &reuse.verdict,
        "app": {"name": &design.app.name, "slug": &design.app.slug},
        "validation": &validation,
        "reuse": &reuse,
    });
    if json {
        if let Err(error) = emit_json(&envelope) {
            eprintln!("rhobuilder: cannot encode JSON: {error}");
            return ExitCode::from(1);
        }
    } else {
        println!(
            "design {}: {}",
            design.app.slug,
            if ok { "pass" } else { "refuse" }
        );
        for check in &validation.checks {
            if check.status != "ok" {
                println!("  {} {}: {}", check.status, check.id, check.detail);
            }
        }
        for check in &reuse.checks {
            if check.status != "ok" {
                println!("  {} {}: {}", check.status, check.id, check.detail);
            }
        }
        for conflict in &reuse.conflicts {
            println!(
                "  conflict {} vs {}: {}",
                conflict.component, conflict.primitive, conflict.detail
            );
        }
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
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

fn decoded(value: &Value) -> Value {
    RhoValue::decode(value)
        .map(|decoded| decoded.to_json())
        .unwrap_or_else(|_| value.clone())
}

#[allow(clippy::too_many_arguments)]
async fn chain_call(
    json: bool,
    registry: &Path,
    network: &str,
    keys_path: &Path,
    file: Option<&Path>,
    term_text: Option<&str>,
    execute: bool,
    allow_live: bool,
    confirm: Option<&str>,
) -> ExitCode {
    if !execute {
        return fail(json, "Usage", None, "chain call needs --execute", 2);
    }
    let (profile, observed, _) = match probe_checked(json, registry, network).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    if network != "devnet" {
        if !allow_live {
            return fail(
                json,
                "LiveExecutionGated",
                None,
                "testnet call needs --allow-live",
                5,
            );
        }
        let expected = confirm_token(&profile, &observed, None);
        if confirm != Some(expected.as_str()) {
            return fail(
                json,
                "ConfirmationMismatch",
                Some("confirm"),
                &format!("pass --confirm '{expected}'"),
                5,
            );
        }
    }
    let source = match (file, term_text) {
        (Some(path), None) => match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => return fail(json, "Io", Some("file"), &error.to_string(), 2),
        },
        (None, Some(text)) => text.to_string(),
        _ => {
            return fail(
                json,
                "Usage",
                None,
                "chain call needs exactly one of --file or --term",
                2,
            )
        }
    };
    let term = executor::minify(&source);
    if term.len() > executor::MAX_TERM_BYTES {
        return fail(json, "GateFailure", None, "term exceeds 64 KiB", 5);
    }
    let keys_path = match checked_keys_path(keys_path, network) {
        Ok(path) => path,
        Err(error) => return fail(json, "UnsafeKeyPath", Some("keys"), &error, 6),
    };
    let keys = match keystore::load(&keys_path) {
        Ok(keys) => keys,
        Err(error) => return fail(json, "Keystore", Some("keys"), &error.to_string(), 6),
    };
    if let Err(error) = keystore::assert_safe_for(&keys_path, network) {
        return fail(json, "UnsafeKey", Some("keys"), &error.to_string(), 6);
    }
    let key = match keys.key("deployer") {
        Ok(key) => key,
        Err(error) => return fail(json, "MissingKey", Some("keys"), &error.to_string(), 6),
    };
    let client = match ChainClient::new(profile.endpoint.clone().unwrap_or_default(), key) {
        Ok(client) => client
            .with_shard(profile.shard_id.clone())
            .with_phlo(8_000_000, 1),
        Err(error) => return fail(json, "ChainError", None, &error.to_string(), 4),
    };
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let op = format!("call:{}:{nonce}", digest_text(&term));
    let cache = PathBuf::from(".ops")
        .join("ops")
        .join(network)
        .join("calls");
    let outcome = match client.deploy_term_cached(&term, &op, &cache).await {
        Ok(outcome) => outcome,
        Err(error) => return fail(json, "ChainError", None, &error.to_string(), 4),
    };
    if !outcome.is_success() {
        return fail(
            json,
            "CallFailed",
            None,
            &format!("deploy ended {}", outcome.status.status_text()),
            1,
        );
    }
    if !outcome.finalized {
        return fail(json, "CallFailed", None, "deploy was not finalized", 1);
    }
    let result: Vec<Value> = outcome
        .status
        .deploy_result()
        .unwrap_or(&[])
        .iter()
        .map(decoded)
        .collect();
    let envelope = json!({
        "schema": rhobuilder_core::schema("chain-call"),
        "ok": true,
        "network": network,
        "deployId": outcome.deploy_id,
        "blockHash": outcome.block_hash(),
        "finalized": outcome.finalized,
        "status": outcome.status.status_text(),
        "result": result,
    });
    if json {
        print_json(envelope)
    } else {
        println!(
            "chain call {network}: {} ({} value(s))",
            outcome.status.status_text(),
            result.len()
        );
        ExitCode::SUCCESS
    }
}

async fn chain_deploy_status(
    json: bool,
    registry: &Path,
    network: &str,
    deploy_id: &str,
) -> ExitCode {
    let (profile, _, _) = match probe_checked(json, registry, network).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    let endpoint = profile.endpoint.clone().unwrap_or_default();
    let url = format!(
        "{}/api/v1/deploy-status/{}",
        endpoint.trim_end_matches('/'),
        deploy_id
    );
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
    {
        Ok(client) => client,
        Err(error) => return fail(json, "Http", None, &error.to_string(), 4),
    };
    let response = match client
        .get(&url)
        .header("accept", "application/json")
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => return fail(json, "NodeUnavailable", None, &error.to_string(), 4),
    };
    let status = response.status();
    let body = match response.text().await {
        Ok(body) => body,
        Err(error) => return fail(json, "NodeUnavailable", None, &error.to_string(), 4),
    };
    if !status.is_success() {
        return fail(
            json,
            "BadResponse",
            None,
            &format!("deploy-status returned {status}: {body}"),
            4,
        );
    }
    let value: Value = match serde_json::from_str(&body) {
        Ok(value) => value,
        Err(error) => return fail(json, "BadResponse", None, &error.to_string(), 4),
    };
    if json {
        print_json(json!({
            "schema": rhobuilder_core::schema("chain-deploy-status"),
            "ok": true,
            "network": network,
            "deployId": deploy_id,
            "status": value,
        }))
    } else {
        println!("{body}");
        ExitCode::SUCCESS
    }
}

fn check_project(json: bool, project: &Path, profile: &str) -> ExitCode {
    let report = match rhobuilder_gates::check(project, profile) {
        Ok(report) => report,
        Err(error) => return fail(json, "GateError", None, &error.to_string(), 2),
    };
    let failed = report.gates.iter().any(|gate| gate.status == "fail");
    if json {
        if let Err(error) = emit_json(&json!(report)) {
            eprintln!("rhobuilder: cannot encode JSON: {error}");
            return ExitCode::from(1);
        }
    } else {
        println!(
            "check {profile}: {}",
            if report.ok {
                "ok"
            } else if failed {
                "failed"
            } else {
                "not_run"
            }
        );
        for gate in &report.gates {
            println!("  {:16} {}", gate.name, gate.status);
        }
    }
    if failed {
        ExitCode::from(1)
    } else if !report.ok {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}

async fn chain_status(json: bool, registry: &Path, network: &str) -> ExitCode {
    let (_, observed, checks) = match probe_checked(json, registry, network).await {
        Ok(value) => value,
        Err(code) => return code,
    };
    if json {
        return print_json(json!({
            "schema": rhobuilder_core::schema("chain-status"),
            "ok": true,
            "network": network,
            "status": observed.status.clone(),
            "observed": observed,
            "checks": checks,
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&observed.status).unwrap_or_default()
    );
    ExitCode::SUCCESS
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

fn error_copy(kind: &str) -> (&'static str, Option<&'static str>) {
    match kind {
        "Usage" => ("usage", Some("Check the command arguments and try again.")),
        "IdentityMismatch" | "NetworkGated" => (
            "identity_mismatch",
            Some("The node is not the network this plan or profile expects. Re-probe before planning."),
        ),
        "LiveExecutionGated" => (
            "gated",
            Some("A live action needs explicit approval, a funded key and the printed confirmation token."),
        ),
        "UnsafeKey" | "UnsafeKeyPath" | "MissingKey" => (
            "unsafe_key",
            Some("Use a mode-0600 key file under the project or installation; never paste a key into chat."),
        ),
        "MissingDependency" => (
            "missing_dependency",
            Some("Install or verify the shared primitives that this app reuses before deploying it."),
        ),
        "PartialRoot" | "Verification" => (
            "partial_root",
            Some("Stop and reconcile the existing root; do not redeploy a commodity primitive."),
        ),
        "NodeUnavailable" | "BadResponse" | "Http" => (
            "node_unavailable",
            Some("The node is unreachable, unhealthy or rate-limited. Check the devnet/testnet status."),
        ),
        "GateFailure" => (
            "gate_failure",
            Some("A generated-project gate failed. Open the gate output before continuing."),
        ),
        "DesignError" | "RegistryError" | "InstallPlanError" => (
            "setup",
            Some("Fix the local design, registry or install-plan input before retrying."),
        ),
        _ => ("error", None),
    }
}

fn fail(json: bool, kind: &str, field: Option<&str>, message: &str, code: u8) -> ExitCode {
    if json {
        let (status, hint) = error_copy(kind);
        let mut error = json!({"type": kind, "message": message, "status": status});
        if let Some(hint) = hint {
            error["hint"] = Value::String(hint.to_string());
        }
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
    fn cli_parses_baseline_plan() {
        let cli = Cli::try_parse_from([
            "rhobuilder",
            "--json",
            "baseline",
            "plan",
            "--network",
            "testnet",
        ])
        .expect("parse");
        assert!(cli.json);
    }

    #[test]
    fn cli_parses_scaffold_plan() {
        let cli = Cli::try_parse_from([
            "rhobuilder",
            "--json",
            "new",
            "/tmp/demo",
            "--design",
            "fixtures/designs/booking.json",
            "--template",
            "headless",
            "--plan",
            "--force",
        ])
        .expect("parse");
        assert!(cli.json);
        assert_eq!(cli.registry, PathBuf::from("registry"));
        assert_eq!(cli.templates, PathBuf::from("templates/dapp"));
    }

    #[test]
    fn cli_parses_registry_drift() {
        let cli = Cli::try_parse_from([
            "rhobuilder",
            "--json",
            "registry",
            "drift",
            "--source-root",
            "/tmp/rhobuilder-siblings",
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
