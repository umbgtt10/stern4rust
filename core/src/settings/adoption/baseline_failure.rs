// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use core::error::Error;
use core::fmt;

use crate::source_path::SourcePath;

// A baseline that was asked for and cannot be used, or cannot be kept.
#[derive(Debug)]
pub enum BaselineFailure {
    Unreadable {
        path: SourcePath,
        cause: Box<dyn Error + Send + Sync>,
    },
    Invalid {
        path: SourcePath,
        cause: Box<dyn Error + Send + Sync>,
    },
    Unrenderable {
        path: SourcePath,
        cause: Box<dyn Error + Send + Sync>,
    },
    Unwritable {
        path: SourcePath,
        cause: Box<dyn Error + Send + Sync>,
    },
}

impl fmt::Display for BaselineFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, .. } => write!(formatter, "{path} could not be read"),
            Self::Invalid { path, .. } => write!(formatter, "{path} is not a valid baseline"),
            Self::Unrenderable { path, .. } => write!(formatter, "{path} could not be rendered"),
            Self::Unwritable { path, .. } => write!(formatter, "{path} could not be written"),
        }
    }
}

impl Error for BaselineFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Unreadable { cause, .. }
            | Self::Invalid { cause, .. }
            | Self::Unrenderable { cause, .. }
            | Self::Unwritable { cause, .. } => Some(cause.as_ref()),
        }
    }
}
