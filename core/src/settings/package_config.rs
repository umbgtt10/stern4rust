// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::source_path::SourcePath;

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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageConfig {
    header_file: Option<SourcePath>,
    max_files_per_directory: Option<usize>,
    max_subfolders_per_directory: Option<usize>,
    rules: Vec<String>,
    // None where the section says nothing, which inherits the root's list. A
    // stated list is the package's whole list, an empty one included -- which is
    // how a package takes back a rule its root skips; with a bare Vec the two
    // read alike and `skip = []` fell back to the root's.
    skip: Option<Vec<String>>,
    exclude: Vec<String>,
}

impl PackageConfig {
    pub fn new(
        header_file: Option<SourcePath>,
        max_files_per_directory: Option<usize>,
        max_subfolders_per_directory: Option<usize>,
        rules: Vec<String>,
        skip: Option<Vec<String>>,
        exclude: Vec<String>,
    ) -> Self {
        Self {
            header_file,
            max_files_per_directory,
            max_subfolders_per_directory,
            rules,
            skip,
            exclude,
        }
    }

    pub fn exclude(&self) -> &[String] {
        &self.exclude
    }

    pub fn header_file(&self) -> Option<&SourcePath> {
        self.header_file.as_ref()
    }

    // Resolved against the directory the config file sits in, which for a root
    // file is the workspace root -- so a section writes `docs/header.txt` and
    // not the `../docs/header.txt` that a file beside a member manifest needed.
    pub fn header_file_from(&self, directory: &SourcePath) -> Option<SourcePath> {
        self.header_file
            .as_ref()
            .map(|path| directory.join(path.as_str()))
    }

    pub fn max_files_per_directory(&self) -> Option<usize> {
        self.max_files_per_directory
    }

    pub fn max_subfolders_per_directory(&self) -> Option<usize> {
        self.max_subfolders_per_directory
    }

    pub fn rules(&self) -> &[String] {
        &self.rules
    }

    pub fn skip(&self) -> Option<&[String]> {
        self.skip.as_deref()
    }

    pub fn with_header_file(self, header_file: SourcePath) -> Self {
        Self {
            header_file: Some(header_file),
            ..self
        }
    }

    pub fn with_skip(self, skip: Vec<String>) -> Self {
        Self {
            skip: Some(skip),
            ..self
        }
    }
}
