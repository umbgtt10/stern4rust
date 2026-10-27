// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reporting::offence::Offence;
use crate::reporting::offence_threshold::OffenceThreshold;
use crate::reporting::package_roster::PackageRoster;
use crate::reporting::unconfigured_rule::UnconfiguredRule;

// Everything one run has to say, before anybody chooses how to say it: what was
// read and left out, which rules ran where, what a baseline hid, and the
// offences in report order. A renderer decides the form; what goes in it is
// decided here, once, for both forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    pub files_scanned: usize,
    pub threshold: OffenceThreshold,
    pub applied: Vec<String>,
    pub skipped: Vec<String>,
    pub unconfigured: Vec<UnconfiguredRule>,
    pub rosters: Vec<PackageRoster>,
    pub exclusions: Vec<(String, usize)>,
    pub config_file: Option<String>,
    pub baseline: Option<String>,
    pub suppressed: usize,
    pub stale: usize,
    pub fixed: usize,
    pub offences: Vec<Offence>,
}
