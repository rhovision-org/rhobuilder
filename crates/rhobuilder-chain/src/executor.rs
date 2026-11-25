//! Deterministic install execution over the pinned `rcash-sdk` client.
//!
//! The plan is data: a pinned commit, ordered terms, root key and required Root keys.
//! Rendering happens here, after dependencies are known. Every write goes through
//! `Client::deploy_term_cached`, which keeps a write-ahead signed body per operation id,
//! waits for finality and refuses to reuse an operation id for a different term.
//!
//! This module never prints or serialises a secret. State files contain only public URIs,
//! deploy ids and block hashes.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rcash_sdk::rcash_chain::keystore::{DevnetKey, DevnetKeys};
use rcash_sdk::rcash_chain::rho_expr::RhoValue;
use rcash_sdk::rcash_chain::ChainError;
use rcash_sdk::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const INSTALL_SCHEMA: &str = "rhobuilder.install/v1";
pub const STATE_SCHEMA: &str = "rhobuilder.install-state/v1";
pub const MAX_TERM_BYTES: usize = 64 * 1024;

#[derive(Debug, Error)]
pub enum ExecutorError {
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
    #[error("install plan is missing {0}")]
    MissingPlan(String),
    #[error("unresolved placeholder {placeholder} in {term}")]
    Unresolved { term: String, placeholder: String },
    #[error("term {term} is {size} bytes, over the {limit}-byte node limit")]
    TermTooLarge {
        term: String,
        size: usize,
        limit: usize,
    },
    #[error("chain: {0}")]
    Chain(#[from] ChainError),
    #[error("state: {0}")]
    State(String),
    #[error("missing key {0}")]
    MissingKey(String),
    #[error("deploy {step} ended {status}")]
    DeployFailed { step: String, status: String },
    #[error("deploy {step} was processed but not finalized")]
    NotFinalized { step: String },
    #[error("root {app} is present but does not verify: {detail}")]
    PartialRoot { app: String, detail: String },
    #[error("Root verification failed for {app}: {detail}")]
    Verification { app: String, detail: String },
    #[error("refusing live execution: {0}")]
    Gated(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPlan {
    pub schema: String,
    pub app: String,
    pub repo: String,
    pub commit: String,
    pub source_root: String,
    pub root_key: String,
    #[serde(default = "default_arity")]
    pub resolve_arity: usize,
    pub phlo: PlanPhlo,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub required_keys: Vec<String>,
    #[serde(default)]
    pub placeholders: Vec<String>,
    #[serde(default)]
    pub steps: Vec<PlanStep>,
}

fn default_arity() -> usize {
    3
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanPhlo {
    pub limit: i64,
    pub price: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStep {
    pub name: String,
    pub term: String,
    #[serde(default)]
    pub root: bool,
    #[serde(default)]
    pub placeholders: Vec<String>,
    #[serde(default)]
    pub captures: Vec<Capture>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    pub placeholder: String,
    pub from: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub index: Option<usize>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallState {
    pub schema: String,
    pub network: String,
    pub app: String,
    pub commit: String,
    #[serde(default)]
    pub root_uri: Option<String>,
    #[serde(default)]
    pub captured: BTreeMap<String, String>,
    #[serde(default)]
    pub steps: BTreeMap<String, StepResult>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub updated_at: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepResult {
    pub name: String,
    pub term: String,
    pub status: String,
    #[serde(default)]
    pub deploy_id: Option<String>,
    #[serde(default)]
    pub block_hash: Option<String>,
    #[serde(default)]
    pub finalized: bool,
    #[serde(default)]
    pub reused: bool,
    #[serde(default)]
    pub uris: Vec<String>,
    #[serde(default)]
    pub uri_map: BTreeMap<String, String>,
    #[serde(default)]
    pub term_sha256: String,
    #[serde(default)]
    pub bytes: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepPreview {
    pub name: String,
    pub term: String,
    pub bytes: usize,
    pub term_sha256: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanPreview {
    pub app: String,
    pub commit: String,
    pub root_key: String,
    pub steps: Vec<StepPreview>,
    pub required_keys: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallReport {
    pub schema: String,
    pub network: String,
    pub app: String,
    pub commit: String,
    pub status: String,
    #[serde(default)]
    pub root_uri: Option<String>,
    #[serde(default)]
    pub captured: BTreeMap<String, String>,
    #[serde(default)]
    pub steps: Vec<StepResult>,
    #[serde(default)]
    pub required_keys: Vec<String>,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub identity_digest: Option<String>,
}

pub struct ExecutorConfig<'a> {
    pub network: &'a str,
    pub node_url: &'a str,
    pub shard: &'a str,
    pub registry_root: &'a Path,
    pub terms_dir: Option<&'a Path>,
    pub key_name: &'a str,
    pub admin_names: [&'a str; 3],
    pub issuer_name: &'a str,
    pub reporter_names: [&'a str; 3],
    pub roots: &'a BTreeMap<String, String>,
    pub state_path: &'a Path,
    pub op_cache: &'a Path,
    pub min_lock_blocks: i64,
    pub dry_run: bool,
}

pub struct Executor<'a> {
    client: Client,
    keys: &'a DevnetKeys,
    config: ExecutorConfig<'a>,
}

impl<'a> Executor<'a> {
    pub fn new(
        keys: &'a DevnetKeys,
        config: ExecutorConfig<'a>,
        phlo: &PlanPhlo,
    ) -> Result<Self, ExecutorError> {
        let key = keys.key(config.key_name)?;
        let client = Client::new(config.node_url, key)?
            .with_shard(config.shard)
            .with_phlo(phlo.limit, phlo.price);
        Ok(Self {
            client,
            keys,
            config,
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn execute(&self, plan: &InstallPlan) -> Result<InstallReport, ExecutorError> {
        if plan.schema != "rhobuilder.install-plan/v1" {
            return Err(ExecutorError::MissingPlan(format!(
                "unexpected plan schema {}",
                plan.schema
            )));
        }
        let mut state = self.load_state(plan)?;
        let mut values = self.base_values()?;
        if let Some(root) = &state.root_uri {
            values.insert("__ROOT_URI__".to_string(), root.clone());
        }
        for (placeholder, value) in &state.captured {
            values.insert(placeholder.clone(), value.clone());
        }
        let mut report = InstallReport {
            schema: INSTALL_SCHEMA.to_string(),
            network: self.config.network.to_string(),
            app: plan.app.clone(),
            commit: plan.commit.clone(),
            status: if self.config.dry_run {
                "planned".to_string()
            } else {
                "running".to_string()
            },
            root_uri: state.root_uri.clone(),
            captured: state.captured.clone(),
            steps: Vec::new(),
            required_keys: plan.required_keys.clone(),
            verified: false,
            dry_run: self.config.dry_run,
            identity_digest: None,
        };

        if let Some(root) = state.root_uri.clone() {
            if self.config.dry_run {
                report.status = "adopted".to_string();
                report.steps = state.steps.values().cloned().collect();
                return Ok(report);
            }
            let found = crate::root::verify_keys(
                &self.client,
                &root,
                &plan.required_keys,
                plan.resolve_arity,
                &self.config.op_cache.join("verify"),
            )
            .await?;
            let missing: Vec<String> = plan
                .required_keys
                .iter()
                .filter(|key| !found.contains_key(*key))
                .cloned()
                .collect();
            if missing.is_empty() {
                report.status = "adopted".to_string();
                report.steps = state.steps.values().cloned().collect();
                report.verified = true;
                state.status = "verified".to_string();
                self.save_state(plan, &state)?;
                return Ok(report);
            }
            state.status = "partial".to_string();
            self.save_state(plan, &state)?;
            return Err(ExecutorError::PartialRoot {
                app: plan.app.clone(),
                detail: format!(
                    "root {root} is present but required keys {missing:?} do not resolve; \
                     stop and reconcile instead of redeploying components"
                ),
            });
        }

        for step in &plan.steps {
            if step.root && state.root_uri.is_some() {
                let record = state
                    .steps
                    .get(&step.name)
                    .cloned()
                    .unwrap_or_else(|| StepResult {
                        name: step.name.clone(),
                        term: step.term.clone(),
                        status: "adopted".to_string(),
                        ..StepResult::default()
                    });
                report.steps.push(record);
                continue;
            }
            let term_text = self.read_term(plan, step)?;
            let rendered = render_term(&term_text, &values, &step.term)?;
            let digest = sha256_hex(rendered.as_bytes());
            let bytes = rendered.len();
            if self.config.dry_run {
                report.steps.push(StepResult {
                    name: step.name.clone(),
                    term: step.term.clone(),
                    status: "planned".to_string(),
                    term_sha256: digest,
                    bytes,
                    ..StepResult::default()
                });
                continue;
            }
            let op = operation_id(
                self.config.network,
                &plan.app,
                &step.name,
                &plan.commit,
                &digest,
            );
            let outcome = match self
                .client
                .deploy_term_cached(&rendered, &op, self.config.op_cache)
                .await
            {
                Ok(outcome) => outcome,
                Err(error) => {
                    state.status = "partial".to_string();
                    self.save_state(plan, &state)?;
                    return Err(ExecutorError::DeployFailed {
                        step: step.name.clone(),
                        status: error.to_string(),
                    });
                }
            };
            if !outcome.is_success() {
                let status = outcome.status.status_text();
                state.status = "partial".to_string();
                self.save_state(plan, &state)?;
                return Err(ExecutorError::DeployFailed {
                    step: step.name.clone(),
                    status,
                });
            }
            if !outcome.finalized {
                state.status = "partial".to_string();
                self.save_state(plan, &state)?;
                return Err(ExecutorError::NotFinalized {
                    step: step.name.clone(),
                });
            }
            let (uris, uri_map) = decode_result(outcome.status.deploy_result());
            let mut record = StepResult {
                name: step.name.clone(),
                term: step.term.clone(),
                status: "processedWithSuccess".to_string(),
                deploy_id: Some(outcome.deploy_id.clone()),
                block_hash: outcome.block_hash().map(str::to_string),
                finalized: true,
                reused: false,
                uris: uris.clone(),
                uri_map: uri_map.clone(),
                term_sha256: digest,
                bytes,
            };
            if step.root {
                let Some(root) = uris.first().cloned() else {
                    state.status = "partial".to_string();
                    self.save_state(plan, &state)?;
                    return Err(ExecutorError::DeployFailed {
                        step: step.name.clone(),
                        status: "no root URI in reply".to_string(),
                    });
                };
                state.root_uri = Some(root.clone());
                values.insert("__ROOT_URI__".to_string(), root.clone());
                report.root_uri = Some(root);
                record.status = "rootCaptured".to_string();
            }
            for capture in &step.captures {
                let value = match capture.from.as_str() {
                    "uris" => capture.index.and_then(|index| uris.get(index)).cloned(),
                    "map" => capture
                        .key
                        .as_ref()
                        .and_then(|key| uri_map.get(key))
                        .cloned(),
                    _ => None,
                };
                if let Some(value) = value {
                    state
                        .captured
                        .insert(capture.placeholder.clone(), value.clone());
                    values.insert(capture.placeholder.clone(), value);
                }
            }
            report.steps.push(record.clone());
            state.steps.insert(step.name.clone(), record);
            state.status = "running".to_string();
            self.save_state(plan, &state)?;
        }

        report.captured = state.captured.clone();
        report.root_uri = state.root_uri.clone();
        if self.config.dry_run {
            report.status = "planned".to_string();
            return Ok(report);
        }
        let Some(root) = state.root_uri.clone() else {
            state.status = "partial".to_string();
            self.save_state(plan, &state)?;
            return Err(ExecutorError::MissingPlan(format!(
                "{} did not capture a root URI",
                plan.app
            )));
        };
        let found = crate::root::verify_keys(
            &self.client,
            &root,
            &plan.required_keys,
            plan.resolve_arity,
            &self.config.op_cache.join("verify"),
        )
        .await?;
        let missing: Vec<String> = plan
            .required_keys
            .iter()
            .filter(|key| !found.contains_key(*key))
            .cloned()
            .collect();
        if !missing.is_empty() {
            state.status = "partial".to_string();
            self.save_state(plan, &state)?;
            return Err(ExecutorError::Verification {
                app: plan.app.clone(),
                detail: format!("missing Root keys {missing:?}"),
            });
        }
        report.verified = true;
        report.status = "verified".to_string();
        state.status = "verified".to_string();
        self.save_state(plan, &state)?;
        Ok(report)
    }

    fn base_values(&self) -> Result<BTreeMap<String, String>, ExecutorError> {
        let deployer = self
            .keys
            .keys
            .get(self.config.key_name)
            .ok_or_else(|| ExecutorError::MissingKey(self.config.key_name.to_string()))?;
        let admins = admin_literal(self.keys, &self.config.admin_names)?;
        let reporters = quoted_list(self.keys, &self.config.reporter_names)?;
        let issuer = self
            .keys
            .keys
            .get(self.config.issuer_name)
            .ok_or_else(|| ExecutorError::MissingKey(self.config.issuer_name.to_string()))?;
        let mut values = BTreeMap::new();
        values.insert("__THRESHOLD__".into(), "2".into());
        values.insert("__INSTALLER_PUB__".into(), deployer.public.clone());
        values.insert("__ADMIN_KEYS__".into(), admins.clone());
        values.insert("__ADMINS__".into(), admins);
        values.insert("__PLATFORM_OWNER__".into(), deployer.address.clone());
        values.insert("__PROTOCOL_OWNER__".into(), deployer.address.clone());
        values.insert("__ISSUER_ADDRESS__".into(), issuer.address.clone());
        values.insert("__ALLOWLIST__".into(), reporters);
        values.insert(
            "__BOOK_PARAMS__".into(),
            "{\"minAmount\": 1, \"defaultExpiryBlocks\": 720, \"maxExpiryBlocks\": 1296000, \"maxMemoBytes\": 128, \"maxOpIdBytes\": 64, \"maxStipend\": 100000000}".into(),
        );
        values.insert(
            "__MIN_LOCK_BLOCKS__".into(),
            self.config.min_lock_blocks.to_string(),
        );
        for (app, root) in self.config.roots {
            let key = match app.as_str() {
                "r-cash" => "__RCASH_ROOT_URI__",
                "r-gen" => "__RGEN_ROOT_URI__",
                "r-sure" => "__RSURE_ROOT_URI__",
                "r-share" => "__RSHARE_ROOT_URI__",
                "r-print" => "__RPRINT_ROOT_URI__",
                _ => continue,
            };
            values.insert(key.to_string(), root.clone());
        }
        Ok(values)
    }

    fn read_term(&self, plan: &InstallPlan, step: &PlanStep) -> Result<String, ExecutorError> {
        let term_path = match self.config.terms_dir {
            Some(dir) => dir.join(&step.term),
            None => term_path(
                self.config.registry_root,
                &plan.app,
                &plan.commit,
                &step.term,
            ),
        };
        std::fs::read_to_string(&term_path).map_err(|source| ExecutorError::Io {
            path: term_path.display().to_string(),
            source,
        })
    }

    fn load_state(&self, plan: &InstallPlan) -> Result<InstallState, ExecutorError> {
        if !self.config.state_path.is_file() {
            return Ok(InstallState {
                schema: STATE_SCHEMA.to_string(),
                network: self.config.network.to_string(),
                app: plan.app.clone(),
                commit: plan.commit.clone(),
                status: "new".to_string(),
                ..InstallState::default()
            });
        }
        let text = std::fs::read_to_string(self.config.state_path).map_err(|source| {
            ExecutorError::Io {
                path: self.config.state_path.display().to_string(),
                source,
            }
        })?;
        let state: InstallState =
            serde_json::from_str(&text).map_err(|source| ExecutorError::Json {
                path: self.config.state_path.display().to_string(),
                source,
            })?;
        if state.commit != plan.commit {
            return Err(ExecutorError::State(format!(
                "state commit {} does not match plan {}",
                state.commit, plan.commit
            )));
        }
        Ok(state)
    }

    fn save_state(&self, plan: &InstallPlan, state: &InstallState) -> Result<(), ExecutorError> {
        let mut state = state.clone();
        state.schema = STATE_SCHEMA.to_string();
        state.network = self.config.network.to_string();
        state.app = plan.app.clone();
        state.commit = plan.commit.clone();
        state.updated_at = Some(now_seconds());
        let text = serde_json::to_string_pretty(&state).map_err(|source| ExecutorError::Json {
            path: self.config.state_path.display().to_string(),
            source,
        })?;
        if let Some(parent) = self.config.state_path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| ExecutorError::Io {
                path: parent.display().to_string(),
                source,
            })?;
        }
        let tmp = self.config.state_path.with_extension("json.tmp");
        std::fs::write(&tmp, text).map_err(|source| ExecutorError::Io {
            path: tmp.display().to_string(),
            source,
        })?;
        std::fs::rename(&tmp, self.config.state_path).map_err(|source| ExecutorError::Io {
            path: self.config.state_path.display().to_string(),
            source,
        })?;
        Ok(())
    }
}

fn term_path(registry_root: &Path, app: &str, commit: &str, term: &str) -> PathBuf {
    registry_root
        .join("install")
        .join(app)
        .join(commit)
        .join("terms")
        .join(term)
}

fn operation_id(network: &str, app: &str, step: &str, commit: &str, term: &str) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        network,
        app,
        step,
        &commit[..commit.len().min(8)],
        &term[..term.len().min(8)]
    )
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    rhobuilder_core::hex(&hasher.finalize())
}

fn admin_literal(keys: &DevnetKeys, names: &[&str; 3]) -> Result<String, ExecutorError> {
    let mut parts = Vec::new();
    for name in names {
        let key: &DevnetKey = keys
            .keys
            .get(*name)
            .ok_or_else(|| ExecutorError::MissingKey((*name).to_string()))?;
        parts.push(format!("\"{}\".hexToBytes()", key.public));
    }
    Ok(format!("[{}]", parts.join(", ")))
}

fn quoted_list(keys: &DevnetKeys, names: &[&str; 3]) -> Result<String, ExecutorError> {
    let mut parts = Vec::new();
    for name in names {
        let key: &DevnetKey = keys
            .keys
            .get(*name)
            .ok_or_else(|| ExecutorError::MissingKey((*name).to_string()))?;
        parts.push(format!("\"{}\"", key.public));
    }
    Ok(format!("[{}]", parts.join(", ")))
}

/// Conservative renderer: substitute known placeholders, then remove line comments and
/// collapse whitespace outside string literals. Parity with the sibling `minify_rho.py`.
pub fn render_term(
    text: &str,
    values: &BTreeMap<String, String>,
    term: &str,
) -> Result<String, ExecutorError> {
    let mut out = text.to_string();
    for (key, value) in values {
        out = out.replace(key, value);
    }
    if let Some(placeholder) = unresolved(&out) {
        return Err(ExecutorError::Unresolved {
            term: term.to_string(),
            placeholder,
        });
    }
    let minified = minify(&out);
    if minified.len() > MAX_TERM_BYTES {
        return Err(ExecutorError::TermTooLarge {
            term: term.to_string(),
            size: minified.len(),
            limit: MAX_TERM_BYTES,
        });
    }
    Ok(minified)
}

pub fn minify(text: &str) -> String {
    let mut parts = Vec::new();
    for line in text.lines() {
        let clean = strip_line_comment(line);
        let trimmed = clean.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed.to_string());
        }
    }
    parts.join(" ")
}

fn strip_line_comment(line: &str) -> String {
    let mut out = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let chars: Vec<char> = line.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
            out.push(ch);
        } else if ch == '/' && index + 1 < chars.len() && chars[index + 1] == '/' {
            break;
        } else {
            out.push(ch);
        }
        index += 1;
    }
    out
}

fn unresolved(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] == b'_' && bytes[index + 1] == b'_' {
            let start = index;
            let mut cursor = index + 2;
            while cursor + 1 < bytes.len() {
                if bytes[cursor] == b'_' && bytes[cursor + 1] == b'_' {
                    let between = &text[start + 2..cursor];
                    let valid = !between.is_empty()
                        && between
                            .chars()
                            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_');
                    if valid {
                        return Some(format!("__{between}__"));
                    }
                    break;
                }
                cursor += 1;
            }
            index += 2;
        } else {
            index += 1;
        }
    }
    None
}

fn decode_result(result: Option<&[Value]>) -> (Vec<String>, BTreeMap<String, String>) {
    let mut uris = Vec::new();
    let mut map = BTreeMap::new();
    for value in result.unwrap_or(&[]) {
        if let Ok(decoded) = RhoValue::decode(value) {
            collect_strings(&decoded, &mut uris);
            collect_map(&decoded, &mut map);
        } else {
            collect_json_strings(value, &mut uris);
        }
    }
    (uris, map)
}

fn collect_strings(value: &RhoValue, out: &mut Vec<String>) {
    match value {
        RhoValue::Str(text) | RhoValue::Uri(text) => out.push(text.clone()),
        RhoValue::Par(items)
        | RhoValue::Tuple(items)
        | RhoValue::List(items)
        | RhoValue::Set(items) => {
            for item in items {
                collect_strings(item, out);
            }
        }
        RhoValue::Map(map) => {
            for item in map.values() {
                collect_strings(item, out);
            }
        }
        _ => {}
    }
}

fn collect_map(value: &RhoValue, out: &mut BTreeMap<String, String>) {
    match value {
        RhoValue::Map(map) => {
            for (key, item) in map {
                if let Some(text) = item.as_str() {
                    out.insert(key.clone(), text.to_string());
                } else {
                    collect_map(item, out);
                }
            }
        }
        RhoValue::List(items) | RhoValue::Tuple(items) | RhoValue::Par(items) => {
            if items.len() == 2 {
                if let (Some(key), Some(text)) = (items[0].as_str(), items[1].as_str()) {
                    out.insert(key.to_string(), text.to_string());
                    return;
                }
            }
            for item in items {
                collect_map(item, out);
            }
        }
        _ => {}
    }
}

/// Placeholder values for a read-only preview. These are not keys and never sign.
pub fn dummy_values() -> BTreeMap<String, String> {
    let public = format!("04{}", "00".repeat(64));
    let admin = format!(
        "[{}]",
        (0..3)
            .map(|_| format!("\"{public}\".hexToBytes()"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let reporters = format!(
        "[{}]",
        (0..3)
            .map(|_| format!("\"{public}\""))
            .collect::<Vec<_>>()
            .join(", ")
    );
    BTreeMap::from([
        ("__ROOT_URI__".to_string(), "rho:id:dryrun-root".to_string()),
        ("__THRESHOLD__".to_string(), "2".to_string()),
        ("__INSTALLER_PUB__".to_string(), public.clone()),
        ("__ADMIN_KEYS__".to_string(), admin.clone()),
        ("__ADMINS__".to_string(), admin),
        ("__PLATFORM_OWNER__".to_string(), "11111111111111111111111111111111".to_string()),
        ("__PROTOCOL_OWNER__".to_string(), "11111111111111111111111111111111".to_string()),
        ("__ISSUER_ADDRESS__".to_string(), "11111111111111111111111111111111".to_string()),
        ("__ALLOWLIST__".to_string(), reporters),
        ("__BOOK_PARAMS__".to_string(), "{\"minAmount\": 1, \"defaultExpiryBlocks\": 720, \"maxExpiryBlocks\": 1296000, \"maxMemoBytes\": 128, \"maxOpIdBytes\": 64, \"maxStipend\": 100000000}".to_string()),
        ("__MIN_LOCK_BLOCKS__".to_string(), "10".to_string()),
        ("__RCASH_ROOT_URI__".to_string(), "rho:id:dryrun-rcash".to_string()),
        ("__RGEN_ROOT_URI__".to_string(), "rho:id:dryrun-rgen".to_string()),
        ("__RSURE_ROOT_URI__".to_string(), "rho:id:dryrun-rsure".to_string()),
        ("__RSHARE_ROOT_URI__".to_string(), "rho:id:dryrun-rshare".to_string()),
        ("__RPRINT_ROOT_URI__".to_string(), "rho:id:dryrun-rprint".to_string()),
    ])
}

/// Render every step without a client. Used by `baseline plan`/`--dry-run`.
pub fn preview(
    plan: &InstallPlan,
    registry_root: &Path,
    values: &BTreeMap<String, String>,
) -> Result<PlanPreview, ExecutorError> {
    preview_with_terms(plan, registry_root, None, values)
}

/// Render every step from an explicit terms directory (generated app deploys).
pub fn preview_with_terms(
    plan: &InstallPlan,
    registry_root: &Path,
    terms_dir: Option<&Path>,
    values: &BTreeMap<String, String>,
) -> Result<PlanPreview, ExecutorError> {
    let mut values = values.clone();
    let mut steps = Vec::new();
    for step in &plan.steps {
        let term_path = match terms_dir {
            Some(dir) => dir.join(&step.term),
            None => term_path(registry_root, &plan.app, &plan.commit, &step.term),
        };
        let text = std::fs::read_to_string(&term_path).map_err(|source| ExecutorError::Io {
            path: term_path.display().to_string(),
            source,
        })?;
        if step.root && !values.contains_key("__ROOT_URI__") {
            values.insert("__ROOT_URI__".to_string(), "rho:id:dryrun-root".to_string());
        }
        let rendered = render_term(&text, &values, &step.term)?;
        for capture in &step.captures {
            values.insert(
                capture.placeholder.clone(),
                format!(
                    "rho:id:dryrun-{}",
                    capture.placeholder.trim_matches('_').to_lowercase()
                ),
            );
        }
        steps.push(StepPreview {
            name: step.name.clone(),
            term: step.term.clone(),
            bytes: rendered.len(),
            term_sha256: sha256_hex(rendered.as_bytes()),
            status: "planned".to_string(),
        });
    }
    Ok(PlanPreview {
        app: plan.app.clone(),
        commit: plan.commit.clone(),
        root_key: plan.root_key.clone(),
        steps,
        required_keys: plan.required_keys.clone(),
    })
}

fn collect_json_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(text) if text.starts_with("rho:id:") => out.push(text.clone()),
        Value::Array(items) => {
            for item in items {
                collect_json_strings(item, out);
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                collect_json_strings(item, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("__ROOT_URI__".to_string(), "rho:id:root".to_string()),
            (
                "__ADMINS__".to_string(),
                "[\"aa\".hexToBytes()]".to_string(),
            ),
        ])
    }

    #[test]
    fn render_substitutes_and_minifies() {
        let text = "// comment\nnew x in {\n  x!(\"__ROOT_URI__\") // tail\n}\n";
        let rendered = render_term(text, &values(), "test.rho").unwrap_or_default();
        assert!(rendered.contains("\"rho:id:root\""));
        assert!(!rendered.contains("comment"));
    }

    #[test]
    fn unresolved_placeholder_is_refused() {
        let error = render_term("ret!(__MISSING__)", &values(), "test.rho");
        assert!(matches!(error, Err(ExecutorError::Unresolved { .. })));
    }

    #[test]
    fn minify_keeps_string_contents_and_separates_tokens() {
        assert_eq!(minify("a // x\n b"), "a b");
        assert_eq!(minify("x!(\"a // b\")"), "x!(\"a // b\")");
    }

    #[test]
    fn oversized_term_is_refused() {
        let text = "x".repeat(MAX_TERM_BYTES + 1);
        let error = render_term(&text, &BTreeMap::new(), "big.rho");
        assert!(matches!(error, Err(ExecutorError::TermTooLarge { .. })));
    }

    #[test]
    fn captures_map_pairs() {
        let value = RhoValue::List(vec![
            RhoValue::Str("identity-directory".into()),
            RhoValue::Uri("rho:id:abc".into()),
        ]);
        let mut map = BTreeMap::new();
        collect_map(&value, &mut map);
        assert_eq!(
            map.get("identity-directory"),
            Some(&"rho:id:abc".to_string())
        );
    }
}
