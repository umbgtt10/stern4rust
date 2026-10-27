// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use crate::reporting::offence::Offence;

// What a baseline let through, and what it hid.
//
// The suppressed count travels with the surviving offences because a run that
// reported nothing while hiding four hundred findings would be the most
// comfortable lie this tool could tell. `baselined=N` goes in the summary of
// every run that used one.
//
// Stale entries are carried for the same reason a dead `--exclude` pattern is
// named: an entry matching nothing describes an offence somebody has since
// fixed, and until the baseline is rewritten it is dead weight that makes the
// file look like it is still holding something back.
pub struct BaselineOutcome {
    kept: Vec<Offence>,
    suppressed: usize,
    stale: usize,
}

impl BaselineOutcome {
    pub fn new(kept: Vec<Offence>, suppressed: usize, stale: usize) -> Self {
        Self {
            kept,
            suppressed,
            stale,
        }
    }

    pub fn into_kept(self) -> Vec<Offence> {
        self.kept
    }

    pub fn is_stale(&self) -> bool {
        self.stale > 0
    }

    pub fn kept(&self) -> &[Offence] {
        &self.kept
    }

    pub fn stale(&self) -> usize {
        self.stale
    }

    pub fn suppressed(&self) -> usize {
        self.suppressed
    }
}
