// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A module-level constant or static declared after an item that is neither an
// import nor a constant: its name, the line naming it, and the lines that move
// with it -- its attributes and the comment directly above it included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MisplacedConstant {
    pub name: String,
    pub line: usize,
    pub first_line: usize,
    pub last_line: usize,
}

impl MisplacedConstant {
    pub fn holds(&self, line: usize) -> bool {
        (self.first_line..=self.last_line).contains(&line)
    }
}
