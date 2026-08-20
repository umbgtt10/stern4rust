// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::manifest_dependency::ManifestDependency;
use crate::reporting::offence_threshold::OffenceThreshold;
use crate::reporting::output_format::OutputFormat;
use crate::settings::rule_selection::RuleSelection;
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
    pub baseline: Option<PathBuf>,
    pub config_file: Option<PathBuf>,
    pub fix: bool,
    pub write_baseline: bool,
    pub manifest_license: Option<String>,
    pub workspace_dependencies: Option<Vec<ManifestDependency>>,
    pub manifest_path: Option<PathBuf>,
    pub max_files_per_directory: Option<usize>,
    pub max_subfolders_per_directory: Option<usize>,
    pub packages: Vec<String>,
    pub excludes: Vec<String>,
    pub expected_header: Vec<String>,
    pub format: OutputFormat,
    pub offence_threshold: OffenceThreshold,
    pub selection: RuleSelection,
}
