// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use crate::output_format::OutputFormat;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    pub manifest_path: Option<PathBuf>,
    pub packages: Vec<String>,
    pub expected_header: Vec<String>,
    pub format: OutputFormat,
}
