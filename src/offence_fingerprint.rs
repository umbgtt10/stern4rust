// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::offence::Offence;

// What makes two offences the same offence across two runs.
//
// The line number is deliberately **not** part of it. An offence that moves
// because somebody added an import above it is the same offence, and a baseline
// keyed on the line would go stale on the first unrelated edit -- which would
// make it useless exactly when it is most needed, on a codebase under active
// change.
//
// The file, the rule and the description are what is left. The description
// carries the subject (`\u{60}ColumnWidths\u{60} is a second type...`), so two different
// offences of one rule in one file stay distinct, while the same offence at a
// new line stays the same.
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
            offence.description
        )
    }
}
