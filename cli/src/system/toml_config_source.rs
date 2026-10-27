// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use stern4rust::settings::config_failure::ConfigFailure;
use stern4rust::settings::config_file::ConfigFile;
use stern4rust::settings::package_config::PackageConfig;
use stern4rust::source_path::SourcePath;
use stern4rust::traits::config_source::ConfigSource;
use toml::from_str;

use crate::system::portable_path::PortablePath;

// `stern4rust.toml`, beside the manifest it configures, read off disk and parsed
// as TOML into the core's `ConfigFile`.
//
// Unknown keys are rejected. A misspelled `exclude` that silently did nothing
// would look exactly like an exclude that worked, which is the failure this
// tool exists to refuse -- and it is the same reason an unknown `--rule` name
// is an error rather than a switch matching nothing. The two records below are
// the file's shape; their field order is the order an unknown key's message
// lists the known ones in.
#[derive(Debug, Clone, Copy, Default)]
pub struct TomlConfigSource;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct TomlConfigFile {
    #[serde(default)]
    pub baseline: Option<PathBuf>,
    #[serde(default)]
    pub header_file: Option<PathBuf>,
    #[serde(default)]
    pub max_files_per_directory: Option<usize>,
    #[serde(default)]
    pub max_subfolders_per_directory: Option<usize>,
    #[serde(default)]
    pub offence_threshold: Option<usize>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub skip: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    // A section per package, keyed by the name its manifest declares. Absent
    // for the packages that apply everything, which is most of them.
    #[serde(default, rename = "package")]
    pub packages: BTreeMap<String, TomlPackageConfig>,
}

// A `[package.<name>]` section. `baseline` and `offence-threshold` are missing
// on purpose: neither is a property of a package, and leaving them off is what
// makes `deny_unknown_fields` reject them.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct TomlPackageConfig {
    #[serde(default)]
    pub header_file: Option<PathBuf>,
    #[serde(default)]
    pub max_files_per_directory: Option<usize>,
    #[serde(default)]
    pub max_subfolders_per_directory: Option<usize>,
    #[serde(default)]
    pub rules: Vec<String>,
    // None where the section says nothing, which inherits the root's list. A
    // stated list is the package's whole list, an empty one included -- which is
    // how a package takes back a rule its root skips; with a bare Vec the two
    // read alike and `skip = []` fell back to the root's.
    #[serde(default)]
    pub skip: Option<Vec<String>>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl TomlConfigSource {
    pub fn new() -> Self {
        Self
    }

    fn config_file(parsed: TomlConfigFile) -> ConfigFile {
        ConfigFile::new(
            parsed.baseline.as_deref().map(Self::portable),
            parsed.offence_threshold,
            PackageConfig::new(
                parsed.header_file.as_deref().map(Self::portable),
                parsed.max_files_per_directory,
                parsed.max_subfolders_per_directory,
                parsed.rules,
                Some(parsed.skip),
                parsed.exclude,
            ),
            parsed
                .packages
                .into_iter()
                .map(|(name, section)| (name, Self::package_config(section)))
                .collect(),
        )
    }

    fn package_config(parsed: TomlPackageConfig) -> PackageConfig {
        PackageConfig::new(
            parsed.header_file.as_deref().map(Self::portable),
            parsed.max_files_per_directory,
            parsed.max_subfolders_per_directory,
            parsed.rules,
            parsed.skip,
            parsed.exclude,
        )
    }

    fn portable(path: &Path) -> SourcePath {
        PortablePath::of_text(&path.to_string_lossy())
    }
}

impl ConfigSource for TomlConfigSource {
    // Ok(None) when there is no file, which is the ordinary case and not a
    // failure. A file that exists and cannot be read or parsed is an error: it
    // was written on purpose, and running as though it were absent would apply
    // a configuration nobody chose.
    fn load(&self, directory: &SourcePath) -> Result<Option<ConfigFile>, ConfigFailure> {
        let path = directory.join(ConfigFile::NAME);
        if !Path::new(path.as_str()).exists() {
            return Ok(None);
        }
        let text = read_to_string(path.as_str()).map_err(|cause| ConfigFailure::Unreadable {
            path: path.clone(),
            cause: Box::new(cause),
        })?;
        let parsed: TomlConfigFile = from_str(&text).map_err(|cause| ConfigFailure::Invalid {
            path: path.clone(),
            cause: Box::new(cause),
        })?;
        Ok(Some(Self::config_file(parsed)))
    }
}
