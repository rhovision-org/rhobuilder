//! Stable product metadata and the shared envelope vocabulary.
//!
//! The design model, reuse gate and term builders land here in P1/P2. This crate stays
//! dependency-free until then so the CLI and the JSON contract can be built without the
//! chain client.

pub mod contracts;
pub mod design;
pub mod formal;
pub mod scaffold;

/// The product name used in JSON envelopes and docs.
pub const PRODUCT: &str = "rhobuilder";

/// The crate version, compiled in.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A versioned schema identifier, for example `rhobuilder.version/v1`.
pub fn schema(name: &str) -> String {
    format!("rhobuilder.{name}/v1")
}

/// Lowercase hex for a hash digest. Kept here so the sha2 major-version change does not
/// leak into formatting code.
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
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
