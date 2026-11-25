//! Faucet and devnet funding.
//!
//! Devnet funding is the one path allowed to use the in-container validator key: a single
//! native REV transfer to the ops address, deployed through `rnode` inside the container.
//! The secret is never read or copied. Testnet funding is faucet-only and remains behind
//! the CLI live gate.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use serde_json::{json, Value};
use thiserror::Error;

pub const ONE_REV: i64 = 100_000_000;

#[derive(Debug, Error)]
pub enum FundingError {
    #[error("HTTP: {0}")]
    Http(String),
    #[error("node is {0}; refusing to fund")]
    Degraded(String),
    #[error("docker: {0}")]
    Docker(String),
    #[error("deploy failed: {0}")]
    Deploy(String),
    #[error("timed out waiting for the funding deploy")]
    Timeout,
    #[error("faucet rate-limited; retry later")]
    RateLimited,
    #[error("faucet refused {address}: {detail}")]
    FaucetRefused { address: String, detail: String },
    #[error("gate: {0}")]
    Gated(String),
}

pub struct DevnetFundRequest<'a> {
    pub node_url: &'a str,
    pub container: &'a str,
    pub shard: &'a str,
    pub key_path: &'a str,
    pub target_public: &'a str,
    pub target_address: &'a str,
    pub amount: i64,
    pub timeout: Duration,
}

pub struct FundReport {
    pub deploy_id: String,
    pub block_hash: Option<String>,
    pub status: String,
    pub amount: i64,
    pub address: String,
}

fn client() -> Result<reqwest::Client, FundingError> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent(concat!("rhobuilder/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| FundingError::Http(error.to_string()))
}

async fn get_json(client: &reqwest::Client, url: &str) -> Result<Value, FundingError> {
    let response = client
        .get(url)
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|error| FundingError::Http(error.to_string()))?;
    response
        .json()
        .await
        .map_err(|error| FundingError::Http(error.to_string()))
}

fn transfer_term(target_public: &str, amount: i64) -> String {
    format!(
        "new revAddress(`rho:rev:address`), revVault(`rho:rchain:revVault`), \
         deployerId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`), aCh, vCh in {{ \
         revAddress!(\"fromPublicKey\", \"{target_public}\".hexToBytes(), *aCh) | \
         for (@addr <- aCh) {{ revVault!(\"transfer\", *deployerId, addr, {amount}, *vCh) | \
         for (@ok <- vCh) {{ deployId!(ok) }} }} }}"
    )
}

/// Fund a devnet target from the in-container validator. No secret is read.
pub async fn fund_devnet(request: DevnetFundRequest<'_>) -> Result<FundReport, FundingError> {
    let client = client()?;
    let status = get_json(
        &client,
        &format!("{}/api/v1/status", request.node_url.trim_end_matches('/')),
    )
    .await?;
    let degraded = status
        .get("degraded")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let accepts = status
        .get("acceptsDeploys")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if degraded != "normal" || !accepts {
        return Err(FundingError::Degraded(degraded.to_string()));
    }
    let height = status
        .get("latestBlockNumber")
        .and_then(Value::as_i64)
        .unwrap_or(0);

    let term = transfer_term(request.target_public, request.amount);
    let temporary =
        std::env::temp_dir().join(format!("rhobuilder-fund-{}.rho", std::process::id()));
    std::fs::write(&temporary, &term).map_err(|error| FundingError::Docker(error.to_string()))?;
    let host = temporary.display().to_string();
    let inside = "/tmp/rhobuilder-fund.rho";
    let copy = Command::new("docker")
        .args(["cp", &host, &format!("{}:{inside}", request.container)])
        .output()
        .map_err(|error| FundingError::Docker(error.to_string()))?;
    if !copy.status.success() {
        let _ = std::fs::remove_file(&temporary);
        return Err(FundingError::Docker(
            String::from_utf8_lossy(&copy.stderr).trim().to_string(),
        ));
    }
    let deploy = Command::new("docker")
        .args([
            "exec",
            "-i",
            request.container,
            "rnode",
            "--grpc-host",
            "localhost",
            "deploy",
            "--phlo-limit",
            "2000000",
            "--phlo-price",
            "1",
            "--private-key-path",
            request.key_path,
            "--shard-id",
            request.shard,
            "--valid-after-block-number",
            &height.to_string(),
            inside,
        ])
        .output()
        .map_err(|error| FundingError::Docker(error.to_string()))?;
    let _ = std::fs::remove_file(&temporary);
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&deploy.stdout),
        String::from_utf8_lossy(&deploy.stderr)
    );
    let deploy_id = output
        .split("DeployId is:")
        .nth(1)
        .and_then(|tail| tail.split_whitespace().next())
        .map(str::to_string)
        .ok_or_else(|| FundingError::Deploy(output.chars().take(500).collect()))?;

    let deadline = tokio::time::Instant::now() + request.timeout;
    loop {
        if tokio::time::Instant::now() >= deadline {
            return Err(FundingError::Timeout);
        }
        let status = get_json(
            &client,
            &format!(
                "{}/api/v1/deploy-status/{}",
                request.node_url.trim_end_matches('/'),
                deploy_id
            ),
        )
        .await?;
        if let Some(success) = status.get("ProcessedWithSuccess") {
            let block = success
                .get("block")
                .and_then(|block| block.get("blockHash"))
                .and_then(Value::as_str)
                .map(str::to_string);
            return Ok(FundReport {
                deploy_id,
                block_hash: block,
                status: "processedWithSuccess".to_string(),
                amount: request.amount,
                address: request.target_address.to_string(),
            });
        }
        if let Some(error) = status.get("ProcessedWithError") {
            return Err(FundingError::Deploy(
                error
                    .get("deployError")
                    .and_then(Value::as_str)
                    .unwrap_or("processedWithError")
                    .to_string(),
            ));
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

/// The faucet allocation for a fresh testnet address (one drip).
pub fn faucet_plan(address: &str, drip: i64) -> Value {
    json!({
        "schema": "rhobuilder.faucet-plan/v1",
        "address": address,
        "drips": 1,
        "amount": drip,
        "gross": drip,
        "note": "the public faucet is one drip per address; consolidation is a separate approved step",
    })
}

/// POST the public faucet. The CLI must gate this behind live approval.
pub async fn faucet_drip(node_url: &str, address: &str) -> Result<Value, FundingError> {
    let client = client()?;
    let response = client
        .post(format!("{}/api/faucet", node_url.trim_end_matches('/')))
        .json(&json!({"address": address}))
        .send()
        .await
        .map_err(|error| FundingError::Http(error.to_string()))?;
    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(FundingError::RateLimited);
    }
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| FundingError::Http(error.to_string()))?;
    if !status.is_success() {
        return Err(FundingError::FaucetRefused {
            address: address.to_string(),
            detail: format!("{status}: {body}"),
        });
    }
    serde_json::from_str(&body).map_err(|error| FundingError::FaucetRefused {
        address: address.to_string(),
        detail: format!("non-JSON response: {error}"),
    })
}

/// A devnet-only convenience: the target address must be a public key, not a secret.
pub fn devnet_amount(default_rev: i64) -> i64 {
    default_rev.saturating_mul(ONE_REV)
}

pub fn _path_hint(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_term_has_public_key_and_amount() {
        let term = transfer_term("04aa", 123);
        assert!(term.contains("\"04aa\".hexToBytes()"));
        assert!(term.contains("123"));
    }

    #[test]
    fn faucet_plan_is_one_drip() {
        let plan = faucet_plan("1111", 30_000_000);
        assert_eq!(plan["drips"], 1);
        assert_eq!(plan["amount"], 30_000_000);
    }
}
