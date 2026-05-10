use std::fmt;

/// A resolved package version.
///
/// This intentionally stores the original package-manager version string. npm-compatible
/// parsing belongs in the npm-facing crate, while the core model only needs a stable value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageVersion(String);

impl PackageVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, PackageVersionError> {
        let value = value.into();
        if value.is_empty() {
            return Err(PackageVersionError::Empty);
        }
        if value.chars().any(char::is_whitespace) {
            return Err(PackageVersionError::Whitespace);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackageVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageVersionError {
    Empty,
    Whitespace,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_npm_versions() {
        let version = PackageVersion::new("1.2.3-beta.1");

        assert!(matches!(version, Ok(version) if version.as_str() == "1.2.3-beta.1"));
    }
}
