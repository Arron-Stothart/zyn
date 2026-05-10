use std::collections::BTreeMap;

use crate::{DependencyKind, DependencySpec, PackageId, PackageName};

/// A resolved package as it appears in the dependency graph.
///
/// Node identity includes peer and platform context because Node package instances can differ
/// even when they share the same package name, version, and source.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageNodeId {
    pub package: PackageId,
    pub peer_context: PeerContext,
    pub platform_context: PlatformContext,
}

impl PackageNodeId {
    pub fn new(package: PackageId) -> Self {
        Self {
            package,
            peer_context: PeerContext::default(),
            platform_context: PlatformContext::default(),
        }
    }

    pub fn with_peer_context(mut self, peer_context: PeerContext) -> Self {
        self.peer_context = peer_context;
        self
    }

    pub fn with_platform_context(mut self, platform_context: PlatformContext) -> Self {
        self.platform_context = platform_context;
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct PeerContext {
    peers: BTreeMap<PackageName, PackageId>,
}

impl PeerContext {
    pub fn new(peers: Vec<PeerResolution>) -> Result<Self, PeerContextError> {
        let mut resolved = BTreeMap::new();
        for peer in peers {
            if resolved.insert(peer.name.clone(), peer.package).is_some() {
                return Err(PeerContextError::DuplicatePeer(peer.name));
            }
        }

        Ok(Self { peers: resolved })
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn peers(&self) -> impl Iterator<Item = (&PackageName, &PackageId)> {
        self.peers.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeerResolution {
    pub name: PackageName,
    pub package: PackageId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerContextError {
    DuplicatePeer(PackageName),
}

impl std::fmt::Display for PeerContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicatePeer(name) => {
                write!(f, "peer context contains duplicate peer `{name}`")
            }
        }
    }
}

impl std::error::Error for PeerContextError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct PlatformContext {
    pub os: Option<String>,
    pub arch: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyEdge {
    pub from: PackageNodeId,
    pub to: PackageNodeId,
    pub alias: PackageName,
    pub kind: DependencyKind,
    pub requested: DependencySpec,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Integrity, NpmSource, PackageVersion, ResolvedSource, SourceUrl};

    #[test]
    fn peer_context_is_part_of_node_identity() {
        let foo_with_react_18 = foo_node().with_peer_context(peer_context(vec![PeerResolution {
            name: package_name("react"),
            package: react_package("18.3.1"),
        }]));
        let foo_with_react_19 = foo_node().with_peer_context(peer_context(vec![PeerResolution {
            name: package_name("react"),
            package: react_package("19.0.0"),
        }]));

        assert_ne!(foo_with_react_18, foo_with_react_19);
    }

    #[test]
    fn peer_context_identity_is_order_insensitive() {
        let left = peer_context(vec![
            PeerResolution {
                name: package_name("react"),
                package: react_package("18.3.1"),
            },
            PeerResolution {
                name: package_name("scheduler"),
                package: scheduler_package(),
            },
        ]);
        let right = peer_context(vec![
            PeerResolution {
                name: package_name("scheduler"),
                package: scheduler_package(),
            },
            PeerResolution {
                name: package_name("react"),
                package: react_package("18.3.1"),
            },
        ]);

        assert_eq!(left, right);
    }

    #[test]
    fn peer_context_rejects_duplicate_names() {
        let context = PeerContext::new(vec![
            PeerResolution {
                name: package_name("react"),
                package: react_package("18.3.1"),
            },
            PeerResolution {
                name: package_name("react"),
                package: react_package("19.0.0"),
            },
        ]);

        assert!(matches!(context, Err(PeerContextError::DuplicatePeer(_))));
    }

    fn foo_node() -> PackageNodeId {
        PackageNodeId::new(package_id(
            package_name("foo"),
            Some(package_version("1.0.0")),
            npm_source("foo", "1.0.0"),
        ))
    }

    fn react_package(version: &str) -> PackageId {
        package_id(
            package_name("react"),
            Some(package_version(version)),
            npm_source("react", version),
        )
    }

    fn scheduler_package() -> PackageId {
        package_id(
            package_name("scheduler"),
            Some(package_version("0.25.0")),
            npm_source("scheduler", "0.25.0"),
        )
    }

    fn package_id(
        name: PackageName,
        version: Option<PackageVersion>,
        source: ResolvedSource,
    ) -> PackageId {
        match PackageId::new(name, version, source) {
            Ok(package) => package,
            Err(error) => panic!("unexpected package id error: {error:?}"),
        }
    }

    fn peer_context(peers: Vec<PeerResolution>) -> PeerContext {
        match PeerContext::new(peers) {
            Ok(context) => context,
            Err(error) => panic!("unexpected peer context error: {error:?}"),
        }
    }

    fn npm_source(name: &str, version: &str) -> ResolvedSource {
        ResolvedSource::Npm(NpmSource {
            registry: source_url("https://registry.npmjs.org"),
            tarball: source_url(format!(
                "https://registry.npmjs.org/{name}/-/{name}-{version}.tgz"
            )),
            integrity: integrity(format!("sha512-{name}-{version}")),
        })
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
}
