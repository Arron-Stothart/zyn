use crate::{PackageName, PackageVersion, ResolvedSource};

/// A resolved package before peer and platform context are applied.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId {
    name: PackageName,
    version: Option<PackageVersion>,
    source: ResolvedSource,
}

impl PackageId {
    pub fn new(
        name: PackageName,
        version: Option<PackageVersion>,
        source: ResolvedSource,
    ) -> Result<Self, PackageIdError> {
        if matches!(source, ResolvedSource::Npm(_)) && version.is_none() {
            return Err(PackageIdError::MissingVersionForRegistryPackage);
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

    pub fn source(&self) -> &ResolvedSource {
        &self.source
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageIdError {
    MissingVersionForRegistryPackage,
}

impl std::fmt::Display for PackageIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingVersionForRegistryPackage => {
                f.write_str("registry packages must have a resolved version")
            }
        }
    }
}

impl std::error::Error for PackageIdError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Integrity, NpmSource, SourceUrl};

    #[test]
    fn requires_version_for_npm_packages() {
        let package = PackageId::new(package_name("left-pad"), None, npm_source());

        assert_eq!(
            package,
            Err(PackageIdError::MissingVersionForRegistryPackage)
        );
    }

    fn package_name(value: &str) -> PackageName {
        match PackageName::new(value) {
            Ok(name) => name,
            Err(error) => panic!("unexpected package name error: {error:?}"),
        }
    }

    fn npm_source() -> ResolvedSource {
        ResolvedSource::Npm(NpmSource {
            registry: source_url("https://registry.npmjs.org"),
            tarball: source_url("https://registry.npmjs.org/left-pad/-/left-pad-1.3.0.tgz"),
            integrity: integrity("sha512-example"),
        })
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
}
