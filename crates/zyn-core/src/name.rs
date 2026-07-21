use std::fmt;

/// A JavaScript package name as it appears in manifests and dependency edges.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageName(String);

impl PackageName {
    pub fn new(value: impl Into<String>) -> Result<Self, PackageNameError> {
        let value = value.into();
        validate_package_name(&value)?;
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
    InvalidStart(char),
    Whitespace,
    InvalidScopedName,
    InvalidCharacter(char),
    Reserved,
}

impl fmt::Display for PackageNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("package name cannot be empty"),
            Self::InvalidStart(character) => {
                write!(f, "package name cannot start with `{character}`")
            }
            Self::Whitespace => f.write_str("package name cannot contain whitespace"),
            Self::InvalidScopedName => f.write_str("scoped package name must be `@scope/name`"),
            Self::InvalidCharacter(character) => {
                write!(f, "package name cannot contain `{character}`")
            }
            Self::Reserved => f.write_str("package name is reserved"),
        }
    }
}

impl std::error::Error for PackageNameError {}

fn validate_package_name(value: &str) -> Result<(), PackageNameError> {
    if value.is_empty() {
        return Err(PackageNameError::Empty);
    }
    if let Some(character @ ('.' | '-' | '_')) = value.chars().next() {
        return Err(PackageNameError::InvalidStart(character));
    }
    if value.chars().any(char::is_whitespace) {
        return Err(PackageNameError::Whitespace);
    }
    if value.eq_ignore_ascii_case("node_modules") || value.eq_ignore_ascii_case("favicon.ico") {
        return Err(PackageNameError::Reserved);
    }

    if let Some(scoped_name) = value.strip_prefix('@') {
        let Some((scope, name)) = scoped_name.split_once('/') else {
            return Err(PackageNameError::InvalidScopedName);
        };
        if scope.is_empty() || name.is_empty() || name.contains('/') || name.starts_with('.') {
            return Err(PackageNameError::InvalidScopedName);
        }
        validate_name_part(scope)?;
        validate_name_part(name)
    } else {
        validate_name_part(value)
    }
}

fn validate_name_part(value: &str) -> Result<(), PackageNameError> {
    match value
        .chars()
        .find(|character| !is_name_character(*character))
    {
        Some(character) => Err(PackageNameError::InvalidCharacter(character)),
        None => Ok(()),
    }
}

fn is_name_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(
            character,
            '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')'
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_scoped_names() {
        let name = PackageName::new("@zyn/two-dot");

        assert!(matches!(name, Ok(name) if name.as_str() == "@zyn/two-dot"));
    }

    #[test]
    fn accepts_existing_npm_package_name_forms() {
        for value in [
            "react",
            "a.b-c_d",
            "Foo",
            "1.2.3",
            "@scope/package",
            "@scope/package.js",
        ] {
            assert!(PackageName::new(value).is_ok(), "{value} should be valid");
        }
    }

    #[test]
    fn rejects_empty_names() {
        assert_eq!(PackageName::new(""), Err(PackageNameError::Empty));
    }

    #[test]
    fn rejects_whitespace() {
        for value in ["not a package", " package", "package "] {
            assert_eq!(PackageName::new(value), Err(PackageNameError::Whitespace));
        }
    }

    #[test]
    fn rejects_invalid_starting_characters() {
        for (value, character) in [(".bin", '.'), ("-package", '-'), ("_package", '_')] {
            assert_eq!(
                PackageName::new(value),
                Err(PackageNameError::InvalidStart(character))
            );
        }
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
        assert_eq!(
            PackageName::new("@scope/.pkg"),
            Err(PackageNameError::InvalidScopedName)
        );
    }

    #[test]
    fn rejects_slashes_in_unscoped_names() {
        assert_eq!(
            PackageName::new("foo/bar"),
            Err(PackageNameError::InvalidCharacter('/'))
        );
    }

    #[test]
    fn rejects_characters_that_are_not_url_safe() {
        for (value, character) in [
            ("package:name", ':'),
            ("package\\name", '\\'),
            ("café", 'é'),
        ] {
            assert_eq!(
                PackageName::new(value),
                Err(PackageNameError::InvalidCharacter(character))
            );
        }
    }

    #[test]
    fn rejects_reserved_names_without_case_sensitivity() {
        for value in ["node_modules", "Node_Modules", "favicon.ico", "FAVICON.ICO"] {
            assert_eq!(PackageName::new(value), Err(PackageNameError::Reserved));
        }
    }
}
