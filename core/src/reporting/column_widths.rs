// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::ToString;

use crate::reporting::offence::Offence;

// How wide each column of the table has to be.
//
// Sized to the contents rather than fixed, so a rule name or a path that grows
// widens its column instead of overflowing it -- and never narrower than its own
// heading, which is what keeps the header row aligned with a report of one short
// offence.
pub struct ColumnWidths {
    file: usize,
    line: usize,
    rule: usize,
    description: usize,
}

impl ColumnWidths {
    pub fn of(offences: &[Offence]) -> Self {
        Self {
            file: Self::widest(offences.iter().map(|offence| offence.file().len()), "file"),
            line: Self::widest(
                offences
                    .iter()
                    .map(|offence| offence.line().to_string().len()),
                "line",
            ),
            rule: Self::widest(offences.iter().map(|offence| offence.rule().len()), "rule"),
            description: Self::widest(
                offences.iter().map(|offence| offence.description().len()),
                "offence",
            ),
        }
    }

    pub fn description(&self) -> usize {
        self.description
    }

    pub fn file(&self) -> usize {
        self.file
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn rule(&self) -> usize {
        self.rule
    }

    fn widest<I: Iterator<Item = usize>>(lengths: I, heading: &str) -> usize {
        lengths.max().unwrap_or(0).max(heading.len())
    }
}
