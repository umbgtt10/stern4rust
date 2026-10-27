// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::source_path::SourcePath;

// What the exclusions did, as well as what survived them.
//
// The count per pattern is carried rather than a single total, because a
// pattern that matched **nothing** is the interesting case and a total would
// hide it. A stale exclude left in a config file after the tree it named was
// deleted still looks like it is doing something, and the run it silences is
// indistinguishable from a run that had nothing to say -- which is the same
// failure this tool refuses everywhere else.
pub struct ExclusionOutcome {
    kept: Vec<SourcePath>,
    excluded: Vec<(String, usize)>,
}

impl ExclusionOutcome {
    pub fn new(kept: Vec<SourcePath>, excluded: Vec<(String, usize)>) -> Self {
        Self { kept, excluded }
    }

    pub fn excluded(&self) -> &[(String, usize)] {
        &self.excluded
    }

    pub fn excluded_total(&self) -> usize {
        self.excluded.iter().map(|(_, count)| count).sum()
    }

    pub fn into_parts(self) -> (Vec<SourcePath>, Vec<(String, usize)>) {
        (self.kept, self.excluded)
    }

    pub fn kept(&self) -> &[SourcePath] {
        &self.kept
    }

    // The patterns that turned out to cover nothing, which the report names so
    // that a dead exclusion can be deleted rather than trusted.
    pub fn unmatched_patterns(&self) -> Vec<&str> {
        self.excluded
            .iter()
            .filter(|(_, count)| *count == 0)
            .map(|(pattern, _)| pattern.as_str())
            .collect()
    }
}
