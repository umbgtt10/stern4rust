// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

// What one package in a workspace says about itself, in a `[package.<name>]`
// section of the root `stern4rust.toml`.
//
// The keys here are the ones that shape which rules run and against what. The
// two that are missing are missing on purpose: `baseline` is a set of
// fingerprints for one run and `offence-threshold` is about how the report
// ends, so neither is a property of a package. Leaving them off the struct is
// what makes `deny_unknown_fields` reject them, with the message it already
// gives a misspelled key -- no separate check, and nothing to keep in step.
//
// See [ADR-PerPackageConfiguration](../../docs/ADRs/ADR-PerPackageConfiguration.md).
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct PackageConfig {
    #[serde(default)]
    pub header_file: Option<PathBuf>,
    #[serde(default)]
    pub max_files_per_directory: Option<usize>,
    #[serde(default)]
    pub max_subfolders_per_directory: Option<usize>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub skip: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl PackageConfig {
    // Resolved against the directory the config file sits in, which for a root
    // file is the workspace root -- so a section writes `docs/header.txt` and
    // not the `../docs/header.txt` that a file beside a member manifest needed.
    pub fn header_file_from(&self, directory: &Path) -> Option<PathBuf> {
        self.header_file.as_ref().map(|path| directory.join(path))
    }
}
