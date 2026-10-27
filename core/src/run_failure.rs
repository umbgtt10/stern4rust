// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;
use core::error::Error;
use core::fmt;

use crate::rules::source::header_rule::HeaderRule;
use crate::settings::adoption::baseline_failure::BaselineFailure;
use crate::settings::adoption::exclusion_failure::ExclusionFailure;
use crate::settings::config_failure::ConfigFailure;
use crate::settings::config_file::ConfigFile;
use crate::settings::workspace_read_failure::WorkspaceReadFailure;
use crate::source_read_failure::SourceReadFailure;
use crate::source_write_failure::SourceWriteFailure;

// Why a run could not judge anything: exit code 1, where a broken rule is 2.
//
// Each says what the reader can act on, worded as the run has always worded
// it. A failure that wraps another states that one's own words and passes its
// reason on, so nothing is said twice.
#[derive(Debug)]
pub enum RunFailure {
    Baseline(BaselineFailure),
    Config(ConfigFailure),
    Exclusion(ExclusionFailure),
    HeaderUnreadable(SourceReadFailure),
    HeaderWithoutFile,
    NoBaselinePath,
    NoRules,
    PackageNotFound(String),
    Rewrite(SourceWriteFailure),
    SourceList(SourceReadFailure),
    UnknownRules {
        unknown: Vec<String>,
        known: Vec<&'static str>,
    },
    UnknownSections {
        unknown: Vec<String>,
        workspace: Vec<String>,
    },
    Workspace(WorkspaceReadFailure),
}

impl fmt::Display for RunFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Baseline(failure) => write!(formatter, "{failure}"),
            Self::Config(failure) => write!(formatter, "{failure}"),
            Self::Exclusion(failure) => write!(formatter, "{failure}"),
            Self::HeaderUnreadable(failure) => {
                write!(formatter, "failed to read header file {}", failure.path)
            }
            Self::HeaderWithoutFile => write!(
                formatter,
                "--rule {} needs --header-file, otherwise the run would apply no rules at all",
                HeaderRule::NAME
            ),
            Self::NoBaselinePath => {
                formatter.write_str("--write-baseline needs a path to write to")
            }
            Self::NoRules => formatter.write_str(
                "no rules are configured, so nothing would be checked -- pass --header-file to \
                 enable the header rule",
            ),
            Self::PackageNotFound(name) => {
                write!(formatter, "package `{name}` is not in this workspace")
            }
            Self::Rewrite(failure) => write!(formatter, "{failure}"),
            Self::SourceList(failure) => write!(formatter, "{failure}"),
            Self::UnknownRules { unknown, known } => write!(
                formatter,
                "unknown rule name(s): {} -- the rules are: {}",
                unknown.join(", "),
                known.join(", ")
            ),
            Self::UnknownSections { unknown, workspace } => write!(
                formatter,
                "{} configures package(s) that are not in this workspace: {} -- it holds: {}",
                ConfigFile::NAME,
                unknown.join(", "),
                workspace.join(", ")
            ),
            Self::Workspace(failure) => write!(formatter, "{failure}"),
        }
    }
}

impl Error for RunFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Baseline(failure) => failure.source(),
            Self::Config(failure) => failure.source(),
            Self::Exclusion(failure) => failure.source(),
            Self::HeaderUnreadable(failure) | Self::SourceList(failure) => failure.source(),
            Self::Rewrite(failure) => failure.source(),
            Self::Workspace(failure) => failure.source(),
            Self::HeaderWithoutFile
            | Self::NoBaselinePath
            | Self::NoRules
            | Self::PackageNotFound(_)
            | Self::UnknownRules { .. }
            | Self::UnknownSections { .. } => None,
        }
    }
}
