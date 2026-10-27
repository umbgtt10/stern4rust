// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::reporting::offence::Offence;
use crate::reporting::offence_fingerprint::OffenceFingerprint;
use crate::settings::adoption::baseline_outcome::BaselineOutcome;

// The offences a repository has already agreed to live with.
//
// `--rule` gives a codebase a way in by enforcing one rule at a time. What it
// cannot express is "every rule, against new code only", which is what a
// codebase with six hundred existing offences actually needs -- otherwise the
// choice is between a gate that fails forever and no gate at all.
//
// Counts, not a set. Two identical offences in one file share a fingerprint, so
// the baseline records that there were two: fixing one and introducing another
// leaves the total unchanged and must still pass, while introducing a third
// must not.
//
// Sorted on the way out, because this file is checked in and a diff that
// reorders itself between runs is a diff nobody reviews.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Baseline {
    offences: BTreeMap<String, usize>,
}

impl Baseline {
    pub fn new(offences: BTreeMap<String, usize>) -> Self {
        Self { offences }
    }

    pub fn of(offences: &[Offence]) -> Self {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for offence in offences {
            *counts.entry(OffenceFingerprint::of(offence)).or_default() += 1;
        }
        Self { offences: counts }
    }

    // Each fingerprint is forgiven up to the number of times the baseline
    // recorded it, and every occurrence beyond that is reported. Which of two
    // identical offences is forgiven does not matter -- they differ only by
    // line, and the one that survives carries a real line either way.
    pub fn apply(&self, offences: Vec<Offence>) -> BaselineOutcome {
        let mut remaining = self.offences.clone();
        let mut kept = Vec::new();
        let mut suppressed = 0;
        for offence in offences {
            let fingerprint = OffenceFingerprint::of(&offence);
            match remaining.get_mut(&fingerprint) {
                Some(count) if *count > 0 => {
                    *count -= 1;
                    suppressed += 1;
                }
                _ => kept.push(offence),
            }
        }
        BaselineOutcome::new(kept, suppressed, remaining.values().sum())
    }

    // Each fingerprint with the number of times it was recorded, in the order a
    // stored baseline lists them.
    pub fn counts(&self) -> &BTreeMap<String, usize> {
        &self.offences
    }

    pub fn is_empty(&self) -> bool {
        self.offences.is_empty()
    }

    pub fn len(&self) -> usize {
        self.offences.values().sum()
    }
}
