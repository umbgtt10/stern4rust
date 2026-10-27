// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::settings::package_config::PackageConfig;
use crate::source_path::SourcePath;

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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigFile {
    baseline: Option<SourcePath>,
    header_file: Option<SourcePath>,
    max_files_per_directory: Option<usize>,
    max_subfolders_per_directory: Option<usize>,
    offence_threshold: Option<usize>,
    rules: Vec<String>,
    skip: Vec<String>,
    exclude: Vec<String>,
    // A section per package, keyed by the name its manifest declares. Absent
    // for the packages that apply everything, which is most of them.
    packages: BTreeMap<String, PackageConfig>,
}

impl ConfigFile {
    pub const NAME: &'static str = "stern4rust.toml";

    // The root keys a package section may also set come as one, from `root`;
    // the root's `skip` is its whole list, so a root that states none skips
    // nothing.
    pub fn new(
        baseline: Option<SourcePath>,
        offence_threshold: Option<usize>,
        root: PackageConfig,
        packages: BTreeMap<String, PackageConfig>,
    ) -> Self {
        Self {
            baseline,
            header_file: root.header_file().cloned(),
            max_files_per_directory: root.max_files_per_directory(),
            max_subfolders_per_directory: root.max_subfolders_per_directory(),
            offence_threshold,
            rules: root.rules().to_vec(),
            skip: root.skip().map(<[String]>::to_vec).unwrap_or_default(),
            exclude: root.exclude().to_vec(),
            packages,
        }
    }

    pub fn baseline(&self) -> Option<&SourcePath> {
        self.baseline.as_ref()
    }

    // Paths in the file are relative to the file, so a repository can be checked
    // out anywhere and cloned into any directory name.
    pub fn baseline_from(&self, directory: &SourcePath) -> Option<SourcePath> {
        self.baseline
            .as_ref()
            .map(|relative| directory.join(relative.as_str()))
    }

    pub fn exclude(&self) -> &[String] {
        &self.exclude
    }

    pub fn header_file(&self) -> Option<&SourcePath> {
        self.header_file.as_ref()
    }

    pub fn header_file_from(&self, directory: &SourcePath) -> Option<SourcePath> {
        self.header_file
            .as_ref()
            .map(|relative| directory.join(relative.as_str()))
    }

    pub fn into_packages(self) -> BTreeMap<String, PackageConfig> {
        self.packages
    }

    pub fn max_files_per_directory(&self) -> Option<usize> {
        self.max_files_per_directory
    }

    pub fn max_subfolders_per_directory(&self) -> Option<usize> {
        self.max_subfolders_per_directory
    }

    pub fn offence_threshold(&self) -> Option<usize> {
        self.offence_threshold
    }

    pub fn packages(&self) -> &BTreeMap<String, PackageConfig> {
        &self.packages
    }

    pub fn rules(&self) -> &[String] {
        &self.rules
    }

    pub fn skip(&self) -> &[String] {
        &self.skip
    }
}
