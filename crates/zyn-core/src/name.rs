use std::fmt;

/// A JavaScript package name as it appears in manifests and dependency edges.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageName(String);

impl PackageName {
    pub fn new(value: impl Into<String>) -> Result<Self, PackageNameError> {
        let value = value.into();
        if value.is_empty() {
            return Err(PackageNameError::Empty);
        }
        if value.chars().any(char::is_whitespace) {
            return Err(PackageNameError::Whitespace);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageNameError {
    Empty,
    Whitespace,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_scoped_names() {
        let name = PackageName::new("@scope/pkg");

        assert!(matches!(name, Ok(name) if name.as_str() == "@scope/pkg"));
    }

    #[test]
    fn rejects_empty_names() {
        assert_eq!(PackageName::new(""), Err(PackageNameError::Empty));
    }

    #[test]
    fn rejects_whitespace() {
        assert_eq!(
            PackageName::new("not a package"),
            Err(PackageNameError::Whitespace)
        );
    }
}
