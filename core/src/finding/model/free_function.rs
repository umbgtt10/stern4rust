// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

// A function that belongs to no impl block: its name, the line it is declared
// on, and the identifiers it uses.
//
// The identifiers are what tells a helper of the file's subject from a stranger:
// a function whose signature or body names the subject is the subject's
// business, and one that never does may be a subject of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeFunction {
    pub name: String,
    pub line: usize,
    uses: BTreeSet<String>,
}

impl FreeFunction {
    pub fn new(name: String, line: usize, uses: BTreeSet<String>) -> Self {
        Self { name, line, uses }
    }

    pub fn names(&self, identifier: &str) -> bool {
        self.uses.contains(identifier)
    }
}
