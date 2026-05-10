use std::path::PathBuf;

use zyn_core::{
    DependencySpec, DistTag, PackageName, PathSpec, SourceUrl, TarballSpec, VersionRange,
    WorkspaceSpec,
};

use crate::error::ManifestError;
use crate::manifest::DependencySection;

pub fn parse_dependency_spec(
    section: DependencySection,
    alias: &PackageName,
    raw_spec: &str,
) -> Result<DependencySpec, ManifestError> {
    let spec = raw_spec.trim();
    if spec.is_empty() {
        return Err(invalid_spec(
            section,
            alias,
            raw_spec,
            zyn_core::NonEmptyStringError::Empty,
        ));
    }

    if let Some(workspace) = spec.strip_prefix("workspace:") {
        return parse_workspace_spec(section, alias, raw_spec, workspace);
    }

    if let Some(path) = spec.strip_prefix("file:") {
        if path.trim().is_empty() {
            return Err(ManifestError::UnsupportedDependencySpec {
                section,
                alias: alias.as_str().to_string(),
                spec: raw_spec.to_string(),
            });
        }

        return Ok(DependencySpec::Path(PathSpec {
            path: PathBuf::from(path),
        }));
    }

    if let Some(npm) = spec.strip_prefix("npm:") {
        return parse_registry_spec(section, alias, raw_spec, npm, RegistrySpecMode::NpmAlias);
    }

    if is_remote_tarball(spec) {
        return Ok(DependencySpec::Tarball(TarballSpec {
            url: SourceUrl::new(spec).map_err(|source| ManifestError::DependencySource {
                section,
                alias: alias.as_str().to_string(),
                spec: raw_spec.to_string(),
                source,
            })?,
            integrity: None,
        }));
    }

    if looks_like_git_spec(spec) {
        return Err(ManifestError::UnsupportedDependencySpec {
            section,
            alias: alias.as_str().to_string(),
            spec: raw_spec.to_string(),
        });
    }

    if looks_like_unsupported_spec(spec) {
        return Err(ManifestError::UnsupportedDependencySpec {
            section,
            alias: alias.as_str().to_string(),
            spec: raw_spec.to_string(),
        });
    }

    parse_registry_spec(
        section,
        alias,
        raw_spec,
        spec,
        RegistrySpecMode::DependencyValue,
    )
}

fn parse_workspace_spec(
    section: DependencySection,
    alias: &PackageName,
    raw_spec: &str,
    workspace: &str,
) -> Result<DependencySpec, ManifestError> {
    let range = if workspace.is_empty() || workspace == "*" {
        None
    } else {
        Some(
            VersionRange::new(workspace)
                .map_err(|source| invalid_spec(section, alias, raw_spec, source))?,
        )
    };

    Ok(DependencySpec::Workspace(WorkspaceSpec { range }))
}

fn parse_registry_spec(
    section: DependencySection,
    alias: &PackageName,
    raw_spec: &str,
    registry_spec: &str,
    mode: RegistrySpecMode,
) -> Result<DependencySpec, ManifestError> {
    let (target, requirement) = split_target_and_requirement(alias, registry_spec, mode);
    let target = PackageName::new(target.to_string()).map_err(|source| {
        ManifestError::DependencyTargetName {
            section,
            alias: alias.as_str().to_string(),
            target: target.to_string(),
            source,
        }
    })?;

    if is_dist_tag(requirement) {
        return Ok(DependencySpec::DistTag {
            target,
            tag: DistTag::new(requirement)
                .map_err(|source| invalid_spec(section, alias, raw_spec, source))?,
        });
    }

    Ok(DependencySpec::RegistryRange {
        target,
        range: VersionRange::new(requirement)
            .map_err(|source| invalid_spec(section, alias, raw_spec, source))?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistrySpecMode {
    DependencyValue,
    NpmAlias,
}

fn split_target_and_requirement<'a>(
    alias: &'a PackageName,
    registry_spec: &'a str,
    mode: RegistrySpecMode,
) -> (&'a str, &'a str) {
    if mode == RegistrySpecMode::NpmAlias
        && let Some(index) = registry_spec
            .char_indices()
            .rev()
            .find_map(|(index, character)| (index > 0 && character == '@').then_some(index))
    {
        let (target, requirement) = registry_spec.split_at(index);
        (target, &requirement[1..])
    } else if mode == RegistrySpecMode::NpmAlias {
        (registry_spec, "*")
    } else {
        (alias.as_str(), registry_spec)
    }
}

fn is_dist_tag(requirement: &str) -> bool {
    requirement
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic())
        && requirement.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}

fn is_remote_tarball(spec: &str) -> bool {
    (spec.starts_with("https://") || spec.starts_with("http://"))
        && (spec.ends_with(".tgz") || spec.ends_with(".tar.gz"))
}

fn looks_like_git_spec(spec: &str) -> bool {
    spec.starts_with("git+")
        || spec.starts_with("git://")
        || spec.starts_with("github:")
        || spec.starts_with("gitlab:")
        || spec.starts_with("bitbucket:")
}

fn looks_like_unsupported_spec(spec: &str) -> bool {
    spec.starts_with("./")
        || spec.starts_with("../")
        || spec.starts_with('/')
        || spec.starts_with("link:")
        || spec.starts_with("portal:")
        || spec.starts_with("catalog:")
        || spec.starts_with("patch:")
        || spec.contains("://")
        || spec.contains('/')
        || spec.contains(':')
}

fn invalid_spec(
    section: DependencySection,
    alias: &PackageName,
    spec: &str,
    source: zyn_core::NonEmptyStringError,
) -> ManifestError {
    ManifestError::DependencySpec {
        section,
        alias: alias.as_str().to_string(),
        spec: spec.to_string(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dist_tags() {
        let spec = parse("latest");

        assert!(matches!(
            spec,
            DependencySpec::DistTag { ref target, ref tag }
                if target.as_str() == "react" && tag.as_str() == "latest"
        ));
    }

    #[test]
    fn parses_dist_tags_with_dots() {
        let spec = parse("beta.1");

        assert!(matches!(
            spec,
            DependencySpec::DistTag { ref target, ref tag }
                if target.as_str() == "react" && tag.as_str() == "beta.1"
        ));
    }

    #[test]
    fn parses_npm_aliases() {
        let spec = parse("npm:@types/bun@^1.2.0");

        assert!(matches!(
            spec,
            DependencySpec::RegistryRange { ref target, ref range }
                if target.as_str() == "@types/bun" && range.as_str() == "^1.2.0"
        ));
    }

    #[test]
    fn parses_npm_aliases_without_ranges() {
        let spec = parse("npm:@scope/pkg");

        assert!(matches!(
            spec,
            DependencySpec::RegistryRange { ref target, ref range }
                if target.as_str() == "@scope/pkg" && range.as_str() == "*"
        ));
    }

    #[test]
    fn parses_npm_aliases_with_dist_tags() {
        let spec = parse("npm:@scope/pkg@latest");

        assert!(matches!(
            spec,
            DependencySpec::DistTag { ref target, ref tag }
                if target.as_str() == "@scope/pkg" && tag.as_str() == "latest"
        ));
    }

    #[test]
    fn rejects_npm_aliases_without_targets() {
        let alias = package_name("react");
        let spec = parse_dependency_spec(DependencySection::Dependencies, &alias, "npm:");

        assert!(matches!(
            spec,
            Err(ManifestError::DependencyTargetName {
                alias,
                target,
                ..
            }) if alias == "react" && target.is_empty()
        ));
    }

    #[test]
    fn parses_file_specs() {
        let spec = parse("file:../can");

        assert!(matches!(
            spec,
            DependencySpec::Path(PathSpec { ref path }) if path == &PathBuf::from("../can")
        ));
    }

    #[test]
    fn rejects_file_specs_without_paths() {
        let alias = package_name("react");

        for raw in ["file:", "file:   "] {
            let spec = parse_dependency_spec(DependencySection::Dependencies, &alias, raw);

            assert!(matches!(
                spec,
                Err(ManifestError::UnsupportedDependencySpec { .. })
            ));
        }
    }

    #[test]
    fn parses_workspace_specs() {
        let spec = parse("workspace:*");

        assert!(matches!(
            spec,
            DependencySpec::Workspace(WorkspaceSpec { range: None })
        ));
    }

    #[test]
    fn parses_tarball_specs() {
        let spec = parse("https://registry.npmjs.org/zod/-/zod-3.23.8.tgz");

        assert!(matches!(
            spec,
            DependencySpec::Tarball(TarballSpec { ref url, .. })
                if url.as_str() == "https://registry.npmjs.org/zod/-/zod-3.23.8.tgz"
        ));
    }

    #[test]
    fn rejects_git_specs_until_supported() {
        let alias = package_name("react");
        let spec = parse_dependency_spec(
            DependencySection::Dependencies,
            &alias,
            "github:facebook/react",
        );

        assert!(matches!(
            spec,
            Err(ManifestError::UnsupportedDependencySpec { .. })
        ));
    }

    #[test]
    fn rejects_path_like_specs_without_file_prefix() {
        let alias = package_name("react");
        let spec = parse_dependency_spec(DependencySection::Dependencies, &alias, "../react");

        assert!(matches!(
            spec,
            Err(ManifestError::UnsupportedDependencySpec { .. })
        ));
    }

    #[test]
    fn treats_bare_numeric_versions_as_ranges() {
        let spec = parse("18");

        assert!(matches!(
            spec,
            DependencySpec::RegistryRange { ref target, ref range }
                if target.as_str() == "react" && range.as_str() == "18"
        ));
    }

    #[test]
    fn parses_scoped_aliases() {
        let alias = package_name("@scope/ui");
        let spec = parse_dependency_spec(DependencySection::Dependencies, &alias, "^0.9.0");

        assert!(matches!(
            spec,
            Ok(DependencySpec::RegistryRange { target, range })
                if target.as_str() == "@scope/ui" && range.as_str() == "^0.9.0"
        ));
    }

    fn parse(raw: &str) -> DependencySpec {
        let alias = package_name("react");
        match parse_dependency_spec(DependencySection::Dependencies, &alias, raw) {
            Ok(spec) => spec,
            Err(error) => panic!("unexpected spec error: {error:?}"),
        }
    }

    fn package_name(value: &str) -> PackageName {
        match PackageName::new(value) {
            Ok(name) => name,
            Err(error) => panic!("unexpected package name error: {error:?}"),
        }
    }
}
