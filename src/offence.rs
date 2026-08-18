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

    // Offences are found in whatever order the rules happen to run, which puts
    // every tree-wide one after every per-file one. Grouping by file and then by
    // line is what lets a reader -- or a tool consuming the report -- work
    // through one file at a time instead of jumping between them.
    pub fn sort_key(&self) -> (&str, usize, &'static str) {
        (&self.file, self.line, self.rule)
    }
}
