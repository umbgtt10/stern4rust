// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A module-level constant or static declared after an item that is neither an
// import nor a constant: its name, the line naming it, and the lines that move
// with it -- its attributes and the comment directly above it included.
use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MisplacedConstant {
    name: String,
    line: usize,
    first_line: usize,
    last_line: usize,
}

impl MisplacedConstant {
    pub fn new(name: String, line: usize, first_line: usize, last_line: usize) -> Self {
        Self {
            name,
            line,
            first_line,
            last_line,
        }
    }

    pub fn first_line(&self) -> usize {
        self.first_line
    }

    pub fn holds(&self, line: usize) -> bool {
        (self.first_line..=self.last_line).contains(&line)
    }

    pub fn last_line(&self) -> usize {
        self.last_line
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
