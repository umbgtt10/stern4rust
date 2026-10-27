// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use core::error::Error;
use core::fmt;

// The workspace could not be described. It says what the reader said, and no
// more, so the reason reaches the reader in the reader's own words.
#[derive(Debug)]
pub struct WorkspaceReadFailure {
    pub cause: Box<dyn Error + Send + Sync>,
}

impl fmt::Display for WorkspaceReadFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.cause)
    }
}

impl Error for WorkspaceReadFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.source()
    }
}
