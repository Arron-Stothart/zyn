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
        if value.starts_with('@') {
            let Some((scope, name)) = value.split_once('/') else {
                return Err(PackageNameError::InvalidScopedName);
            };
            if scope.len() == 1 || name.is_empty() || name.contains('/') {
                return Err(PackageNameError::InvalidScopedName);
            }
        } else if value.contains('/') {
            return Err(PackageNameError::Slash);
        }
        if value == "." || value == ".." {
            return Err(PackageNameError::Reserved);
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
    InvalidScopedName,
    Slash,
    Reserved,
}

impl fmt::Display for PackageNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("package name cannot be empty"),
            Self::Whitespace => f.write_str("package name cannot contain whitespace"),
            Self::InvalidScopedName => f.write_str("scoped package name must be `@scope/name`"),
            Self::Slash => f.write_str("unscoped package name cannot contain `/`"),
            Self::Reserved => f.write_str("package name cannot be `.` or `..`"),
        }
    }
}

impl std::error::Error for PackageNameError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_scoped_names() {
        let name = PackageName::new("@zyn/two-dot");

        assert!(matches!(name, Ok(name) if name.as_str() == "@zyn/two-dot"));
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

    #[test]
    fn rejects_invalid_scoped_names() {
        assert_eq!(
            PackageName::new("@scope"),
            Err(PackageNameError::InvalidScopedName)
        );
        assert_eq!(
            PackageName::new("@scope/"),
            Err(PackageNameError::InvalidScopedName)
        );
        assert_eq!(
            PackageName::new("@scope/pkg/extra"),
            Err(PackageNameError::InvalidScopedName)
        );
    }

    #[test]
    fn rejects_slashes_in_unscoped_names() {
        assert_eq!(PackageName::new("foo/bar"), Err(PackageNameError::Slash));
    }
}
