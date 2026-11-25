//! Signed Root verification.
//!
//! The node's exploratory deploy cannot see the native registry, so verification is a
//! small signed read. One deploy resolves every requested key and returns a list in the
//! same order. A `429`/`503` before submission is `not_run`, never `verified`.

use std::collections::BTreeMap;
use std::path::Path;

use rcash_sdk::rcash_chain::rho_expr::RhoValue;
use rcash_sdk::Client;
use serde_json::Value;

use crate::executor::{minify, ExecutorError, MAX_TERM_BYTES};

pub async fn verify_keys(
    client: &Client,
    root_uri: &str,
    keys: &[String],
    resolve_arity: usize,
    cache_dir: &Path,
) -> Result<BTreeMap<String, String>, ExecutorError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let term = resolve_term(root_uri, keys, resolve_arity);
    let term = minify(&term);
    if term.len() > MAX_TERM_BYTES {
        return Err(ExecutorError::TermTooLarge {
            term: "root-verify".to_string(),
            size: term.len(),
            limit: MAX_TERM_BYTES,
        });
    }
    let height = client.height().await.unwrap_or(0);
    let digest = short_hash(&format!("{root_uri}:{:?}:{height}", keys));
    let op = format!("verify:{digest}");
    let outcome = client.deploy_term_cached(&term, &op, cache_dir).await?;
    if !outcome.is_success() {
        return Err(ExecutorError::DeployFailed {
            step: "root-verify".to_string(),
            status: outcome.status.status_text(),
        });
    }
    let values = decode_strings(outcome.status.deploy_result());
    if values.len() < keys.len() {
        return Err(ExecutorError::Verification {
            app: root_uri.to_string(),
            detail: format!(
                "resolver returned {} value(s) for {} key(s)",
                values.len(),
                keys.len()
            ),
        });
    }
    let mut found = BTreeMap::new();
    for (key, value) in keys.iter().zip(values.iter()) {
        if value != "missing" {
            found.insert(key.clone(), value.clone());
        }
    }
    Ok(found)
}

fn resolve_term(root_uri: &str, keys: &[String], resolve_arity: usize) -> String {
    let channels: Vec<String> = (0..keys.len()).map(|i| format!("c{i}")).collect();
    let outputs: Vec<String> = (0..keys.len()).map(|i| format!("o{i}")).collect();
    let sends: Vec<String> = keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            if resolve_arity >= 3 {
                format!("Root!(\"resolve\", \"{key}\", 1, *c{i})")
            } else {
                format!("Root!(\"resolve\", \"{key}\", *c{i})")
            }
        })
        .collect();
    // Both known roots return `(true, uri, version)`; only the call arity differs
    // (r-cash is 3-argument, r-gen is the 2-argument exception).
    let arms: Vec<String> = (0..keys.len())
        .map(|i| {
            format!(
                "for (@a{i} <- c{i}) {{ match a{i} {{ (true, u, _) => o{i}!(u) _ => o{i}!(\"missing\") }} }}"
            )
        })
        .collect();
    let joins: Vec<String> = (0..keys.len()).map(|i| format!("@u{i} <- o{i}")).collect();
    let values: Vec<String> = (0..keys.len()).map(|i| format!("u{i}")).collect();
    format!(
        "new rl(`rho:registry:lookup`), rCh, Root, {}, {}, deployId(`rho:rchain:deployId`) in {{ \
         rl!(`{root_uri}`, *rCh) | for (Root <- rCh) {{ {} | {} | for ({}) {{ deployId!([{}]) }} }} }}",
        channels.join(", "),
        outputs.join(", "),
        sends.join(" | "),
        arms.join(" | "),
        joins.join("; "),
        values.join(", ")
    )
}

fn decode_strings(result: Option<&[Value]>) -> Vec<String> {
    let mut out = Vec::new();
    for value in result.unwrap_or(&[]) {
        if let Ok(decoded) = RhoValue::decode(value) {
            collect(&decoded, &mut out);
        } else {
            collect_json(value, &mut out);
        }
    }
    out
}

fn collect(value: &RhoValue, out: &mut Vec<String>) {
    match value {
        RhoValue::Str(text) | RhoValue::Uri(text) => out.push(text.clone()),
        RhoValue::Par(items)
        | RhoValue::Tuple(items)
        | RhoValue::List(items)
        | RhoValue::Set(items) => {
            for item in items {
                collect(item, out);
            }
        }
        RhoValue::Map(map) => {
            for item in map.values() {
                collect(item, out);
            }
        }
        _ => {}
    }
}

fn collect_json(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(text) => out.push(text.clone()),
        Value::Array(items) => {
            for item in items {
                collect_json(item, out);
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                collect_json(item, out);
            }
        }
        _ => {}
    }
}

fn short_hash(text: &str) -> String {
    // A small, dependency-free FNV-style tag is enough for an operation id; the cache
    // path is local and the term itself is checked by deploy_term_cached.
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_contains_each_key_and_join() {
        let keys = vec!["component:root".to_string(), "component:clock".to_string()];
        let term = resolve_term("rho:id:abc", &keys, 3);
        assert!(term.contains("component:root"));
        assert!(term.contains("component:clock"));
        assert!(term.contains("@u0 <- o0"));
        assert!(term.contains("@u1 <- o1"));
    }

    #[test]
    fn arity_two_omits_min_version_but_keeps_reply_tuple() {
        let term = resolve_term("rho:id:abc", &["rcash:root".to_string()], 2);
        assert!(term.contains("Root!(\"resolve\", \"rcash:root\", *c0)"));
        assert!(term.contains("(true, u, _)"));
    }
}
