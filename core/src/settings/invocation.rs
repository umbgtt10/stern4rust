// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reporting::output_format::OutputFormat;
use crate::source_path::SourcePath;

// What the command line asked of a run, in the core's terms. The manifest's
// directory is where `stern4rust.toml` and a default baseline are looked for;
// the manifest itself is the workspace reader's.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Invocation {
    pub baseline: Option<SourcePath>,
    pub excludes: Vec<String>,
    pub fix: bool,
    pub format: OutputFormat,
    pub header_file: Option<SourcePath>,
    pub list_rules: bool,
    pub manifest_directory: SourcePath,
    pub offence_threshold: Option<usize>,
    pub packages: Vec<String>,
    pub rules: Vec<String>,
    pub skipped_rules: Vec<String>,
    pub write_baseline: bool,
}
