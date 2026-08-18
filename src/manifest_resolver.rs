// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use anyhow::anyhow;
use cargo_metadata::MetadataCommand;

use crate::config::Config;

// Turns the requested packages into the directories to walk.
//
// Named packages that do not exist are an error rather than an empty result: a
// typo in a gate script would otherwise scan nothing and report success, which
// is the failure mode that makes a green gate meaningless.
pub struct ManifestResolver;

impl ManifestResolver {
    pub fn package_roots(config: &Config) -> Result<Vec<PathBuf>> {
        let mut command = MetadataCommand::new();
        command.no_deps();
        if let Some(manifest_path) = &config.manifest_path {
            command.manifest_path(manifest_path);
        }
        let metadata = command.exec()?;

        let mut roots = Vec::new();
        if config.packages.is_empty() {
            for package in &metadata.packages {
                roots.push(Self::manifest_dir(package.manifest_path.as_std_path()));
            }
            return Ok(roots);
        }

        for name in &config.packages {
            let package = metadata
                .packages
                .iter()
                .find(|package| package.name.as_str() == name.as_str())
                .ok_or_else(|| anyhow!("package `{name}` is not in this workspace"))?;
            roots.push(Self::manifest_dir(package.manifest_path.as_std_path()));
        }
        Ok(roots)
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
