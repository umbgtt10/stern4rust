// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::offence::Offence;

// How much of the report is printed.
//
// A first run against a large codebase can find a thousand offences, and a
// thousand rows is not a report -- it is a wall that gets scrolled past. The
// default shows the first hundred, which is roughly what somebody will act on
// before re-running anyway.
//
// The cap is on what is SHOWN and never on what is counted. A summary that said
// a hundred when the tree holds a thousand would be a quietly wrong report, and
// it would be this tool producing it. The full total stays in the summary, the
// omitted count is stated outright, and the exit code is decided from every
// offence rather than from the printed ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OffenceThreshold {
    limit: usize,
}

impl OffenceThreshold {
    pub const DEFAULT: usize = 100;

    pub fn new(limit: usize) -> Self {
        Self { limit }
    }

    pub fn limit(self) -> usize {
        self.limit
    }

    // Zero is the escape hatch for "show me everything", not a way to silence
    // the report. A limit of nothing would be a tool that finds problems and
    // then refuses to say which.
    pub fn is_unlimited(self) -> bool {
        self.limit == 0
    }

    // The offences arrive sorted by file then line, so what survives is whole
    // files from the top rather than a scattering across the tree. A reader
    // fixes what is shown, re-runs, and gets the next file.
    pub fn kept(self, offences: &[Offence]) -> &[Offence] {
        if self.is_unlimited() {
            return offences;
        }
        &offences[..self.limit.min(offences.len())]
    }

    pub fn omitted(self, offences: &[Offence]) -> usize {
        offences.len() - self.kept(offences).len()
    }
}

impl Default for OffenceThreshold {
    fn default() -> Self {
        Self::new(Self::DEFAULT)
    }
}
