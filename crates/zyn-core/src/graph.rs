use std::collections::BTreeMap;

use crate::{DependencyKind, DependencySpec, PackageName, ResolvedPackageId};

/// A resolved package as it appears in the dependency graph.
///
/// Node identity includes peer and platform context because Node package instances can differ
/// even when they share the same resolved package.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageNodeId {
    package: ResolvedPackageId,
    peer_context: PeerContext,
    platform_context: PlatformContext,
}

impl PackageNodeId {
    pub fn new(package: ResolvedPackageId) -> Self {
        Self {
            package,
            peer_context: PeerContext::default(),
            platform_context: PlatformContext::default(),
        }
    }

    pub fn package(&self) -> &ResolvedPackageId {
        &self.package
    }

    pub fn peer_context(&self) -> &PeerContext {
        &self.peer_context
    }

    pub fn platform_context(&self) -> &PlatformContext {
        &self.platform_context
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
    peers: BTreeMap<PackageName, ResolvedPackageId>,
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

    pub fn peers(&self) -> impl Iterator<Item = (&PackageName, &ResolvedPackageId)> {
        self.peers.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeerResolution {
    name: PackageName,
    package: ResolvedPackageId,
}

impl PeerResolution {
    pub fn new(name: PackageName, package: ResolvedPackageId) -> Self {
        Self { name, package }
    }

    pub fn name(&self) -> &PackageName {
        &self.name
    }

    pub fn package(&self) -> &ResolvedPackageId {
        &self.package
    }
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
    use crate::{
        Integrity, NpmSource, PackageContentHash, PackageRevisionId, PackageSource,
        PackageSourceId, PackageVersion, PatchContentHash, PatchId, PatchStack, SourceUrl,
    };

    #[test]
    fn peer_context_is_part_of_node_identity() {
        let foo_with_react_18 = foo_node().with_peer_context(peer_context(vec![peer_resolution(
            "react",
            react_package("18.3.1"),
        )]));
        let foo_with_react_19 = foo_node().with_peer_context(peer_context(vec![peer_resolution(
            "react",
            react_package("19.0.0"),
        )]));

        assert_ne!(foo_with_react_18, foo_with_react_19);
    }

    #[test]
    fn peer_context_identity_is_order_insensitive() {
        let left = peer_context(vec![
            peer_resolution("react", react_package("18.3.1")),
            peer_resolution("scheduler", scheduler_package()),
        ]);
        let right = peer_context(vec![
            peer_resolution("scheduler", scheduler_package()),
            peer_resolution("react", react_package("18.3.1")),
        ]);

        assert_eq!(left, right);
    }

    #[test]
    fn peer_context_rejects_duplicate_names() {
        let context = PeerContext::new(vec![
            peer_resolution("react", react_package("18.3.1")),
            peer_resolution("react", react_package("19.0.0")),
        ]);

        assert!(matches!(context, Err(PeerContextError::DuplicatePeer(_))));
    }

    #[test]
    fn peer_context_distinguishes_patched_providers() {
        let unpatched = foo_node().with_peer_context(peer_context(vec![peer_resolution(
            "react",
            react_package("19.0.0"),
        )]));
        let patched = foo_node().with_peer_context(peer_context(vec![peer_resolution(
            "react",
            patched_react_package("19.0.0"),
        )]));

        assert_ne!(unpatched, patched);
    }

    fn foo_node() -> PackageNodeId {
        package_node("foo", "1.0.0")
    }

    fn react_package(version: &str) -> ResolvedPackageId {
        resolved_package("react", version)
    }

    fn patched_react_package(version: &str) -> ResolvedPackageId {
        ResolvedPackageId::new(
            PackageRevisionId::new(
                package_source("react", version),
                patch_stack(vec![patch_id("sha256-react-patch")]),
            ),
            package_content_hash("sha256-react-patched"),
        )
    }

    fn scheduler_package() -> ResolvedPackageId {
        resolved_package("scheduler", "0.25.0")
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

    fn package_artifact_hash(name: &str, version: &str) -> PackageContentHash {
        package_content_hash(format!("sha256-{name}-{version}"))
    }

    fn package_revision(name: &str, version: &str) -> PackageRevisionId {
        PackageRevisionId::unpatched(package_source(name, version))
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

    fn patch_id(value: &str) -> PatchId {
        PatchId::new(patch_content_hash(value))
    }

    fn patch_stack(patches: Vec<PatchId>) -> PatchStack {
        match PatchStack::new(patches) {
            Ok(stack) => stack,
            Err(error) => panic!("unexpected patch stack error: {error:?}"),
        }
    }

    fn peer_context(peers: Vec<PeerResolution>) -> PeerContext {
        match PeerContext::new(peers) {
            Ok(context) => context,
            Err(error) => panic!("unexpected peer context error: {error:?}"),
        }
    }

    fn peer_resolution(name: &str, package: ResolvedPackageId) -> PeerResolution {
        PeerResolution::new(package_name(name), package)
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

    fn package_content_hash(value: impl Into<String>) -> PackageContentHash {
        match PackageContentHash::new(value) {
            Ok(hash) => hash,
            Err(error) => panic!("unexpected package content hash error: {error:?}"),
        }
    }

    fn patch_content_hash(value: impl Into<String>) -> PatchContentHash {
        match PatchContentHash::new(value) {
            Ok(hash) => hash,
            Err(error) => panic!("unexpected patch content hash error: {error:?}"),
        }
    }
}
