//! Network identity and read-only chain access.
//!
//! P1 implements the identity probe against a registry network profile. Deploying and
//! signing stay in the pinned `rcash-chain` crate; this crate adds only the endpoints and
//! comparisons that crate does not expose.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// A committed network profile (`registry/networks/<network>.json`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkProfile {
    #[serde(default)]
    pub schema: Option<String>,
    pub network: String,
    pub chain: String,
    #[serde(default)]
    pub network_id: Option<String>,
    pub shard_id: String,
    #[serde(default)]
    pub genesis_block_hash: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    pub status: String,
    pub deploy: String,
    #[serde(default)]
    pub faucet: Option<Faucet>,
    #[serde(default)]
    pub phlo: Option<Phlo>,
    #[serde(default)]
    pub observed: Option<Observed>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Faucet {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub amount: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Phlo {
    #[serde(default)]
    pub price: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Observed {
    #[serde(default)]
    pub probed_at: Option<String>,
    #[serde(default)]
    pub node_build: Option<String>,
    #[serde(default)]
    pub height: Option<i64>,
    #[serde(default)]
    pub network_id: Option<String>,
    #[serde(default)]
    pub genesis_block_hash: Option<String>,
}

/// What a probe actually saw.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkObservation {
    pub endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_build: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finalized_height: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genesis_block_hash: Option<String>,
    pub status: Value,
    pub capabilities: Value,
    pub shards: Value,
}

/// One identity or capability check.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub name: String,
    pub status: String,
    pub detail: String,
}

impl Check {
    fn ok(name: &str, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "ok".into(),
            detail: detail.into(),
        }
    }

    fn warn(name: &str, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "warn".into(),
            detail: detail.into(),
        }
    }

    fn fail(name: &str, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "fail".into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("network {network} is gated: {reason}")]
    Gated { network: String, reason: String },
    #[error("network {network} has no endpoint")]
    NoEndpoint { network: String },
    #[error("cannot read profile: {0}")]
    Io(String),
    #[error("invalid profile JSON: {0}")]
    Json(String),
    #[error("HTTP request failed: {0}")]
    Http(String),
    #[error("invalid response from {endpoint}: {detail}")]
    BadResponse { endpoint: String, detail: String },
    #[error("identity mismatch: {field} expected {expected}, observed {observed}")]
    IdentityMismatch {
        field: String,
        expected: String,
        observed: String,
    },
}

pub fn load_profile(path: &Path) -> Result<NetworkProfile, ProbeError> {
    let text = std::fs::read_to_string(path).map_err(|error| ProbeError::Io(error.to_string()))?;
    serde_json::from_str(&text).map_err(|error| ProbeError::Json(error.to_string()))
}

async fn get_json(
    client: &reqwest::Client,
    endpoint: &str,
    path: &str,
) -> Result<Value, ProbeError> {
    let url = format!("{}{}", endpoint.trim_end_matches('/'), path);
    let response = client
        .get(&url)
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|error| ProbeError::Http(format!("{path}: {error}")))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| ProbeError::Http(format!("{path}: {error}")))?;
    if !status.is_success() {
        return Err(ProbeError::BadResponse {
            endpoint: endpoint.to_string(),
            detail: format!("{path} returned {status}"),
        });
    }
    serde_json::from_str(&body).map_err(|error| ProbeError::BadResponse {
        endpoint: endpoint.to_string(),
        detail: format!("{path}: {error}"),
    })
}

fn text_at<'a>(value: &'a Value, pointer: &str) -> Option<&'a str> {
    value.pointer(pointer).and_then(Value::as_str)
}

fn int_at(value: &Value, pointer: &str) -> Option<i64> {
    value.pointer(pointer).and_then(Value::as_i64)
}

/// Probe the public read surface. No exploratory deploy and no signed deploy.
pub async fn probe(profile: &NetworkProfile) -> Result<NetworkObservation, ProbeError> {
    if profile.status == "unavailable" || profile.deploy == "refused" {
        return Err(ProbeError::Gated {
            network: profile.network.clone(),
            reason: format!("status={} deploy={}", profile.status, profile.deploy),
        });
    }
    let endpoint = profile
        .endpoint
        .clone()
        .ok_or_else(|| ProbeError::NoEndpoint {
            network: profile.network.clone(),
        })?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent(concat!("rhobuilder/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| ProbeError::Http(error.to_string()))?;

    let status = get_json(&client, &endpoint, "/api/v1/status").await?;
    let blocks = get_json(&client, &endpoint, "/api/blocks/0/0").await?;
    let capabilities = get_json(&client, &endpoint, "/api/v1/capabilities")
        .await
        .unwrap_or(Value::Null);
    let shards = get_json(&client, &endpoint, "/api/v1/shards")
        .await
        .unwrap_or(Value::Null);
    let finalized = get_json(&client, &endpoint, "/api/last-finalized-block")
        .await
        .unwrap_or(Value::Null);

    let genesis_block_hash = blocks
        .as_array()
        .and_then(|items| items.first())
        .and_then(|block| block.get("blockHash"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let node_version = text_at(&status, "/version/node").map(str::to_owned);
    let node_build = node_version
        .as_deref()
        .and_then(|text| text.rsplit_once('('))
        .and_then(|(_, tail)| tail.split(')').next())
        .map(str::to_owned)
        .or_else(|| node_version.clone());
    let height = int_at(&status, "/latestBlockNumber").or_else(|| {
        blocks
            .as_array()
            .and_then(|items| items.first())
            .and_then(|block| block.get("blockNumber"))
            .and_then(Value::as_i64)
    });
    let finalized_height =
        int_at(&finalized, "/blockInfo/blockNumber").or_else(|| int_at(&finalized, "/blockNumber"));

    Ok(NetworkObservation {
        endpoint,
        network_id: text_at(&status, "/networkId").map(str::to_owned),
        shard_id: text_at(&status, "/shardId").map(str::to_owned),
        node_version,
        node_build,
        height,
        finalized_height,
        genesis_block_hash,
        status,
        capabilities,
        shards,
    })
}

/// Compare a probe with the committed profile. Identity fields are hard failures; the node
/// build and faucet are warnings so a node upgrade does not read as a network reset.
pub fn compare(profile: &NetworkProfile, observed: &NetworkObservation) -> Vec<Check> {
    let mut checks = Vec::new();

    match (&profile.network_id, &observed.network_id) {
        (Some(expected), Some(found)) if expected == found => {
            checks.push(Check::ok("networkId", found.clone()));
        }
        (Some(expected), Some(found)) => {
            checks.push(Check::fail(
                "networkId",
                format!("expected {expected}, observed {found}"),
            ));
        }
        (Some(expected), None) => {
            checks.push(Check::fail(
                "networkId",
                format!("expected {expected}, observed none"),
            ));
        }
        (None, Some(found)) => checks.push(Check::warn(
            "networkId",
            format!("unpinned; observed {found}"),
        )),
        (None, None) => checks.push(Check::warn("networkId", "unpinned; not observed")),
    }

    match &observed.shard_id {
        Some(found) if found == &profile.shard_id => {
            checks.push(Check::ok("shardId", found.clone()));
        }
        Some(found) => checks.push(Check::fail(
            "shardId",
            format!("expected {}, observed {found}", profile.shard_id),
        )),
        None => checks.push(Check::fail("shardId", "observed none")),
    }

    match (&profile.genesis_block_hash, &observed.genesis_block_hash) {
        (Some(expected), Some(found)) if expected == found => {
            checks.push(Check::ok("genesisBlockHash", found.clone()));
        }
        (Some(expected), Some(found)) => checks.push(Check::fail(
            "genesisBlockHash",
            format!("expected {expected}, observed {found}"),
        )),
        (Some(expected), None) => checks.push(Check::fail(
            "genesisBlockHash",
            format!("expected {expected}, observed none"),
        )),
        (None, Some(found)) => checks.push(Check::warn(
            "genesisBlockHash",
            format!("unpinned; observed {found}"),
        )),
        (None, None) => checks.push(Check::warn("genesisBlockHash", "unpinned; not observed")),
    }

    if let Some(expected) = profile
        .observed
        .as_ref()
        .and_then(|observed| observed.node_build.as_deref())
    {
        match &observed.node_build {
            Some(found) if found.contains(expected) => {
                checks.push(Check::ok("nodeBuild", expected.to_string()));
            }
            Some(found) => checks.push(Check::warn(
                "nodeBuild",
                format!("expected prefix {expected}, observed {found}"),
            )),
            None => checks.push(Check::warn("nodeBuild", "not observed")),
        }
    }

    if profile.faucet.is_some() {
        let faucet = observed.capabilities.get("faucet").and_then(Value::as_bool);
        match faucet {
            Some(true) => checks.push(Check::ok("faucet", "available")),
            Some(false) => checks.push(Check::warn(
                "faucet",
                "profile expects one; node reports false",
            )),
            None => checks.push(Check::warn("faucet", "capability not reported")),
        }
    }

    checks
}

/// The first failing check, if any.
pub fn first_failure(checks: &[Check]) -> Option<&Check> {
    checks.iter().find(|check| check.status == "fail")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn profile() -> NetworkProfile {
        NetworkProfile {
            schema: Some("rhobuilder.network/v1".into()),
            network: "testnet".into(),
            chain: "rholo".into(),
            network_id: Some("testnet-133b1d307b175012".into()),
            shard_id: "/testnet".into(),
            genesis_block_hash: Some(
                "133b1d307b1750129cf0cfb69c4c8f950da21c4e8db2a6a076464d0ccfd98d44".into(),
            ),
            endpoint: Some("https://testnet.rhobot.net".into()),
            status: "live".into(),
            deploy: "gated".into(),
            faucet: Some(Faucet {
                path: Some("/api/faucet".into()),
                amount: Some(30_000_000),
            }),
            phlo: None,
            observed: Some(Observed {
                probed_at: None,
                node_build: Some("0a1e02362a7c".into()),
                height: None,
                network_id: None,
                genesis_block_hash: None,
            }),
        }
    }

    fn observation() -> NetworkObservation {
        NetworkObservation {
            endpoint: "https://testnet.rhobot.net".into(),
            network_id: Some("testnet-133b1d307b175012".into()),
            shard_id: Some("/testnet".into()),
            node_version: Some(
                "rholo node 0.1.0 (0a1e02362a7c34e474e068ffd47469c8d4be707a)".into(),
            ),
            node_build: Some("0a1e02362a7c34e474e068ffd47469c8d4be707a".into()),
            height: Some(1248),
            finalized_height: Some(1244),
            genesis_block_hash: Some(
                "133b1d307b1750129cf0cfb69c4c8f950da21c4e8db2a6a076464d0ccfd98d44".into(),
            ),
            status: json!({"networkId":"testnet-133b1d307b175012","shardId":"/testnet"}),
            capabilities: json!({"faucet": true}),
            shards: json!({"primaryShard":"/testnet"}),
        }
    }

    #[test]
    fn matching_identity_has_no_failures() {
        let checks = compare(&profile(), &observation());
        assert!(first_failure(&checks).is_none(), "{checks:?}");
    }

    #[test]
    fn wrong_network_id_is_a_hard_failure() {
        let mut observed = observation();
        observed.network_id = Some("testnet-other".into());
        let checks = compare(&profile(), &observed);
        let failure = first_failure(&checks).expect("a failure");
        assert_eq!(failure.name, "networkId");
    }

    #[test]
    fn wrong_genesis_is_a_hard_failure() {
        let mut observed = observation();
        observed.genesis_block_hash = Some("aa".into());
        let checks = compare(&profile(), &observed);
        let failure = first_failure(&checks).expect("a failure");
        assert_eq!(failure.name, "genesisBlockHash");
    }

    #[test]
    fn node_build_drift_is_only_a_warning() {
        let mut observed = observation();
        observed.node_build = Some("new-build".into());
        let checks = compare(&profile(), &observed);
        assert!(first_failure(&checks).is_none());
        assert!(checks
            .iter()
            .any(|check| check.name == "nodeBuild" && check.status == "warn"));
    }

    #[test]
    fn profile_parses_camel_case() {
        let text = r#"{
          "schema":"rhobuilder.network/v1","network":"testnet","chain":"rholo",
          "networkId":"testnet-133b1d307b175012","shardId":"/testnet",
          "genesisBlockHash":"133b","endpoint":"https://testnet.rhobot.net",
          "status":"live","deploy":"gated"
        }"#;
        let parsed: NetworkProfile = serde_json::from_str(text).expect("profile");
        assert_eq!(parsed.shard_id, "/testnet");
        assert_eq!(parsed.deploy, "gated");
    }

    #[test]
    fn mainnet_is_gated_before_http() {
        let mut mainnet = profile();
        mainnet.network = "mainnet".into();
        mainnet.status = "unavailable".into();
        mainnet.deploy = "refused".into();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let error = runtime.block_on(probe(&mainnet)).expect_err("gated");
        assert!(matches!(error, ProbeError::Gated { .. }));
    }
}
