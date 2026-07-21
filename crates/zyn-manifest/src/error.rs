use zyn_core::{NonEmptyStringError, PackageNameError, PackageVersionError, SourceTextError};

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
        section: &'static str,
        alias: String,
        source: PackageNameError,
    },
    #[error("invalid dependency target `{target}` for dependency `{alias}` in {section}")]
    DependencyTargetName {
        section: &'static str,
        alias: String,
        target: String,
        source: PackageNameError,
    },
    #[error("invalid dependency spec `{spec}` for dependency `{alias}` in {section}")]
    DependencySpec {
        section: &'static str,
        alias: String,
        spec: String,
        source: NonEmptyStringError,
    },
    #[error("invalid dependency source `{spec}` for dependency `{alias}` in {section}")]
    DependencySource {
        section: &'static str,
        alias: String,
        spec: String,
        source: SourceTextError,
    },
    #[error("unsupported dependency spec `{spec}` for dependency `{alias}` in {section}")]
    UnsupportedDependencySpec {
        section: &'static str,
        alias: String,
        spec: String,
    },
}
