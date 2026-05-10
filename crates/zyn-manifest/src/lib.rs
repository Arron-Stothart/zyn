//! Read package manifests into zyn's core dependency model.

mod error;
mod manifest;
mod package_json;
mod spec;

pub use error::ManifestError;
pub use manifest::{DependencySection, ManifestDependency, PackageManifest};
