// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use core::error::Error;
use core::fmt;

use crate::source_path::SourcePath;

// A file `--fix` repaired and the tree could not write back, and the platform's
// reason.
#[derive(Debug)]
pub struct SourceWriteFailure {
    pub path: SourcePath,
    pub cause: Box<dyn Error + Send + Sync>,
}

impl fmt::Display for SourceWriteFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} could not be rewritten", self.path)
    }
}

impl Error for SourceWriteFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.cause.as_ref())
    }
}
