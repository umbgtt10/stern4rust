// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::manifest_dependency::ManifestDependency;
use crate::finding::model::name_readers::NameReaders;
use crate::reporting::offence_threshold::OffenceThreshold;
use crate::reporting::output_format::OutputFormat;
use crate::settings::rule_selection::RuleSelection;
use std::path::Path;
use std::path::PathBuf;

// The threshold is an OffenceThreshold rather than a bare usize precisely so
// this can stay derived. A usize field would default to zero, which this tool
// reads as "no limit" -- the opposite of the CLI default, and a mismatch that
// would only surface on a codebase big enough to need the cap.
//
// The directory limits are Options for the same reason and the opposite danger:
// a bare usize defaulting to zero would mean "no files may exist", turning
// every directory in an unconfigured run into an offence. None means "not
// configured", and each rule supplies its own default.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    baseline: Option<PathBuf>,
    config_file: Option<PathBuf>,
    fix: bool,
    write_baseline: bool,
    manifest_license: Option<String>,
    workspace_dependencies: Option<Vec<ManifestDependency>>,
    manifest_path: Option<PathBuf>,
    max_files_per_directory: Option<usize>,
    max_subfolders_per_directory: Option<usize>,
    name_readers: NameReaders,
    packages: Vec<String>,
    excludes: Vec<String>,
    expected_header: Vec<String>,
    format: OutputFormat,
    offence_threshold: OffenceThreshold,
    selection: RuleSelection,
}

impl Config {
    pub fn baseline(&self) -> Option<&Path> {
        self.baseline.as_deref()
    }

    pub fn config_file(&self) -> Option<&Path> {
        self.config_file.as_deref()
    }

    pub fn excludes(&self) -> &[String] {
        &self.excludes
    }

    pub fn expected_header(&self) -> &[String] {
        &self.expected_header
    }

    pub fn fix(&self) -> bool {
        self.fix
    }

    pub fn format(&self) -> OutputFormat {
        self.format
    }

    pub fn manifest_license(&self) -> Option<&str> {
        self.manifest_license.as_deref()
    }

    pub fn manifest_path(&self) -> Option<&Path> {
        self.manifest_path.as_deref()
    }

    pub fn max_files_per_directory(&self) -> Option<usize> {
        self.max_files_per_directory
    }

    pub fn max_subfolders_per_directory(&self) -> Option<usize> {
        self.max_subfolders_per_directory
    }

    pub fn name_readers(&self) -> &NameReaders {
        &self.name_readers
    }

    pub fn offence_threshold(&self) -> OffenceThreshold {
        self.offence_threshold
    }

    pub fn packages(&self) -> &[String] {
        &self.packages
    }

    pub fn selection(&self) -> &RuleSelection {
        &self.selection
    }

    pub fn with_baseline(self, baseline: Option<PathBuf>) -> Self {
        Self { baseline, ..self }
    }

    pub fn with_config_file(self, config_file: Option<PathBuf>) -> Self {
        Self {
            config_file,
            ..self
        }
    }

    pub fn with_excludes(self, excludes: Vec<String>) -> Self {
        Self { excludes, ..self }
    }

    pub fn with_expected_header(self, expected_header: Vec<String>) -> Self {
        Self {
            expected_header,
            ..self
        }
    }

    pub fn with_fix(self, fix: bool) -> Self {
        Self { fix, ..self }
    }

    pub fn with_format(self, format: OutputFormat) -> Self {
        Self { format, ..self }
    }

    pub fn with_manifest_license(self, manifest_license: Option<String>) -> Self {
        Self {
            manifest_license,
            ..self
        }
    }

    pub fn with_manifest_path(self, manifest_path: Option<PathBuf>) -> Self {
        Self {
            manifest_path,
            ..self
        }
    }

    pub fn with_max_files_per_directory(self, max_files_per_directory: Option<usize>) -> Self {
        Self {
            max_files_per_directory,
            ..self
        }
    }

    pub fn with_max_subfolders_per_directory(
        self,
        max_subfolders_per_directory: Option<usize>,
    ) -> Self {
        Self {
            max_subfolders_per_directory,
            ..self
        }
    }

    pub fn with_name_readers(self, name_readers: NameReaders) -> Self {
        Self {
            name_readers,
            ..self
        }
    }

    pub fn with_offence_threshold(self, offence_threshold: OffenceThreshold) -> Self {
        Self {
            offence_threshold,
            ..self
        }
    }

    pub fn with_packages(self, packages: Vec<String>) -> Self {
        Self { packages, ..self }
    }

    pub fn with_selection(self, selection: RuleSelection) -> Self {
        Self { selection, ..self }
    }

    pub fn with_workspace_dependencies(
        self,
        workspace_dependencies: Option<Vec<ManifestDependency>>,
    ) -> Self {
        Self {
            workspace_dependencies,
            ..self
        }
    }

    pub fn with_write_baseline(self, write_baseline: bool) -> Self {
        Self {
            write_baseline,
            ..self
        }
    }

    pub fn workspace_dependencies(&self) -> Option<&[ManifestDependency]> {
        self.workspace_dependencies.as_deref()
    }

    pub fn write_baseline(&self) -> bool {
        self.write_baseline
    }
}
