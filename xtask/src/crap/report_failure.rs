// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fmt;

// Why crap4rust's output yielded no report: none was there, or the one there did
// not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportFailure {
    Missing,
    Unparseable(String),
}

impl fmt::Display for ReportFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(
                formatter,
                "could not find a JSON report in crap4rust's output"
            ),
            Self::Unparseable(reason) => {
                write!(formatter, "could not parse crap4rust JSON: {reason}")
            }
        }
    }
}
