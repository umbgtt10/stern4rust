// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fmt::Debug;

// One thing wrong with one file. Every rule reports in this currency, so the
// report is a single table rather than one section per rule, and a new rule
// costs nothing in the printer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offence {
    pub file: String,
    pub line: usize,
    pub rule: &'static str,
    pub description: String,
}

impl Offence {
    pub fn new(file: &str, line: usize, rule: &'static str, description: String) -> Self {
        Self {
            file: file.to_string(),
            line,
            rule,
            description,
        }
    }
}
