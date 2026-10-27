// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use anyhow::Result;
use anyhow::anyhow;

use crate::finding::model::manifest_dependency::ManifestDependency;
use crate::settings::config::Config;
use crate::settings::package_description::PackageDescription;
use crate::settings::scanned_package::ScannedPackage;
use crate::settings::workspace_description::WorkspaceDescription;
use crate::source_path::SourcePath;

// Turns the requested packages into the directories to walk.
//
// Named packages that do not exist are an error rather than an empty result: a
// typo in a gate script would otherwise scan nothing and report success, which
// is the failure mode that makes a green gate meaningless.
pub struct ManifestResolver;

impl ManifestResolver {
    pub const MANIFEST: &'static str = "Cargo.toml";

    // Every package the run will walk, each carrying what its own manifest says
    // about it.
    //
    // Replaces the pair this used to be -- a list of bare directories, and one
    // licence aggregated across all of them. The aggregate could not answer for
    // a workspace: it compared a set of distinct licence strings against a count
    // of packages, which are only ever the same length when there is one
    // package, so four members all declaring Apache-2.0 read as none declaring
    // it. See
    // [ADR-ManifestDataIsPerPackage](../../docs/ADRs/ADR-ManifestDataIsPerPackage.md).
    // Every dependency each package's manifest declares, as written, in every
    // table -- the plain three and each under `[target.<cfg>]` -- whether or not
    // the package sits in a workspace.
    pub fn declared_dependencies(workspace: &WorkspaceDescription) -> Vec<ManifestDependency> {
        workspace
            .packages
            .iter()
            .flat_map(|package| package.declared.iter().cloned())
            .collect()
    }

    pub fn packages(
        workspace: &WorkspaceDescription,
        config: &Config,
    ) -> Result<Vec<ScannedPackage>> {
        Ok(Self::selected(workspace, config)?
            .into_iter()
            .map(|package| {
                ScannedPackage::new(
                    package.name.as_str(),
                    Self::manifest_dir(&package.manifest_path),
                    package.license.clone(),
                )
            })
            .collect())
    }

    pub fn relative_to(root: &SourcePath, path: &SourcePath) -> String {
        path.strip_prefix(root)
            .unwrap_or(path.as_str())
            .replace('\\', "/")
    }

    // Every dependency each member manifest declares, as written, in the three
    // plain tables. None when the package is not a workspace, which is not the
    // same as a workspace with nothing to report.
    //
    // Read from the TOML rather than from `cargo metadata`, because the question
    // is *how* a dependency was declared and resolution erases that: a
    // `{ workspace = true }` and a spelled-out version look identical once
    // resolved. Target tables are left out: `workspace-dependencies` has never
    // judged them.
    pub fn workspace_dependencies(
        workspace: &WorkspaceDescription,
    ) -> Option<Vec<ManifestDependency>> {
        if !workspace.declares_workspace {
            return None;
        }
        Some(
            Self::declared_dependencies(workspace)
                .into_iter()
                .filter(|dependency| dependency.target().is_none())
                .collect(),
        )
    }

    // Every package in the workspace, whether this run walks it or not. What a
    // `[package.<name>]` section is checked against: a section for a member this
    // run is not scanning is ordinary, one for a member that does not exist is a
    // typo.
    pub fn workspace_package_names(workspace: &WorkspaceDescription) -> Vec<String> {
        workspace
            .packages
            .iter()
            .map(|package| package.name.to_string())
            .collect()
    }

    // The directory every manifest path in `workspace_dependencies` is stated
    // relative to.
    //
    // Not derivable from `--manifest-path`, which may name a member: cargo
    // still reports the workspace above it, so a caller that assumed otherwise
    // would compare a member-relative path against a workspace-relative one and
    // match nothing.
    pub fn workspace_root(workspace: &WorkspaceDescription) -> SourcePath {
        workspace.root.clone()
    }

    fn manifest_dir(manifest_path: &SourcePath) -> SourcePath {
        manifest_path
            .parent()
            .unwrap_or_else(|| manifest_path.clone())
    }

    fn selected<'a>(
        workspace: &'a WorkspaceDescription,
        config: &Config,
    ) -> Result<Vec<&'a PackageDescription>> {
        if config.packages.is_empty() {
            return Ok(workspace.packages.iter().collect());
        }
        config
            .packages
            .iter()
            .map(|name| {
                workspace
                    .packages
                    .iter()
                    .find(|package| package.name.as_str() == name.as_str())
                    .ok_or_else(|| anyhow!("package `{name}` is not in this workspace"))
            })
            .collect()
    }
}
