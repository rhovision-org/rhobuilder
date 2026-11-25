//! Gate orchestration and vendoring.
//!
//! P2 vendors the proven Python checkers and harness plumbing into generated repos and
//! exposes `rhobuilder check` as an orchestration layer over the project's Makefile
//! targets. There is one gate implementation, not two.

/// The phase this subsystem is scheduled for.
pub const STAGE: &str = "P2";

/// Whether gate orchestration is present.
pub fn implemented() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_is_declared() {
        assert_eq!(STAGE, "P2");
        assert!(!implemented());
    }
}
