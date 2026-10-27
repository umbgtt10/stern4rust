// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::error::Error;
use std::fmt;

// A path given on the command line that the core cannot hold: it is not UTF-8.
// The run stops and names it rather than looking for a file under a name it
// does not have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnusablePath {
    pub flag: String,
    pub shown: String,
}

impl fmt::Display for UnusablePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {} is not valid UTF-8, which stern4rust needs to read beside it",
            self.flag, self.shown
        )
    }
}

impl Error for UnusablePath {}
