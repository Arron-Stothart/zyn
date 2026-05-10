use std::fmt;
use std::path::PathBuf;

use crate::{GitReference, Subdirectory};

/// The resolved source zyn uses for a package.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvedSource {
    Npm(NpmSource),
    Git(GitSource),
    Path(PathSource),
    Tarball(TarballSource),
    Workspace(WorkspaceSource),
    Vendored(VendoredSource),
}

impl ResolvedSource {
    pub fn is_local(&self) -> bool {
        matches!(self, Self::Path(_) | Self::Workspace(_) | Self::Vendored(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NpmSource {
    pub registry: SourceUrl,
    pub tarball: SourceUrl,
    pub integrity: Integrity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GitSource {
    pub repository: SourceUrl,
    pub commit: GitCommit,
    pub reference: Option<GitReference>,
    pub subdirectory: Option<Subdirectory>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathSource {
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TarballSource {
    pub url: SourceUrl,
    pub integrity: Integrity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceSource {
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VendoredSource {
    pub path: PathBuf,
    pub content_hash: ContentHash,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceUrl(String);

impl SourceUrl {
    pub fn new(value: impl Into<String>) -> Result<Self, SourceTextError> {
        non_empty(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SourceUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Integrity(String);

impl Integrity {
    pub fn new(value: impl Into<String>) -> Result<Self, SourceTextError> {
        non_empty(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentHash(String);

impl ContentHash {
    pub fn new(value: impl Into<String>) -> Result<Self, SourceTextError> {
        non_empty(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GitCommit(String);

impl GitCommit {
    pub fn new(value: impl Into<String>) -> Result<Self, SourceTextError> {
        non_empty(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTextError {
    Empty,
}

impl fmt::Display for SourceTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("source text cannot be empty"),
        }
    }
}

impl std::error::Error for SourceTextError {}

fn non_empty(value: impl Into<String>) -> Result<String, SourceTextError> {
    let value = value.into();
    if value.is_empty() {
        Err(SourceTextError::Empty)
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PackageId, PackageName};

    #[test]
    fn distinguishes_editable_sources_from_registry_sources() {
        let registry = ResolvedSource::Npm(NpmSource {
            registry: source_url("https://registry.npmjs.org"),
            tarball: source_url("https://registry.npmjs.org/is-even/-/is-even-1.0.0.tgz"),
            integrity: integrity("sha512-example"),
        });
        let vendored = ResolvedSource::Vendored(VendoredSource {
            path: ".zyn/deps/is-even@1.0.0".into(),
            content_hash: content_hash("sha256-example"),
        });

        assert!(!registry.is_local());
        assert!(vendored.is_local());
    }

    #[test]
    fn package_id_can_use_non_npm_sources() {
        let name = match PackageName::new("toolkit") {
            Ok(name) => name,
            Err(error) => panic!("unexpected package name error: {error:?}"),
        };

        let package = package_id(
            name,
            None,
            ResolvedSource::Path(PathSource {
                path: "../toolkit".into(),
            }),
        );

        assert!(package.version().is_none());
        assert!(package.source().is_local());
    }

    fn package_id(
        name: PackageName,
        version: Option<crate::PackageVersion>,
        source: ResolvedSource,
    ) -> PackageId {
        match PackageId::new(name, version, source) {
            Ok(package) => package,
            Err(error) => panic!("unexpected package id error: {error:?}"),
        }
    }

    fn source_url(value: &str) -> SourceUrl {
        match SourceUrl::new(value) {
            Ok(url) => url,
            Err(error) => panic!("unexpected source url error: {error:?}"),
        }
    }

    fn integrity(value: &str) -> Integrity {
        match Integrity::new(value) {
            Ok(integrity) => integrity,
            Err(error) => panic!("unexpected integrity error: {error:?}"),
        }
    }

    fn content_hash(value: &str) -> ContentHash {
        match ContentHash::new(value) {
            Ok(hash) => hash,
            Err(error) => panic!("unexpected content hash error: {error:?}"),
        }
    }
}
