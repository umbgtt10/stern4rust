// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;

use crate::finding::model::layout::section::Section;

// One top-level item, with the lines it occupies.
//
// first_line is the start of the item's whole block, comments included: a
// comment introducing a test belongs to that test, so the blank line before the
// comment is the separator and the comment itself is not a gap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestFileItem {
    section: Section,
    name: String,
    first_line: usize,
    last_line: usize,
}

impl TestFileItem {
    pub fn new(section: Section, name: String, first_line: usize, last_line: usize) -> Self {
        Self {
            section,
            name,
            first_line,
            last_line,
        }
    }

    pub fn first_line(&self) -> usize {
        self.first_line
    }

    pub fn last_line(&self) -> usize {
        self.last_line
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn section(&self) -> Section {
        self.section
    }

    // Case-insensitive, because a constant is SHOUTED and a function is not, and
    // sorting them by byte value would put every constant before every helper
    // for reasons that have nothing to do with the alphabet.
    pub fn sort_key(&self) -> String {
        self.name.to_lowercase()
    }
}
