use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};
use std::fmt;
use std::path::Path;

use crate::{PackageName, PackageSourceId, PatchContentHash, ResolvedPackageId};

pub const PATCHES_DIR: &str = "patches";

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

impl fmt::Display for PatchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.content_hash.fmt(f)
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

/// A resolved patch definition and the package revision it was authored against.
///
/// Constructing this value records the caller's claim that `id` is the content hash of `path`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatchDefinition {
    id: PatchId,
    path: PatchPath,
    base: ResolvedPackageId,
    effects: PatchEffects,
}

impl PatchDefinition {
    pub fn new(
        id: PatchId,
        path: PatchPath,
        base: ResolvedPackageId,
        effects: PatchEffects,
    ) -> Self {
        Self {
            id,
            path,
            base,
            effects,
        }
    }

    pub fn id(&self) -> &PatchId {
        &self.id
    }

    pub fn path(&self) -> &PatchPath {
        &self.path
    }

    pub fn base(&self) -> &ResolvedPackageId {
        &self.base
    }

    pub fn effects(&self) -> PatchEffects {
        self.effects
    }
}

/// Project-relative path to a patch file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PatchPath(String);

impl PatchPath {
    pub fn new(path: impl Into<String>) -> Result<Self, PatchPathError> {
        let path = path.into();
        if path.is_empty() {
            return Err(PatchPathError::Empty);
        }
        if path.starts_with('/') || path.contains('\\') {
            return Err(PatchPathError::Absolute);
        }

        let mut segments = path.split('/');
        match segments.next() {
            Some(PATCHES_DIR) => {}
            _ => return Err(PatchPathError::OutsidePatchesDirectory),
        }

        let mut has_file = false;
        let mut segments = segments.peekable();
        while let Some(segment) = segments.next() {
            match segment {
                "" if segments.peek().is_none() => return Err(PatchPathError::MissingFilename),
                "" => return Err(PatchPathError::EmptySegment),
                "." => return Err(PatchPathError::CurrentDirectory),
                ".." => return Err(PatchPathError::ParentDirectory),
                _ => has_file = true,
            }
        }

        if !has_file {
            return Err(PatchPathError::MissingFilename);
        }
        if !path.ends_with(".patch") {
            return Err(PatchPathError::MissingPatchExtension);
        }

        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PatchPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for PatchPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Path> for PatchPath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchPathError {
    Empty,
    Absolute,
    EmptySegment,
    CurrentDirectory,
    ParentDirectory,
    OutsidePatchesDirectory,
    MissingFilename,
    MissingPatchExtension,
}

impl fmt::Display for PatchPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("patch path cannot be empty"),
            Self::Absolute => f.write_str("patch path must be relative"),
            Self::EmptySegment => f.write_str("patch path cannot contain empty segments"),
            Self::CurrentDirectory => f.write_str("patch path cannot contain `.` components"),
            Self::ParentDirectory => f.write_str("patch path cannot contain `..` components"),
            Self::OutsidePatchesDirectory => {
                f.write_str("patch path must be inside the `patches` directory")
            }
            Self::MissingFilename => f.write_str("patch path must include a filename"),
            Self::MissingPatchExtension => f.write_str("patch path must end with `.patch`"),
        }
    }
}

impl std::error::Error for PatchPathError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PatchEffects {
    manifest: ManifestPatchEffect,
    source: SourcePatchEffect,
}

impl PatchEffects {
    pub fn new(
        manifest: ManifestPatchEffect,
        source: SourcePatchEffect,
    ) -> Result<Self, PatchEffectsError> {
        let effects = Self { manifest, source };
        if effects.is_empty() {
            Err(PatchEffectsError::Empty)
        } else {
            Ok(effects)
        }
    }

    pub fn source_only() -> Self {
        Self {
            manifest: ManifestPatchEffect::None,
            source: SourcePatchEffect::Changes,
        }
    }

    pub fn manifest_resolution() -> Self {
        Self {
            manifest: ManifestPatchEffect::Resolution,
            source: SourcePatchEffect::None,
        }
    }

    pub fn manifest(&self) -> ManifestPatchEffect {
        self.manifest
    }

    pub fn source(&self) -> SourcePatchEffect {
        self.source
    }

    fn is_empty(&self) -> bool {
        self.manifest == ManifestPatchEffect::None && self.source == SourcePatchEffect::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchEffectsError {
    Empty,
}

impl fmt::Display for PatchEffectsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("patch effects must describe at least one effect"),
        }
    }
}

impl std::error::Error for PatchEffectsError {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ManifestPatchEffect {
    /// The patch does not change package manifest data.
    #[default]
    None,
    /// The patch changes manifest data that does not affect dependency graph expansion.
    MetadataOnly,
    /// The patch may change dependencies, peers, optional dependencies, or platform constraints.
    Resolution,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SourcePatchEffect {
    /// The patch does not change package source files.
    #[default]
    None,
    Changes,
}

/// Patch definitions plus the graph policy that selects them.
///
/// Definitions without selections are allowed so users can commit a patch before enabling it.
/// Catalog construction checks that selections reference known patches with matching targets.
/// The same patch may appear in multiple selections, but a resolver must not apply the same patch
/// twice to a single package revision. Stack composition is validated during resolution because it
/// depends on the resolved package candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchCatalog {
    definitions: BTreeMap<PatchId, PatchDefinition>,
    selections: Vec<PatchSelection>,
}

impl PatchCatalog {
    pub fn new(
        definitions: Vec<PatchDefinition>,
        selections: Vec<PatchSelection>,
    ) -> Result<Self, PatchCatalogError> {
        let mut indexed = BTreeMap::new();
        for definition in definitions {
            let id = definition.id().clone();
            match indexed.entry(id) {
                Entry::Vacant(entry) => {
                    entry.insert(definition);
                }
                Entry::Occupied(entry) => {
                    return Err(PatchCatalogError::DuplicateDefinition(entry.key().clone()));
                }
            }
        }

        for selection in &selections {
            for patch in selection.stack().patches() {
                let Some(definition) = indexed.get(patch) else {
                    return Err(PatchCatalogError::UnknownSelectedPatch(patch.clone()));
                };
                if !selection
                    .selector()
                    .target()
                    .matches_source(definition.base().revision().source())
                {
                    return Err(PatchCatalogError::SelectedPatchTargetMismatch(
                        patch.clone(),
                    ));
                }
            }
        }

        Ok(Self {
            definitions: indexed,
            selections,
        })
    }

    pub fn empty() -> Self {
        Self {
            definitions: BTreeMap::new(),
            selections: Vec::new(),
        }
    }

    pub fn definitions(&self) -> impl Iterator<Item = &PatchDefinition> {
        self.definitions.values()
    }

    pub fn get(&self, id: &PatchId) -> Option<&PatchDefinition> {
        self.definitions.get(id)
    }

    pub fn selections(&self) -> &[PatchSelection] {
        &self.selections
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchCatalogError {
    DuplicateDefinition(PatchId),
    UnknownSelectedPatch(PatchId),
    SelectedPatchTargetMismatch(PatchId),
}

impl fmt::Display for PatchCatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateDefinition(patch) => {
                write!(f, "patch catalog contains duplicate patch `{patch}`")
            }
            Self::UnknownSelectedPatch(patch) => {
                write!(f, "patch selection references unknown patch `{patch}`")
            }
            Self::SelectedPatchTargetMismatch(patch) => {
                write!(
                    f,
                    "patch `{patch}` targets a different package than its selection"
                )
            }
        }
    }
}

impl std::error::Error for PatchCatalogError {}

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
    ///
    /// Patch definitions still carry exact bases. Resolution must only apply a selected patch when
    /// the candidate package matches the patch definition's base.
    Global(PatchTarget),
    Edge(EdgePatchSelector),
}

impl PatchSelector {
    pub fn target(&self) -> &PatchTarget {
        match self {
            Self::Global(target) => target,
            Self::Edge(edge) => edge.target(),
        }
    }
}

/// Apply a patch stack to a target only when it is a direct dependency of the parent.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgePatchSelector {
    parent: PatchTarget,
    target: PatchTarget,
}

impl EdgePatchSelector {
    pub fn new(parent: PatchTarget, target: PatchTarget) -> Self {
        Self { parent, target }
    }

    pub fn parent(&self) -> &PatchTarget {
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
    /// Match an exact package source by its identity.
    Source(PackageSourceId),
}

impl PatchTarget {
    pub fn matches_source(&self, source: &PackageSourceId) -> bool {
        match self {
            Self::Package(name) => source.name() == name,
            Self::Source(target_source) => target_source == source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Integrity, NpmSource, PackageContentHash, PackageRevisionId, PackageSource, PackageVersion,
        SourceUrl,
    };

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

    #[test]
    fn patch_path_must_stay_inside_patches_directory() {
        assert_eq!(
            PatchPath::new(format!("{PATCHES_DIR}/react.patch"))
                .map(|path| path.as_path().to_path_buf()),
            Ok(format!("{PATCHES_DIR}/react.patch").into())
        );
        assert_eq!(
            PatchPath::new("react.patch"),
            Err(PatchPathError::OutsidePatchesDirectory)
        );
        assert_eq!(
            PatchPath::new("patches/../react.patch"),
            Err(PatchPathError::ParentDirectory)
        );
        assert_eq!(
            PatchPath::new("patches//react.patch"),
            Err(PatchPathError::EmptySegment)
        );
        assert_eq!(
            PatchPath::new("patches/./react.patch"),
            Err(PatchPathError::CurrentDirectory)
        );
        assert_eq!(
            PatchPath::new("/tmp/react.patch"),
            Err(PatchPathError::Absolute)
        );
        assert_eq!(
            PatchPath::new("patches\\react.patch"),
            Err(PatchPathError::Absolute)
        );
        assert_eq!(
            PatchPath::new("patches/react.diff"),
            Err(PatchPathError::MissingPatchExtension)
        );
        assert_eq!(
            PatchPath::new("patches/"),
            Err(PatchPathError::MissingFilename)
        );
    }

    #[test]
    fn patch_path_exposes_manifest_safe_text() {
        let path = patch_path("patches/react.patch");

        assert_eq!(path.as_str(), "patches/react.patch");
        assert_eq!(path.as_path(), Path::new("patches/react.patch"));
    }

    #[test]
    fn patch_definition_records_base_and_effects() {
        let definition = patch_definition(
            "sha256-react-patch",
            "patches/react.patch",
            resolved_package("react", "19.0.0"),
            PatchEffects::manifest_resolution(),
        );

        assert_eq!(definition.id(), &patch_id("sha256-react-patch"));
        assert_eq!(definition.base(), &resolved_package("react", "19.0.0"));
        assert_eq!(
            definition.effects().manifest(),
            ManifestPatchEffect::Resolution
        );
        assert_eq!(definition.effects().source(), SourcePatchEffect::None);
    }

    #[test]
    fn patch_effects_reject_empty_effects() {
        let effects = PatchEffects::new(ManifestPatchEffect::None, SourcePatchEffect::None);

        assert_eq!(effects, Err(PatchEffectsError::Empty));
    }

    #[test]
    fn patch_effects_can_describe_manifest_and_source_changes() {
        let effects = PatchEffects::new(
            ManifestPatchEffect::MetadataOnly,
            SourcePatchEffect::Changes,
        );

        assert_eq!(
            effects,
            Ok(PatchEffects {
                manifest: ManifestPatchEffect::MetadataOnly,
                source: SourcePatchEffect::Changes,
            })
        );
    }

    #[test]
    fn patch_catalog_rejects_duplicate_definitions() {
        let first = patch_definition(
            "sha256-react-patch",
            "patches/react.patch",
            resolved_package("react", "19.0.0"),
            PatchEffects::source_only(),
        );
        let second = patch_definition(
            "sha256-react-patch",
            "patches/react-again.patch",
            resolved_package("react", "19.0.0"),
            PatchEffects::source_only(),
        );

        let catalog = PatchCatalog::new(vec![first, second], Vec::new());

        assert_eq!(
            catalog,
            Err(PatchCatalogError::DuplicateDefinition(patch_id(
                "sha256-react-patch"
            )))
        );
    }

    #[test]
    fn patch_catalog_rejects_unknown_selected_patches() {
        let unknown = patch_id("sha256-missing-patch");
        let selection = patch_selection(
            PatchSelector::Global(PatchTarget::Package(package_name("react"))),
            patch_stack(vec![unknown.clone()]),
        );

        let catalog = PatchCatalog::new(Vec::new(), vec![selection]);

        assert_eq!(
            catalog,
            Err(PatchCatalogError::UnknownSelectedPatch(unknown))
        );
    }

    #[test]
    fn patch_catalog_rejects_selected_patches_that_cannot_match_target() {
        let definition = patch_definition(
            "sha256-vue-patch",
            "patches/vue.patch",
            resolved_package("vue", "3.5.0"),
            PatchEffects::source_only(),
        );
        let selection = patch_selection(
            PatchSelector::Global(PatchTarget::Package(package_name("react"))),
            patch_stack(vec![definition.id().clone()]),
        );

        let catalog = PatchCatalog::new(vec![definition], vec![selection]);

        assert_eq!(
            catalog,
            Err(PatchCatalogError::SelectedPatchTargetMismatch(patch_id(
                "sha256-vue-patch"
            )))
        );
    }

    #[test]
    fn patch_catalog_links_definitions_to_selections() {
        let definition = patch_definition(
            "sha256-react-patch",
            "patches/react.patch",
            resolved_package("react", "19.0.0"),
            PatchEffects::source_only(),
        );
        let selection = patch_selection(
            PatchSelector::Global(PatchTarget::Package(package_name("react"))),
            patch_stack(vec![definition.id().clone()]),
        );

        let catalog = patch_catalog(vec![definition], vec![selection]);

        assert!(catalog.get(&patch_id("sha256-react-patch")).is_some());
        assert_eq!(catalog.definitions().count(), 1);
        assert_eq!(catalog.selections().len(), 1);
    }

    #[test]
    fn patch_catalog_can_be_empty() {
        let catalog = PatchCatalog::empty();

        assert_eq!(catalog.definitions().count(), 0);
        assert!(catalog.selections().is_empty());
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
                PatchTarget::Package(package_name("design-system")),
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

    fn patch_definition(
        id: &str,
        path: &str,
        base: ResolvedPackageId,
        effects: PatchEffects,
    ) -> PatchDefinition {
        PatchDefinition::new(patch_id(id), patch_path(path), base, effects)
    }

    fn patch_path(value: &str) -> PatchPath {
        match PatchPath::new(value) {
            Ok(path) => path,
            Err(error) => panic!("unexpected patch path error: {error:?}"),
        }
    }

    fn patch_catalog(
        definitions: Vec<PatchDefinition>,
        selections: Vec<PatchSelection>,
    ) -> PatchCatalog {
        match PatchCatalog::new(definitions, selections) {
            Ok(catalog) => catalog,
            Err(error) => panic!("unexpected patch catalog error: {error:?}"),
        }
    }

    fn resolved_package(name: &str, version: &str) -> ResolvedPackageId {
        ResolvedPackageId::new(
            PackageRevisionId::unpatched(package_source(name, version)),
            package_content_hash(format!("sha256-{name}-{version}")),
        )
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

    fn package_content_hash(value: impl Into<String>) -> PackageContentHash {
        match PackageContentHash::new(value) {
            Ok(hash) => hash,
            Err(error) => panic!("unexpected package content hash error: {error:?}"),
        }
    }
}
