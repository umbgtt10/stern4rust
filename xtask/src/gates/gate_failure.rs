// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fmt;

use crate::crap::report_failure::ReportFailure;
use crate::process::launch_failure::LaunchFailure;

// Why a gate failed, one variant per way it can: its tool is not installed or
// could not be started, it exited with a code nobody expected, it could not
// finish, it found what it exists to find, or it ran and reported nothing a
// reader could parse. Each says itself exactly as the run has always said it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateFailure {
    CouldNotRun(Option<i32>),
    Exited(Option<i32>),
    Found(String),
    NotInstalled(String),
    NotLaunched(LaunchFailure),
    Unreported {
        failure: ReportFailure,
        exit_code: Option<i32>,
        stderr: String,
        stdout_tail: String,
    },
}

impl From<LaunchFailure> for GateFailure {
    fn from(failure: LaunchFailure) -> Self {
        Self::NotLaunched(failure)
    }
}

impl fmt::Display for GateFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CouldNotRun(Some(code)) => write!(formatter, "could not run, exit code {code}"),
            Self::CouldNotRun(None) => write!(formatter, "could not run, terminated by signal"),
            Self::Exited(code) => write!(formatter, "exit code {code:?}"),
            Self::Found(finding) => write!(formatter, "{finding}"),
            Self::NotInstalled(binary) => {
                write!(
                    formatter,
                    "{binary} is not installed -- run: cargo install {binary}"
                )
            }
            Self::NotLaunched(failure) => write!(formatter, "{failure}"),
            Self::Unreported {
                failure,
                exit_code,
                stderr,
                stdout_tail,
            } => write!(
                formatter,
                "{failure} (exit code {exit_code:?})\nstderr: {stderr}\nstdout tail:\n{stdout_tail}"
            ),
        }
    }
}
