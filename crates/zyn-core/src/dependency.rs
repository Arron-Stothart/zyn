use std::fmt;
use std::path::PathBuf;

use crate::{Integrity, PackageName, SourceUrl};

/// The manifest section or relationship that introduced a dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DependencyKind {
    Production,
    Development,
    Optional,
    Peer,
}

/// A dependency as requested by a manifest, before resolution chooses concrete bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyRequest {
    pub alias: PackageName,
    pub spec: DependencySpec,
    pub kind: DependencyKind,
}

impl DependencyRequest {
    pub fn new(alias: PackageName, spec: DependencySpec, kind: DependencyKind) -> Self {
        Self { alias, spec, kind }
    }
}

/// A dependency target before zyn resolves it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencySpec {
    RegistryRange {
        target: PackageName,
        range: VersionRange,
    },
    DistTag {
        target: PackageName,
        tag: DistTag,
    },
    Git(GitSpec),
    Path(PathSpec),
    Tarball(TarballSpec),
    Workspace(WorkspaceSpec),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionRange(String);

impl VersionRange {
    pub fn new(value: impl Into<String>) -> Result<Self, NonEmptyStringError> {
        non_empty(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DistTag(String);

impl DistTag {
    pub fn new(value: impl Into<String>) -> Result<Self, NonEmptyStringError> {
        non_empty(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GitReference(String);

impl GitReference {
    pub fn new(value: impl Into<String>) -> Result<Self, NonEmptyStringError> {
        non_empty(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Subdirectory(String);

impl Subdirectory {
    pub fn new(value: impl Into<String>) -> Result<Self, NonEmptyStringError> {
        non_empty(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GitSpec {
    pub repository: SourceUrl,
    pub reference: Option<GitReference>,
    pub subdirectory: Option<Subdirectory>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathSpec {
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TarballSpec {
    pub url: SourceUrl,
    pub integrity: Option<Integrity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceSpec {
    pub range: Option<VersionRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonEmptyStringError {
    Empty,
}

impl fmt::Display for NonEmptyStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("value cannot be empty"),
        }
    }
}

impl std::error::Error for NonEmptyStringError {}

fn non_empty(value: impl Into<String>) -> Result<String, NonEmptyStringError> {
    let value = value.into();
    if value.is_empty() {
        Err(NonEmptyStringError::Empty)
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_requested_range_unresolved() {
        let alias = match PackageName::new("react") {
            Ok(alias) => alias,
            Err(error) => panic!("unexpected package name error: {error:?}"),
        };
        let range = match VersionRange::new("^19.0.0") {
            Ok(range) => range,
            Err(error) => panic!("unexpected range error: {error:?}"),
        };

        let request = DependencyRequest::new(
            alias.clone(),
            DependencySpec::RegistryRange {
                target: alias,
                range,
            },
            DependencyKind::Production,
        );

        assert!(matches!(
            request.spec,
            DependencySpec::RegistryRange { ref range, .. } if range.as_str() == "^19.0.0"
        ));
    }
}
