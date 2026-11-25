//! Stable product metadata and the shared envelope vocabulary.
//!
//! The design model, reuse gate and term builders land here in P1/P2. This crate stays
//! dependency-free until then so the CLI and the JSON contract can be built without the
//! chain client.

/// The product name used in JSON envelopes and docs.
pub const PRODUCT: &str = "rhobuilder";

/// The crate version, compiled in.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A versioned schema identifier, for example `rhobuilder.version/v1`.
pub fn schema(name: &str) -> String {
    format!("rhobuilder.{name}/v1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_is_versioned() {
        assert_eq!(schema("version"), "rhobuilder.version/v1");
    }

    #[test]
    fn product_is_stable() {
        assert_eq!(PRODUCT, "rhobuilder");
    }
}
