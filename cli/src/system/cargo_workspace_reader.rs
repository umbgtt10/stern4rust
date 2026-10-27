// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;

use cargo_metadata::Metadata;
use cargo_metadata::MetadataCommand;
use cargo_metadata::Package;
use stern4rust::settings::manifest_resolver::ManifestResolver;
use stern4rust::settings::package_description::PackageDescription;
use stern4rust::settings::workspace_description::WorkspaceDescription;
use stern4rust::settings::workspace_read_failure::WorkspaceReadFailure;
use stern4rust::source_path::SourcePath;
use stern4rust::traits::workspace_reader::WorkspaceReader;
use toml::Table;

use crate::system::dependency_tables::DependencyTables;
use crate::system::portable_path::PortablePath;

// The workspace as `cargo metadata --no-deps` describes it, with what each
// member's manifest declares read from the TOML, translated into the core's own
// terms so that nothing past this file knows cargo_metadata's or toml's types.
//
// The manifests are read as written rather than taken from `cargo metadata`,
// because the manifest rules ask how a dependency was declared and resolution
// erases that. A manifest that cannot be read or parsed declares nothing; cargo
// has already read it, so that is not the run's failure to report.
//
// The manifest path stays a `PathBuf` here and never enters the core, so one
// that is not UTF-8 reaches cargo exactly as it was given.
#[derive(Debug, Clone)]
pub struct CargoWorkspaceReader {
    manifest_path: Option<PathBuf>,
}

impl CargoWorkspaceReader {
    pub fn new(manifest_path: Option<PathBuf>) -> Self {
        Self { manifest_path }
    }

    pub fn translate(metadata: &Metadata) -> WorkspaceDescription {
        let root = PortablePath::of_text(metadata.workspace_root.as_str());
        WorkspaceDescription {
            declares_workspace: Self::parsed(
                &metadata
                    .workspace_root
                    .as_std_path()
                    .join(ManifestResolver::MANIFEST),
            )
            .is_some_and(|table| table.contains_key("workspace")),
            packages: metadata
                .packages
                .iter()
                .map(|package| Self::package(&root, package))
                .collect(),
            root,
        }
    }

    fn package(root: &SourcePath, package: &Package) -> PackageDescription {
        let manifest_path = PortablePath::of_text(package.manifest_path.as_str());
        let shown = ManifestResolver::relative_to(root, &manifest_path);
        PackageDescription {
            name: package.name.to_string(),
            license: package.license.clone(),
            declared: Self::parsed(package.manifest_path.as_std_path())
                .map(|table| DependencyTables::declared(&table, &shown))
                .unwrap_or_default(),
            manifest_path,
        }
    }

    fn parsed(manifest: &Path) -> Option<Table> {
        read_to_string(manifest).ok()?.parse::<Table>().ok()
    }
}

impl WorkspaceReader for CargoWorkspaceReader {
    fn describe(&self) -> Result<WorkspaceDescription, WorkspaceReadFailure> {
        let mut command = MetadataCommand::new();
        command.no_deps();
        if let Some(manifest_path) = &self.manifest_path {
            command.manifest_path(manifest_path);
        }
        let metadata = command.exec().map_err(|error| WorkspaceReadFailure {
            cause: Box::new(error),
        })?;
        Ok(Self::translate(&metadata))
    }
}
