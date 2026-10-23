// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A trait or type a file declares, and the line it is declared on.
//
// The line is the declaration's, because the declaration is what a reader
// renames or moves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredItem {
    name: String,
    line: usize,
}

impl DeclaredItem {
    pub fn new(name: &str, line: usize) -> Self {
        Self {
            name: name.to_string(),
            line,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
