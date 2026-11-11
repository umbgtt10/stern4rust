// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::process::launch_failure::LaunchFailure;
use std::fmt;

// Why core could not be scored: git or cargo would not start, a command they
// ran failed -- named, with what it wrote to stderr -- a tool's report could
// not be read, or no source file under core/ is staged to score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoreFailure {
    Launch(LaunchFailure),
    CommandFailed { command: String, stderr: String },
    Unreadable { tool: String, reason: String },
    NoSource,
}

impl From<LaunchFailure> for ScoreFailure {
    fn from(failure: LaunchFailure) -> Self {
        Self::Launch(failure)
    }
}

impl fmt::Display for ScoreFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Launch(failure) => write!(formatter, "{failure}"),
            Self::CommandFailed { command, stderr } => {
                write!(formatter, "{command} failed: {stderr}")
            }
            Self::Unreadable { tool, reason } => {
                write!(formatter, "{tool}'s report is unreadable: {reason}")
            }
            Self::NoSource => write!(formatter, "core/ has no staged source file"),
        }
    }
}
