use zyn_core::{NonEmptyStringError, PackageNameError, PackageVersionError, SourceTextError};

use crate::manifest::DependencySection;

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("failed to parse package.json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid package name `{value}`")]
    PackageName {
        value: String,
        source: PackageNameError,
    },
    #[error("invalid package version `{value}`")]
    PackageVersion {
        value: String,
        source: PackageVersionError,
    },
    #[error("invalid dependency `{alias}` in {section}")]
    DependencyName {
        section: DependencySection,
        alias: String,
        source: PackageNameError,
    },
    #[error("invalid dependency target `{target}` for dependency `{alias}` in {section}")]
    DependencyTargetName {
        section: DependencySection,
        alias: String,
        target: String,
        source: PackageNameError,
    },
    #[error("invalid dependency spec `{spec}` for dependency `{alias}` in {section}")]
    DependencySpec {
        section: DependencySection,
        alias: String,
        spec: String,
        source: NonEmptyStringError,
    },
    #[error("invalid dependency source `{spec}` for dependency `{alias}` in {section}")]
    DependencySource {
        section: DependencySection,
        alias: String,
        spec: String,
        source: SourceTextError,
    },
    #[error("unsupported dependency spec `{spec}` for dependency `{alias}` in {section}")]
    UnsupportedDependencySpec {
        section: DependencySection,
        alias: String,
        spec: String,
    },
}
