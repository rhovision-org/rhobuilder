//! Domain core for __app_name__. Scaffold: add the term builders and codecs here.

pub const APP: &str = "__APP__";

pub fn slug() -> &'static str {
    "__app__"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_stable() {
        assert_eq!(slug(), "__app__");
    }
}
