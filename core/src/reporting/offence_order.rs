// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where an offence sits in the report: by file, then by line, then by rule. The
// fields compare in that order, so the key orders exactly as the tuple it
// replaced -- and says which `&str` is the file and which the rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OffenceOrder<'a> {
    file: &'a str,
    line: usize,
    rule: &'static str,
}

impl<'a> OffenceOrder<'a> {
    pub fn new(file: &'a str, line: usize, rule: &'static str) -> Self {
        Self { file, line, rule }
    }

    pub fn file(&self) -> &'a str {
        self.file
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn rule(&self) -> &'static str {
        self.rule
    }
}
