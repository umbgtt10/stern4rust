// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Which rules ran against one package, and which did not.
//
// A run whose members answer to different rule sets cannot state one roster and
// stay honest: `applied: <twenty-one rules>` is false the moment one package
// applies twenty. But a run whose members all agree should not cost the reader a
// block each either, which is what `agrees_with` is for -- the printer collapses
// rosters that say the same thing and only separates the ones that do not.
//
// The name is what the comparison ignores. Two packages running the same rules
// are one thing to report, and which package it was is the only difference.
//
// See [ADR-PerPackageConfiguration](../../docs/ADRs/ADR-PerPackageConfiguration.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageRoster {
    pub package: String,
    pub applied: Vec<String>,
    pub skipped: Vec<String>,
    pub unconfigured: Vec<(String, String)>,
}

impl PackageRoster {
    pub fn new(
        package: &str,
        applied: Vec<String>,
        skipped: Vec<String>,
        unconfigured: Vec<(String, String)>,
    ) -> Self {
        Self {
            package: package.to_string(),
            applied,
            skipped,
            unconfigured,
        }
    }

    pub fn agrees_with(&self, other: &Self) -> bool {
        self.applied == other.applied
            && self.skipped == other.skipped
            && self.unconfigured == other.unconfigured
    }

    // Why a rule is missing, not merely that it is. A rule nobody asked for and
    // a rule that could not run are different facts, and a reader who cannot
    // tell them apart has no idea which one to act on.
    pub fn absences(&self) -> Vec<String> {
        self.skipped
            .iter()
            .map(|name| format!("{name} (skipped)"))
            .chain(
                self.unconfigured
                    .iter()
                    .map(|(name, requirement)| format!("{name} ({requirement})")),
            )
            .collect()
    }
}
