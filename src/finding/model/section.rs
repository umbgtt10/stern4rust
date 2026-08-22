// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The four groups a test file is made of, in the order they appear.
//
// Helpers are defined by exclusion -- whatever is neither an import, nor a
// constant, nor a test. That is what makes the set closed: a `struct`, an
// `impl`, a type alias and a plain `fn` are all helpers, so a new kind of item
// lands somewhere sensible without the rule needing to learn about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
    Imports,
    Constants,
    Helpers,
    Tests,
}

impl Section {
    // Reads as the thing itself, so an offence says "a constant follows a
    // helper" rather than naming an enum variant at the reader.
    pub fn label(self) -> &'static str {
        match self {
            Self::Imports => "import",
            Self::Constants => "constant",
            Self::Helpers => "helper",
            Self::Tests => "test",
        }
    }

    // Imports are the one group written without gaps. Everything else is
    // separated by exactly one blank line.
    pub fn blank_lines_between_entries(self) -> usize {
        match self {
            Self::Imports => 0,
            _ => 1,
        }
    }
}
