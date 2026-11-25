//! The on-chain primitive registry: sibling ingest, network manifests, graph and drift.
//!
//! P0 keeps the crate as a compiling boundary. P1 adds the schemas and the ingest of
//! sibling `configs/`, `deployments/`, `spec/graph/` and `spec/flows/` artifacts.

/// The phase this subsystem is scheduled for.
pub const STAGE: &str = "P1";

/// Whether the registry implementation is present.
pub fn implemented() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_is_declared() {
        assert_eq!(STAGE, "P1");
        assert!(!implemented());
    }
}
