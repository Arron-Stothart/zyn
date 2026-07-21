use zyn_core::{DependencyKind, DependencyRequest, PackageName, PackageVersion};

use crate::error::ManifestError;
use crate::package_json::PackageJson;
use crate::spec::parse_dependency_spec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    name: Option<PackageName>,
    version: Option<PackageVersion>,
    dependencies: Vec<DependencyRequest>,
}

impl PackageManifest {
    pub fn from_package_json_str(source: &str) -> Result<Self, ManifestError> {
        let package_json = PackageJson::from_str(source)?;
        Self::from_package_json(package_json)
    }

    pub fn name(&self) -> Option<&PackageName> {
        self.name.as_ref()
    }

    pub fn version(&self) -> Option<&PackageVersion> {
        self.version.as_ref()
    }

    pub fn dependencies(&self) -> &[DependencyRequest] {
        &self.dependencies
    }

    fn from_package_json(package_json: PackageJson) -> Result<Self, ManifestError> {
        let name = package_json
            .name
            .map(|value| {
                PackageName::new(value.clone())
                    .map_err(|source| ManifestError::PackageName { value, source })
            })
            .transpose()?;
        let version = package_json
            .version
            .map(|value| {
                PackageVersion::new(value.clone())
                    .map_err(|source| ManifestError::PackageVersion { value, source })
            })
            .transpose()?;

        let mut dependencies = Vec::new();
        add_dependency_section(
            &mut dependencies,
            package_json.dependencies,
            DependencySection::Dependencies,
        )?;
        add_dependency_section(
            &mut dependencies,
            package_json.dev_dependencies,
            DependencySection::DevDependencies,
        )?;
        add_dependency_section(
            &mut dependencies,
            package_json.peer_dependencies,
            DependencySection::PeerDependencies,
        )?;
        add_dependency_section(
            &mut dependencies,
            package_json.optional_dependencies,
            DependencySection::OptionalDependencies,
        )?;

        Ok(Self {
            name,
            version,
            dependencies,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DependencySection {
    Dependencies,
    DevDependencies,
    OptionalDependencies,
    PeerDependencies,
}

impl DependencySection {
    fn kind(self) -> DependencyKind {
        match self {
            Self::Dependencies => DependencyKind::Production,
            Self::DevDependencies => DependencyKind::Development,
            Self::OptionalDependencies => DependencyKind::Optional,
            Self::PeerDependencies => DependencyKind::Peer,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Dependencies => "dependencies",
            Self::DevDependencies => "devDependencies",
            Self::OptionalDependencies => "optionalDependencies",
            Self::PeerDependencies => "peerDependencies",
        }
    }
}

fn add_dependency_section(
    dependencies: &mut Vec<DependencyRequest>,
    section_dependencies: impl IntoIterator<Item = (String, String)>,
    section: DependencySection,
) -> Result<(), ManifestError> {
    for (alias, raw_spec) in section_dependencies {
        let package_name =
            PackageName::new(alias.clone()).map_err(|source| ManifestError::DependencyName {
                section: section.as_str(),
                alias: alias.clone(),
                source,
            })?;
        let spec = parse_dependency_spec(section, &package_name, &raw_spec)?;
        let request = DependencyRequest::new(package_name.clone(), spec, section.kind());

        if section == DependencySection::OptionalDependencies {
            dependencies.retain(|existing| {
                existing.alias != package_name || existing.kind != DependencyKind::Production
            });
        }

        dependencies.push(request);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zyn_core::DependencySpec;

    #[test]
    fn reads_package_identity() {
        let manifest = parse(
            r#"{
              "name": "web-app",
              "version": "1.0.0"
            }"#,
        );

        assert!(matches!(manifest.name(), Some(name) if name.as_str() == "web-app"));
        assert!(matches!(manifest.version(), Some(version) if version.as_str() == "1.0.0"));
    }

    #[test]
    fn reads_dependency_sections() {
        let manifest = parse(
            r#"{
              "dependencies": {
                "@scope/ui": "^1.0.0"
              },
              "devDependencies": {
                "typescript": "^5.8.0"
              },
              "optionalDependencies": {
                "zod": "^3.23.8"
              },
              "peerDependencies": {
                "react": "^19.0.0"
              }
            }"#,
        );

        assert_eq!(manifest.dependencies().len(), 4);
        assert_registry_range(
            manifest.dependencies(),
            "@scope/ui",
            "@scope/ui",
            "^1.0.0",
            DependencySection::Dependencies,
        );
        assert_registry_range(
            manifest.dependencies(),
            "typescript",
            "typescript",
            "^5.8.0",
            DependencySection::DevDependencies,
        );
        assert_registry_range(
            manifest.dependencies(),
            "zod",
            "zod",
            "^3.23.8",
            DependencySection::OptionalDependencies,
        );
        assert_registry_range(
            manifest.dependencies(),
            "react",
            "react",
            "^19.0.0",
            DependencySection::PeerDependencies,
        );
    }

    #[test]
    fn optional_dependencies_override_production_dependencies() {
        let manifest = parse(
            r#"{
              "dependencies": {
                "zod": "^3.0.0"
              },
              "optionalDependencies": {
                "zod": "^3.23.8"
              }
            }"#,
        );

        assert_eq!(manifest.dependencies().len(), 1);
        assert_registry_range(
            manifest.dependencies(),
            "zod",
            "zod",
            "^3.23.8",
            DependencySection::OptionalDependencies,
        );
    }

    #[test]
    fn peer_and_dev_dependencies_can_share_an_alias() {
        let manifest = parse(
            r#"{
              "devDependencies": {
                "react": "^19.0.0"
              },
              "peerDependencies": {
                "react": "^18.0.0 || ^19.0.0"
              }
            }"#,
        );

        assert_eq!(manifest.dependencies().len(), 2);
        assert_eq!(
            manifest
                .dependencies()
                .iter()
                .filter(|dependency| dependency.alias.as_str() == "react")
                .count(),
            2
        );
    }

    fn parse(source: &str) -> PackageManifest {
        match PackageManifest::from_package_json_str(source) {
            Ok(manifest) => manifest,
            Err(error) => panic!("unexpected manifest error: {error:?}"),
        }
    }

    fn assert_registry_range(
        dependencies: &[DependencyRequest],
        alias: &str,
        target: &str,
        range: &str,
        section: DependencySection,
    ) {
        let dependency = dependencies
            .iter()
            .find(|dependency| dependency.alias.as_str() == alias)
            .unwrap_or_else(|| panic!("missing dependency `{alias}`"));

        assert_eq!(dependency.kind, section.kind());
        assert!(matches!(
            dependency.spec,
            DependencySpec::RegistryRange { target: ref actual_target, range: ref actual_range }
                if actual_target.as_str() == target && actual_range.as_str() == range
        ));
    }
}
