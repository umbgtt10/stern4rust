// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fmt;
use std::path::PathBuf;

// A directory that could not be listed or a file that could not be read, and
// why -- either way the scan saw less than it was asked to, so it cannot pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceReadFailure {
    pub path: PathBuf,
    pub reason: String,
}

impl fmt::Display for SourceReadFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "could not read {}: {}",
            self.path.display(),
            self.reason
        )
    }
}
