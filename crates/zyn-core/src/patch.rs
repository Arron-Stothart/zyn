use std::collections::BTreeSet;
use std::fmt;

use crate::{PackageName, PackageSourceId, PatchContentHash};

/// Stable identity for a zyn-tracked patch file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PatchId {
    content_hash: PatchContentHash,
}

impl PatchId {
    pub fn new(content_hash: PatchContentHash) -> Self {
        Self { content_hash }
    }

    pub fn content_hash(&self) -> &PatchContentHash {
        &self.content_hash
    }
}

/// Ordered patches that produce a package revision.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct PatchStack {
    patches: Vec<PatchId>,
}

impl PatchStack {
    pub fn new(patches: Vec<PatchId>) -> Result<Self, PatchStackError> {
        let mut seen = BTreeSet::new();
        for patch in &patches {
            if !seen.insert(patch) {
                return Err(PatchStackError::DuplicatePatch(patch.clone()));
            }
        }

        Ok(Self { patches })
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn patches(&self) -> &[PatchId] {
        &self.patches
    }

    pub fn is_empty(&self) -> bool {
        self.patches.is_empty()
    }

    pub fn len(&self) -> usize {
        self.patches.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchStackError {
    DuplicatePatch(PatchId),
}

impl fmt::Display for PatchStackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicatePatch(patch) => {
                write!(
                    f,
                    "patch stack contains duplicate patch `{}`",
                    patch.content_hash()
                )
            }
        }
    }
}

impl std::error::Error for PatchStackError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchSelectionError {
    EmptyPatchStack,
}

impl fmt::Display for PatchSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPatchStack => f.write_str("patch selection must contain at least one patch"),
        }
    }
}

impl std::error::Error for PatchSelectionError {}

/// Where zyn should apply an ordered patch stack.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatchSelection {
    selector: PatchSelector,
    stack: PatchStack,
}

impl PatchSelection {
    pub fn new(selector: PatchSelector, stack: PatchStack) -> Result<Self, PatchSelectionError> {
        if stack.is_empty() {
            return Err(PatchSelectionError::EmptyPatchStack);
        }

        Ok(Self { selector, stack })
    }

    pub fn selector(&self) -> &PatchSelector {
        &self.selector
    }

    pub fn stack(&self) -> &PatchStack {
        &self.stack
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatchSelector {
    /// Apply a patch stack to every matching package target.
    Global(PatchTarget),
    Edge(EdgePatchSelector),
}

/// Apply a patch stack to a target only when it is a direct dependency of the parent.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgePatchSelector {
    parent: PackageName,
    target: PatchTarget,
}

impl EdgePatchSelector {
    pub fn new(parent: PackageName, target: PatchTarget) -> Self {
        Self { parent, target }
    }

    pub fn parent(&self) -> &PackageName {
        &self.parent
    }

    pub fn target(&self) -> &PatchTarget {
        &self.target
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatchTarget {
    /// Match every resolved source with this package name.
    Package(PackageName),
    Source(PackageSourceId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Integrity, NpmSource, PackageSource, PackageVersion, SourceUrl};

    #[test]
    fn patch_stack_order_is_part_of_identity() {
        let first = patch_id("sha256-first");
        let second = patch_id("sha256-second");

        let left = patch_stack(vec![first.clone(), second.clone()]);
        let right = patch_stack(vec![second, first]);

        assert_ne!(left, right);
    }

    #[test]
    fn patch_stack_rejects_duplicate_patches() {
        let patch = patch_id("sha256-patch");
        let stack = PatchStack::new(vec![patch.clone(), patch.clone()]);

        assert_eq!(stack, Err(PatchStackError::DuplicatePatch(patch)));
    }

    #[test]
    fn patch_selection_rejects_empty_stacks() {
        let selection = PatchSelection::new(
            PatchSelector::Global(PatchTarget::Package(package_name("react"))),
            PatchStack::empty(),
        );

        assert_eq!(selection, Err(PatchSelectionError::EmptyPatchStack));
    }

    fn patch_id(value: &str) -> PatchId {
        PatchId::new(content_hash(value))
    }

    fn patch_stack(patches: Vec<PatchId>) -> PatchStack {
        match PatchStack::new(patches) {
            Ok(stack) => stack,
            Err(error) => panic!("unexpected patch stack error: {error:?}"),
        }
    }

    fn content_hash(value: &str) -> PatchContentHash {
        match PatchContentHash::new(value) {
            Ok(hash) => hash,
            Err(error) => panic!("unexpected content hash error: {error:?}"),
        }
    }

    #[test]
    fn patch_selection_can_target_a_specific_edge() {
        let selection = patch_selection(
            PatchSelector::Edge(EdgePatchSelector::new(
                package_name("design-system"),
                PatchTarget::Source(package_source("react", "19.0.0")),
            )),
            patch_stack(vec![patch_id("sha256-react-patch")]),
        );

        assert!(matches!(selection.selector(), PatchSelector::Edge(_)));
        assert_eq!(selection.stack().len(), 1);
    }

    fn patch_selection(selector: PatchSelector, stack: PatchStack) -> PatchSelection {
        match PatchSelection::new(selector, stack) {
            Ok(selection) => selection,
            Err(error) => panic!("unexpected patch selection error: {error:?}"),
        }
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

    fn npm_source(name: &str, version: &str) -> PackageSource {
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
}
