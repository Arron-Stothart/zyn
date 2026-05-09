pub const NAME: &str = "zyn";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_package_identity() {
        assert_eq!(NAME, "zyn");
        assert!(!VERSION.is_empty());
    }
}
