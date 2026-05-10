mod dependency;
mod graph;
mod lockfile;
mod name;
mod package;
mod source;
mod version;

pub use dependency::{
    DependencyKind, DependencyRequest, DependencySpec, DistTag, GitReference, GitSpec,
    NonEmptyStringError, PathSpec, Subdirectory, TarballSpec, VersionRange, WorkspaceSpec,
};
pub use graph::{
    DependencyEdge, PackageNodeId, PeerContext, PeerContextError, PeerResolution, PlatformContext,
};
pub use lockfile::{LOCKFILE_REVISION, LOCKFILE_VERSION, Lockfile, LockfileError};
pub use name::{PackageName, PackageNameError};
pub use package::{PackageId, PackageIdError};
pub use source::{
    ContentHash, GitCommit, GitSource, Integrity, NpmSource, PathSource, ResolvedSource,
    SourceTextError, SourceUrl, TarballSource, VendoredSource, WorkspaceSource,
};
pub use version::{PackageVersion, PackageVersionError};

pub const NAME: &str = "zyn";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_package_identity() {
        assert_eq!(NAME, "zyn");
        assert!(!VERSION.is_empty());
    }
}
