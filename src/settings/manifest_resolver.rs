// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use anyhow::anyhow;
use cargo_metadata::Metadata;
use cargo_metadata::MetadataCommand;
use cargo_metadata::Package;

use crate::settings::config::Config;

// Turns the requested packages into the directories to walk.
//
// Named packages that do not exist are an error rather than an empty result: a
// typo in a gate script would otherwise scan nothing and report success, which
// is the failure mode that makes a green gate meaningless.
pub struct ManifestResolver;

impl ManifestResolver {
    pub fn package_roots(config: &Config) -> Result<Vec<PathBuf>> {
        let metadata = Self::metadata(config)?;
        Ok(Self::selected(&metadata, config)?
            .into_iter()
            .map(|package| Self::manifest_dir(package.manifest_path.as_std_path()))
            .collect())
    }

    // The one licence the scanned packages agree on, read from the manifest so
    // that `spdx-matches-manifest` needs no flag to hold.
    //
    // None where there is no single answer: no `license` field, or several
    // packages declaring different ones. Errors are swallowed rather than
    // reported, because `package_roots` runs moments later and says the same
    // thing better.
    pub fn license(config: &Config) -> Option<String> {
        let metadata = Self::metadata(config).ok()?;
        let selected = Self::selected(&metadata, config).ok()?;
        let declared: BTreeSet<&String> = selected
            .iter()
            .filter_map(|package| package.license.as_ref())
            .collect();
        if declared.len() != 1 || declared.len() != selected.len() {
            return None;
        }
        declared.into_iter().next().cloned()
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
