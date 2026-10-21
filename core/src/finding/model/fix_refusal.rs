// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Why --fix leaves a file's constants where they are. Moving them is pure layout
// only while nothing in the file depends on where an item is written, or on
// which lines belong to which item, and each of these is a case where the fixer
// cannot be sure of that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixRefusal {
    MacroRules,
    OpeningComment,
    SharedLine,
}

impl FixRefusal {
    pub fn reason(self) -> &'static str {
        match self {
            Self::MacroRules => {
                "a file with a `macro_rules!`, whose textual order decides what it can call"
            }
            Self::OpeningComment => {
                "a file whose opening comment runs into its first item, where the header cannot be \
                 told from that item's own comment"
            }
            Self::SharedLine => "a constant that shares a line with another item",
        }
    }
}
