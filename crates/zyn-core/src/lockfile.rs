use std::collections::HashSet;

use crate::{DependencyEdge, PackageNodeId};

pub const LOCKFILE_VERSION: u32 = 1;
pub const LOCKFILE_REVISION: u32 = 0;

/// The resolved dependency graph represented by `zyn.lock`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lockfile {
    pub version: u32,
    pub revision: u32,
    pub packages: Vec<PackageNodeId>,
    pub edges: Vec<DependencyEdge>,
}

impl Lockfile {
    pub fn new(
        packages: Vec<PackageNodeId>,
        edges: Vec<DependencyEdge>,
    ) -> Result<Self, LockfileError> {
        let mut seen_packages = HashSet::new();
        for package in &packages {
            if !seen_packages.insert(package) {
                return Err(LockfileError::DuplicatePackage);
            }
        }

        let mut seen_edges = HashSet::new();
        for edge in &edges {
            if !seen_edges.insert(edge) {
                return Err(LockfileError::DuplicateEdge);
            }
            if !seen_packages.contains(&edge.from) || !seen_packages.contains(&edge.to) {
                return Err(LockfileError::UnknownEdgeEndpoint);
            }
        }

        Ok(Self {
            version: LOCKFILE_VERSION,
            revision: LOCKFILE_REVISION,
            packages,
            edges,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.packages.is_empty() && self.edges.is_empty()
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self {
            version: LOCKFILE_VERSION,
            revision: LOCKFILE_REVISION,
            packages: Vec::new(),
            edges: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockfileError {
    DuplicatePackage,
    DuplicateEdge,
    UnknownEdgeEndpoint,
}

impl std::fmt::Display for LockfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicatePackage => f.write_str("lockfile contains a duplicate package"),
            Self::DuplicateEdge => f.write_str("lockfile contains a duplicate edge"),
            Self::UnknownEdgeEndpoint => {
                f.write_str("lockfile edge points to a package that is not locked")
            }
        }
    }
}

impl std::error::Error for LockfileError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DependencyKind, DependencySpec, Integrity, NpmSource, PackageContentHash, PackageName,
        PackageRevisionId, PackageSource, PackageSourceId, PackageVersion, ResolvedPackageId,
        SourceUrl, VersionRange,
    };

    #[test]
    fn starts_at_version_one() {
        let lockfile = Lockfile::default();

        assert_eq!(lockfile.version, 1);
        assert_eq!(lockfile.revision, LOCKFILE_REVISION);
        assert!(lockfile.is_empty());
    }

    #[test]
    fn rejects_edges_with_missing_packages() {
        let from = package_node("web-app", "1.0.0");
        let to = package_node("left-pad", "1.3.0");
        let edge = DependencyEdge {
            from: from.clone(),
            to,
            alias: package_name("left-pad"),
            kind: DependencyKind::Production,
            requested: DependencySpec::RegistryRange {
                target: package_name("left-pad"),
                range: version_range("^1.3.0"),
            },
        };

        let lockfile = Lockfile::new(vec![from], vec![edge]);

        assert_eq!(lockfile, Err(LockfileError::UnknownEdgeEndpoint));
    }

    fn package_node(name: &str, version: &str) -> PackageNodeId {
        PackageNodeId::new(resolved_package(name, version))
    }

    fn resolved_package(name: &str, version: &str) -> ResolvedPackageId {
        ResolvedPackageId::new(
            package_revision(name, version),
            package_artifact_hash(name, version),
        )
    }

    fn package_revision(name: &str, version: &str) -> PackageRevisionId {
        PackageRevisionId::unpatched(package_source(name, version))
    }

    fn package_artifact_hash(name: &str, version: &str) -> PackageContentHash {
        package_content_hash(format!("sha256-{name}-{version}"))
    }

    fn package_source(name: &str, version: &str) -> PackageSourceId {
        match PackageSourceId::new(
            package_name(name),
            Some(package_version(version)),
            npm_source(name, version),
        ) {
            Ok(source) => source,
            Err(error) => panic!("unexpected package source id error: {error:?}"),
        }
    }

    fn package_name(value: &str) -> PackageName {
        match PackageName::new(value) {
            Ok(name) => name,
            Err(error) => panic!("unexpected package name error: {error:?}"),
        }
    }

    fn package_version(value: &str) -> PackageVersion {
        match PackageVersion::new(value) {
            Ok(version) => version,
            Err(error) => panic!("unexpected package version error: {error:?}"),
        }
    }

    fn version_range(value: &str) -> VersionRange {
        match VersionRange::new(value) {
            Ok(range) => range,
            Err(error) => panic!("unexpected version range error: {error:?}"),
        }
    }

    fn npm_source(name: &str, version: &str) -> PackageSource {
        PackageSource::Npm(NpmSource {
            registry: source_url("https://registry.npmjs.org"),
            tarball: source_url(format!(
                "https://registry.npmjs.org/{name}/-/{name}-{version}.tgz"
            )),
            integrity: integrity(format!("sha512-{name}-{version}")),
        })
    }

    fn source_url(value: impl Into<String>) -> SourceUrl {
        match SourceUrl::new(value) {
            Ok(url) => url,
            Err(error) => panic!("unexpected source url error: {error:?}"),
        }
    }

    fn integrity(value: impl Into<String>) -> Integrity {
        match Integrity::new(value) {
            Ok(integrity) => integrity,
            Err(error) => panic!("unexpected integrity error: {error:?}"),
        }
    }

    fn package_content_hash(value: impl Into<String>) -> PackageContentHash {
        match PackageContentHash::new(value) {
            Ok(hash) => hash,
            Err(error) => panic!("unexpected content hash error: {error:?}"),
        }
    }
}
