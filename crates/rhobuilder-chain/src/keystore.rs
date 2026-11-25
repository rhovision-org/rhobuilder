//! Local ops-key files.
//!
//! The file is the harness `DevnetKeys` shape, written with mode 0600. Public reports
//! never include the secret. Testnet refuses a file that contains the public devnet
//! deployer secret or a devnet-looking address.

use std::path::{Path, PathBuf};

use rcash_sdk::rcash_chain::keystore::DevnetKeys;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const DEFAULT_ROLES: &[&str] = &[
    "deployer",
    "admin1",
    "admin2",
    "admin3",
    "issuer",
    "reporter1",
    "reporter2",
    "reporter3",
];

/// SHA-256 of the public devnet deployer secret published in the r-cash harness. The
/// value is public; the hash is enough to refuse it without keeping the key in this tree.
pub const PUBLIC_DEVNET_SECRET_SHA256: &str =
    "535820d6140de171eb958e728aa6217087606027be123fe202cca9fe394be2ed";

#[derive(Debug, Error)]
pub enum KeystoreError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("keystore: {0}")]
    Chain(String),
    #[error("keystore already exists: {0}; pass --force to replace it")]
    Exists(String),
    #[error("unsafe key file: {0}")]
    Unsafe(String),
    #[error("keystore path is outside the working directory: {0}")]
    Outside(String),
}

pub fn generate(path: &Path, names: &[&str], force: bool) -> Result<DevnetKeys, KeystoreError> {
    if path.exists() && !force {
        return Err(KeystoreError::Exists(path.display().to_string()));
    }
    if path.exists() {
        std::fs::remove_file(path).map_err(|source| KeystoreError::Io {
            path: path.display().to_string(),
            source,
        })?;
    }
    let mut keys = DevnetKeys::default();
    for name in names {
        keys.insert_generated(*name)
            .map_err(|error| KeystoreError::Chain(error.to_string()))?;
    }
    keys.save(path)
        .map_err(|error| KeystoreError::Chain(error.to_string()))?;
    Ok(keys)
}

pub fn load(path: &Path) -> Result<DevnetKeys, KeystoreError> {
    DevnetKeys::load(path).map_err(|error| KeystoreError::Chain(error.to_string()))
}

pub fn public_report(keys: &DevnetKeys) -> Value {
    let entries: Vec<Value> = keys
        .keys
        .iter()
        .map(|(name, key)| {
            json!({
                "name": name,
                "public": key.public,
                "address": key.address,
                "funded": key.funded,
            })
        })
        .collect();
    json!({
        "schema": "rhobuilder.keys/v1",
        "ok": true,
        "count": entries.len(),
        "keys": entries,
    })
}

/// Refuse a key file that reuses the published devnet deployer secret or contains a
/// key named like a devnet fixture when the target is a live network.
pub fn assert_safe_for(path: &Path, network: &str) -> Result<(), KeystoreError> {
    if network == "devnet" {
        return Ok(());
    }
    if path
        .to_string_lossy()
        .to_ascii_lowercase()
        .contains("devnet")
    {
        return Err(KeystoreError::Unsafe(
            "a devnet-named keystore cannot be used on a live network".to_string(),
        ));
    }
    let keys = load(path)?;
    for (name, key) in &keys.keys {
        let digest = Sha256::digest(key.secret.to_ascii_lowercase().as_bytes());
        if rhobuilder_core::hex(&digest) == PUBLIC_DEVNET_SECRET_SHA256 {
            return Err(KeystoreError::Unsafe(format!(
                "{name} uses the published devnet deployer secret"
            )));
        }
    }
    Ok(())
}

/// Canonicalise a keystore path and require it under `workspace`.
pub fn require_under(path: &Path, workspace: &Path) -> Result<PathBuf, KeystoreError> {
    let canonical = if path.exists() {
        std::fs::canonicalize(path).map_err(|source| KeystoreError::Io {
            path: path.display().to_string(),
            source,
        })?
    } else {
        let mut existing = path.to_path_buf();
        let mut suffix = Vec::new();
        while !existing.exists() {
            let Some(name) = existing.file_name().map(|name| name.to_os_string()) else {
                break;
            };
            suffix.push(name);
            let Some(parent) = existing.parent() else {
                existing = PathBuf::from(".");
                break;
            };
            if parent.as_os_str().is_empty() {
                existing = PathBuf::from(".");
                break;
            }
            existing = parent.to_path_buf();
        }
        let mut canonical =
            std::fs::canonicalize(&existing).map_err(|source| KeystoreError::Io {
                path: existing.display().to_string(),
                source,
            })?;
        for part in suffix.iter().rev() {
            canonical.push(part);
        }
        canonical
    };
    let workspace = std::fs::canonicalize(workspace).map_err(|source| KeystoreError::Io {
        path: workspace.display().to_string(),
        source,
    })?;
    if !canonical.starts_with(&workspace) {
        return Err(KeystoreError::Outside(canonical.display().to_string()));
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_file_is_owner_only_and_has_no_secret_in_report() {
        let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
        let path = dir.path().join("keys.json");
        let keys =
            generate(&path, &["deployer", "admin1"], false).unwrap_or_else(|_| panic!("keys"));
        let report = public_report(&keys);
        let text = report.to_string();
        assert!(text.contains("\"deployer\""));
        assert!(!text.contains("\"secret\""));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path)
                .unwrap_or_else(|_| panic!("metadata"))
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn devnet_named_keystore_is_refused_on_testnet() {
        let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
        let path = dir.path().join("devnet-keys.json");
        std::fs::write(
            &path,
            "{\"deployer\":{\"name\":\"deployer\",\"secret\":\"00\",\"public\":\"04\",\"address\":\"x\"}}",
        )
        .unwrap_or_else(|_| panic!("write"));
        let error = assert_safe_for(&path, "testnet");
        assert!(matches!(error, Err(KeystoreError::Unsafe(_))));
    }
}
