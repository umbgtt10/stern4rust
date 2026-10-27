// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

use crate::boundary::source_read_failure::SourceReadFailure;

pub trait SourceFiles {
    fn read(&self, file: &Path) -> Result<String, SourceReadFailure>;

    fn rust_files(&self, dir: &Path) -> Result<Vec<PathBuf>, SourceReadFailure>;
}
