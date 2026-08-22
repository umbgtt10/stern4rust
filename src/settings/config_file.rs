// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::settings::package_config::PackageConfig;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use toml::from_str;

// `stern4rust.toml`, beside the manifest it configures.
//
// Every switch this tool has had to be repeated at every invocation, which is
// tolerable for a person running it once and useless for a repository that
// wants the same run in a gate script, a pre-commit hook and a developer's
// terminal. The excludes are the sharpest case: a pattern naming a vendored
// tree is a fact about the repository, not about one command line.
//
// Unknown keys are rejected. A misspelled `exclude` that silently did nothing
// would look exactly like an exclude that worked, which is the failure this
// tool exists to refuse -- and it is the same reason an unknown `--rule` name
// is an error rather than a switch matching nothing.
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ConfigFile {
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
    pub packages: BTreeMap<String, PackageConfig>,
}

impl ConfigFile {
    pub const NAME: &'static str = "stern4rust.toml";

    // Ok(None) when there is no file, which is the ordinary case and not a
    // failure. A file that exists and cannot be read or parsed is an error: it
    // was written on purpose, and running as though it were absent would apply
    // a configuration nobody chose.
    pub fn load(directory: &Path) -> Result<Option<Self>> {
        let path = directory.join(Self::NAME);
        if !path.exists() {
            return Ok(None);
        }
        let text = read_to_string(&path)
            .with_context(|| format!("{} could not be read", path.display()))?;
        let parsed = from_str(&text)
            .with_context(|| format!("{} is not valid stern4rust configuration", path.display()))?;
        Ok(Some(parsed))
    }

    // Paths in the file are relative to the file, so a repository can be checked
    // out anywhere and cloned into any directory name.
    pub fn baseline_from(&self, directory: &Path) -> Option<PathBuf> {
        self.baseline
            .as_ref()
            .map(|relative| directory.join(relative))
    }

    pub fn header_file_from(&self, directory: &Path) -> Option<PathBuf> {
        self.header_file
            .as_ref()
            .map(|relative| directory.join(relative))
    }
}
