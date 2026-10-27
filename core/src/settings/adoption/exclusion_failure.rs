// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use core::error::Error;
use core::fmt;

use globset::Error as GlobError;

// An `--exclude` pattern that is not a glob. An error rather than a pattern
// matching nothing, which is a legitimate outcome the report names.
#[derive(Debug)]
pub struct ExclusionFailure {
    pub pattern: String,
    pub cause: GlobError,
}

impl fmt::Display for ExclusionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "`{}` is not a usable exclude pattern",
            self.pattern
        )
    }
}

impl Error for ExclusionFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}
