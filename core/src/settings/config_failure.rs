// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use core::error::Error;
use core::fmt;

use crate::source_path::SourcePath;

// A `stern4rust.toml` that is there and cannot be used: written on purpose, so
// running as though it were absent would apply a configuration nobody chose.
#[derive(Debug)]
pub enum ConfigFailure {
    Unreadable {
        path: SourcePath,
        cause: Box<dyn Error + Send + Sync>,
    },
    Invalid {
        path: SourcePath,
        cause: Box<dyn Error + Send + Sync>,
    },
}

impl fmt::Display for ConfigFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, .. } => write!(formatter, "{path} could not be read"),
            Self::Invalid { path, .. } => {
                write!(formatter, "{path} is not valid stern4rust configuration")
            }
        }
    }
}

impl Error for ConfigFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Unreadable { cause, .. } | Self::Invalid { cause, .. } => Some(cause.as_ref()),
        }
    }
}
