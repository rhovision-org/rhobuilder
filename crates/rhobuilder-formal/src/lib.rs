//! Formal surface: vendored `RhoApp`, hash checks and per-arm obligations.
//!
//! The implementation lives in `rhobuilder_core::formal`; this crate is the stable
//! library surface for callers that do not depend on the core internals. `proved-model`
//! is the ceiling; nothing here is a Rholang refinement.

pub const STAGE: &str = "P4";

pub use rhobuilder_core::formal::{check, collect, obligations, sync, tree_hash, vendor_into};

pub fn implemented() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_surface_is_present() {
        assert_eq!(STAGE, "P4");
        assert!(implemented());
    }
}
