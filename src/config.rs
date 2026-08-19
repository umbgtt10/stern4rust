// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use crate::offence_threshold::OffenceThreshold;
use crate::output_format::OutputFormat;

// The threshold is an OffenceThreshold rather than a bare usize precisely so
// this can stay derived. A usize field would default to zero, which this tool
// reads as "no limit" -- the opposite of the CLI default, and a mismatch that
// would only surface on a codebase big enough to need the cap.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    pub manifest_path: Option<PathBuf>,
    pub packages: Vec<String>,
    pub expected_header: Vec<String>,
    pub format: OutputFormat,
    pub offence_threshold: OffenceThreshold,
}
