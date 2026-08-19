// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::from_str;
use serde_json::to_string_pretty;
use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::fs::write;
use std::path::Path;

use crate::baseline_outcome::BaselineOutcome;
use crate::offence::Offence;
use crate::offence_fingerprint::OffenceFingerprint;

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
#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Baseline {
    offences: BTreeMap<String, usize>,
}

impl Baseline {
    pub fn of(offences: &[Offence]) -> Self {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for offence in offences {
            *counts.entry(OffenceFingerprint::of(offence)).or_default() += 1;
        }
        Self { offences: counts }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let text = read_to_string(path)
            .with_context(|| format!("{} could not be read", path.display()))?;
        from_str(&text).with_context(|| format!("{} is not a valid baseline", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let text = to_string_pretty(self)
            .with_context(|| format!("{} could not be rendered", path.display()))?;
        write(path, text).with_context(|| format!("{} could not be written", path.display()))
    }

    pub fn len(&self) -> usize {
        self.offences.values().sum()
    }

    pub fn is_empty(&self) -> bool {
        self.offences.is_empty()
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
}
