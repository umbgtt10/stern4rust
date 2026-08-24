// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One place in the production source tree where a test, or the machinery of a
// test, is living.
//
// The correction is built here rather than by the rule because the finder is
// the only thing that knows which of the two offences it found: a test belongs
// in the mirrored test file, while a conditional attribute has to stop being
// conditional. One instruction would have to cover both, and would say nothing
// useful about either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitTestSite {
    pub line: usize,
    pub label: String,
    pub correction: String,
}

impl UnitTestSite {
    pub fn new(line: usize, label: &str, correction: &str) -> Self {
        Self {
            line,
            label: label.to_string(),
            correction: correction.to_string(),
        }
    }
}
