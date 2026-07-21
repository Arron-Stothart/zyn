use crate::{PackageContentHash, PackageName, PackageSource, PackageVersion, PatchStack};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageSourceId {
    name: PackageName,
    version: Option<PackageVersion>,
    source: PackageSource,
}

impl PackageSourceId {
    pub fn new(
        name: PackageName,
        version: Option<PackageVersion>,
        source: PackageSource,
    ) -> Result<Self, PackageSourceIdError> {
        if matches!(source, PackageSource::Npm(_)) && version.is_none() {
            return Err(PackageSourceIdError::MissingVersionForRegistryPackage);
        }

        Ok(Self {
            name,
            version,
            source,
        })
    }

    pub fn name(&self) -> &PackageName {
        &self.name
    }

    pub fn version(&self) -> Option<&PackageVersion> {
        self.version.as_ref()
    }

    pub fn source(&self) -> &PackageSource {
        &self.source
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageSourceIdError {
    MissingVersionForRegistryPackage,
}

impl std::fmt::Display for PackageSourceIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingVersionForRegistryPackage => {
                f.write_str("registry packages must have a resolved version")
            }
        }
    }
}

impl std::error::Error for PackageSourceIdError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageRevisionId {
    source: PackageSourceId,
    patch_stack: PatchStack,
}

impl PackageRevisionId {
    pub fn new(source: PackageSourceId, patch_stack: PatchStack) -> Self {
        Self {
            source,
            patch_stack,
        }
    }

    pub fn unpatched(source: PackageSourceId) -> Self {
        Self::new(source, PatchStack::empty())
    }

    pub fn source(&self) -> &PackageSourceId {
        &self.source
    }

    pub fn patch_stack(&self) -> &PatchStack {
        &self.patch_stack
    }
}

/// A package revision paired with the artifact hash zyn recorded for it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedPackageId {
    revision: PackageRevisionId,
    artifact_hash: PackageContentHash,
}

impl ResolvedPackageId {
    pub fn new(revision: PackageRevisionId, artifact_hash: PackageContentHash) -> Self {
        Self {
            revision,
            artifact_hash,
        }
    }

    pub fn revision(&self) -> &PackageRevisionId {
        &self.revision
    }

    pub fn artifact_hash(&self) -> &PackageContentHash {
        &self.artifact_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Integrity, NpmSource, PatchContentHash, PatchId, SourceUrl};

    #[test]
    fn requires_version_for_npm_packages() {
        let package = PackageSourceId::new(package_name("left-pad"), None, npm_source());

        assert_eq!(
            package,
            Err(PackageSourceIdError::MissingVersionForRegistryPackage)
        );
    }

    #[test]
    fn patch_stack_is_part_of_revision_identity() {
        let source = package_source("react", "19.0.0");
        let unpatched = PackageRevisionId::unpatched(source.clone());
        let patched = PackageRevisionId::new(source, patch_stack(vec![patch_id("sha256-patch")]));

        assert_ne!(unpatched, patched);
    }

    fn package_source(name: &str, version: &str) -> PackageSourceId {
        match PackageSourceId::new(
            package_name(name),
            Some(package_version(version)),
            npm_source_for(name, version),
        ) {
            Ok(package) => package,
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

    fn npm_source() -> PackageSource {
        npm_source_for("left-pad", "1.3.0")
    }

    fn npm_source_for(name: &str, version: &str) -> PackageSource {
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

    fn patch_id(value: &str) -> PatchId {
        PatchId::new(patch_content_hash(value))
    }

    fn patch_stack(patches: Vec<PatchId>) -> PatchStack {
        match PatchStack::new(patches) {
            Ok(stack) => stack,
            Err(error) => panic!("unexpected patch stack error: {error:?}"),
        }
    }

    fn patch_content_hash(value: &str) -> PatchContentHash {
        match PatchContentHash::new(value) {
            Ok(hash) => hash,
            Err(error) => panic!("unexpected content hash error: {error:?}"),
        }
    }
}
