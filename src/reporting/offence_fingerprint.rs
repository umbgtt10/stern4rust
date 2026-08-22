// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reporting::offence::Offence;

// What makes two offences the same offence across two runs.
//
// The line number is deliberately **not** part of it. An offence that moves
// because somebody added an import above it is the same offence, and a baseline
// keyed on the line would go stale on the first unrelated edit -- which would
// make it useless exactly when it is most needed, on a codebase under active
// change.
//
// The file, the rule and **the subject** are what is left. The subject is the
// thing the offence is about, named -- `\u{60}ColumnWidths\u{60}`, `\u{60}src/a.rs\u{60}`, a test's
// name -- so two different offences of one rule in one file stay distinct while
// the same offence at a new line stays the same.
//
// The description used to stand in for the subject, from before `Offence`
// carried one. That made **rule descriptions part of a published interface
// without anything saying so**: a rule whose sentence gained a word reported its
// offences as new across every repository that had baselined them, all at once,
// and the only signal was a spike in the stale-entry count. The subject survives
// a rewrite of the sentence around it, which is the whole point.
//
// The description is still the fallback for an offence with no subject. Only
// `header` emits one, and it reports once per file, so the file and the rule
// already tell those apart.
//
// Duplicates are counted rather than deduplicated. Two identical
// `syn::parse_file` calls in one file share a fingerprint, so the baseline
// records how many there were: fixing one and adding another must not pass.
pub struct OffenceFingerprint;

impl OffenceFingerprint {
    pub const SEPARATOR: &'static str = "\u{1f}";

    pub fn of(offence: &Offence) -> String {
        format!(
            "{}{}{}{}{}",
            offence.file,
            Self::SEPARATOR,
            offence.rule,
            Self::SEPARATOR,
            offence.subject.as_ref().unwrap_or(&offence.description)
        )
    }
}
