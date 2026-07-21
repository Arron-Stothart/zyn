//! Parse package.json files into zyn's dependency model.

mod error;
mod manifest;
mod package_json;
mod spec;

pub use error::ManifestError;
pub use manifest::PackageManifest;
