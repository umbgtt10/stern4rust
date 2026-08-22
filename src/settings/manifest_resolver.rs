// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use anyhow::anyhow;
use cargo_metadata::Metadata;
use cargo_metadata::MetadataCommand;
use cargo_metadata::Package;
use toml::Table;
use toml::Value;

use crate::finding::model::manifest_dependency::ManifestDependency;
use crate::settings::config::Config;
use crate::settings::scanned_package::ScannedPackage;

// Turns the requested packages into the directories to walk.
//
// Named packages that do not exist are an error rather than an empty result: a
// typo in a gate script would otherwise scan nothing and report success, which
// is the failure mode that makes a green gate meaningless.
pub struct ManifestResolver;

impl ManifestResolver {
    pub const MANIFEST: &'static str = "Cargo.toml";

    // Every package in the workspace, whether this run walks it or not. What a
    // `[package.<name>]` section is checked against: a section for a member this
    // run is not scanning is ordinary, one for a member that does not exist is a
    // typo.
    pub fn workspace_package_names(config: &Config) -> Result<Vec<String>> {
        Ok(Self::metadata(config)?
            .packages
            .iter()
            .map(|package| package.name.to_string())
            .collect())
    }

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
    pub fn packages(config: &Config) -> Result<Vec<ScannedPackage>> {
        let metadata = Self::metadata(config)?;
        Ok(Self::selected(&metadata, config)?
            .into_iter()
            .map(|package| {
                ScannedPackage::new(
                    package.name.as_str(),
                    Self::manifest_dir(package.manifest_path.as_std_path()),
                    package.license.clone(),
                )
            })
            .collect())
    }

    // Every dependency each member manifest declares, as written. None when the
    // package is not a workspace, which is not the same as a workspace with
    // nothing to report.
    //
    // Read from the TOML rather than from `cargo metadata`, because the question
    // is *how* a dependency was declared and resolution erases that: a
    // `{ workspace = true }` and a spelled-out version look identical once
    // resolved.
    pub fn workspace_dependencies(config: &Config) -> Option<Vec<ManifestDependency>> {
        let metadata = Self::metadata(config).ok()?;
        let root = metadata.workspace_root.as_std_path();
        if !Self::is_workspace(&root.join(Self::MANIFEST)) {
            return None;
        }
        Some(
            metadata
                .packages
                .iter()
                .flat_map(|package| {
                    let path = package.manifest_path.as_std_path();
                    Self::declared_in(path, &Self::relative_to(root, path))
                })
                .collect(),
        )
    }

    fn is_workspace(manifest: &Path) -> bool {
        Self::parsed(manifest).is_some_and(|table| table.contains_key("workspace"))
    }

    fn parsed(manifest: &Path) -> Option<Table> {
        read_to_string(manifest).ok()?.parse::<Table>().ok()
    }

    fn declared_in(manifest: &Path, shown: &str) -> Vec<ManifestDependency> {
        let Some(table) = Self::parsed(manifest) else {
            return Vec::new();
        };
        ManifestDependency::SECTIONS
            .into_iter()
            .flat_map(|section| Self::in_section(&table, section, shown))
            .collect()
    }

    // A dependency takes from the workspace when it is a table saying so. A
    // string version, or a table with a version of its own, does not.
    fn in_section(table: &Table, section: &str, shown: &str) -> Vec<ManifestDependency> {
        table
            .get(section)
            .and_then(Value::as_table)
            .map(|declared| {
                declared
                    .iter()
                    .map(|(name, value)| {
                        let takes = value
                            .as_table()
                            .and_then(|entry| entry.get("workspace"))
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        ManifestDependency::new(shown, name, section, takes)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    // The directory every manifest path in `workspace_dependencies` is stated
    // relative to.
    //
    // Not derivable from `--manifest-path`, which may name a member: cargo
    // still reports the workspace above it, so a caller that assumed otherwise
    // would compare a member-relative path against a workspace-relative one and
    // match nothing.
    pub fn workspace_root(config: &Config) -> Option<PathBuf> {
        Self::metadata(config)
            .ok()
            .map(|metadata| metadata.workspace_root.as_std_path().to_path_buf())
    }

    fn metadata(config: &Config) -> Result<Metadata> {
        let mut command = MetadataCommand::new();
        command.no_deps();
        if let Some(manifest_path) = &config.manifest_path {
            command.manifest_path(manifest_path);
        }
        Ok(command.exec()?)
    }

    fn selected<'a>(metadata: &'a Metadata, config: &Config) -> Result<Vec<&'a Package>> {
        if config.packages.is_empty() {
            return Ok(metadata.packages.iter().collect());
        }
        config
            .packages
            .iter()
            .map(|name| {
                metadata
                    .packages
                    .iter()
                    .find(|package| package.name.as_str() == name.as_str())
                    .ok_or_else(|| anyhow!("package `{name}` is not in this workspace"))
            })
            .collect()
    }

    pub fn relative_to(root: &Path, path: &Path) -> String {
        path.strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }

    fn manifest_dir(manifest_path: &Path) -> PathBuf {
        manifest_path
            .parent()
            .map_or_else(|| manifest_path.to_path_buf(), Path::to_path_buf)
    }
}
