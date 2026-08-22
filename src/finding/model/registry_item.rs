// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One item that does not belong in a registry file, carrying the two things an
// offence needs to be worth printing: where it is, and what it is.
//
// Both halves matter together. A registry with four strays used to produce four
// rows that were byte-identical and all pointed at line 1 -- a report that
// claimed a file had four problems while naming none of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryItem {
    pub line: usize,
    pub label: String,
}

impl RegistryItem {
    pub fn new(line: usize, label: &str) -> Self {
        Self {
            line,
            label: label.to_string(),
        }
    }
}
