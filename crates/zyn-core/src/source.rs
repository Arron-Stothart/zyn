use std::borrow::Borrow;
use std::fmt;
use std::path::PathBuf;

use crate::{GitReference, Subdirectory};

/// The resolved source for a package before applying zyn-managed patches.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageSource {
    Npm(NpmSource),
    Git(GitSource),
    Path(PathSource),
    Tarball(TarballSource),
    Workspace(WorkspaceSource),
    Vendored(VendoredSource),
}

impl PackageSource {
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
    pub content_hash: PackageContentHash,
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
pub(crate) struct ContentHash(String);

impl ContentHash {
    pub fn new(value: impl Into<String>) -> Result<Self, SourceTextError> {
        non_empty(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageContentHash(ContentHash);

impl PackageContentHash {
    pub fn new(value: impl Into<String>) -> Result<Self, SourceTextError> {
        ContentHash::new(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for PackageContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for PackageContentHash {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for PackageContentHash {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PatchContentHash(ContentHash);

impl PatchContentHash {
    pub fn new(value: impl Into<String>) -> Result<Self, SourceTextError> {
        ContentHash::new(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for PatchContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for PatchContentHash {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for PatchContentHash {
    fn borrow(&self) -> &str {
        self.as_str()
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
    use crate::{PackageName, PackageSourceId};

    #[test]
    fn distinguishes_editable_sources_from_registry_sources() {
        let registry = PackageSource::Npm(NpmSource {
            registry: source_url("https://registry.npmjs.org"),
            tarball: source_url("https://registry.npmjs.org/is-even/-/is-even-1.0.0.tgz"),
            integrity: integrity("sha512-example"),
        });
        let vendored = PackageSource::Vendored(VendoredSource {
            path: ".zyn/deps/is-even@1.0.0".into(),
            content_hash: package_content_hash("sha256-example"),
        });

        assert!(!registry.is_local());
        assert!(vendored.is_local());
    }

    #[test]
    fn package_source_can_use_non_npm_sources() {
        let name = match PackageName::new("toolkit") {
            Ok(name) => name,
            Err(error) => panic!("unexpected package name error: {error:?}"),
        };

        let package = package_source(
            name,
            None,
            PackageSource::Path(PathSource {
                path: "../toolkit".into(),
            }),
        );

        assert!(package.version().is_none());
        assert!(package.source().is_local());
    }

    fn package_source(
        name: PackageName,
        version: Option<crate::PackageVersion>,
        source: PackageSource,
    ) -> PackageSourceId {
        match PackageSourceId::new(name, version, source) {
            Ok(package) => package,
            Err(error) => panic!("unexpected package source id error: {error:?}"),
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

    fn package_content_hash(value: &str) -> PackageContentHash {
        match PackageContentHash::new(value) {
            Ok(hash) => hash,
            Err(error) => panic!("unexpected content hash error: {error:?}"),
        }
    }
}
