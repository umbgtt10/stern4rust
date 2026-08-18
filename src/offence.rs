// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fmt::Debug;

use serde::Serialize;

// One thing wrong with one file. Every rule reports in this currency, so the
// report is a single table rather than one section per rule, and a new rule
// costs nothing in the printer.
//
// `subject` and `expected` are what make the JSON report worth consuming rather
// than only reading. The description is a sentence for a person; the subject is
// the thing the offence is about, and `expected` is the correct text where the
// rule knows it. A rule opts into both, so a rule with nothing precise to add
// says nothing rather than repeating its own prose in another field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Offence {
    pub file: String,
    pub line: usize,
    pub rule: &'static str,
    pub description: String,
    pub subject: Option<String>,
    pub expected: Option<String>,
}

impl Offence {
    pub fn new(file: &str, line: usize, rule: &'static str, description: String) -> Self {
        Self {
            file: file.to_string(),
            line,
            rule,
            description,
            subject: None,
            expected: None,
        }
    }

    pub fn with_subject(self, subject: &str) -> Self {
        Self {
            subject: Some(subject.to_string()),
            ..self
        }
    }

    pub fn with_expected(self, expected: &str) -> Self {
        Self {
            expected: Some(expected.to_string()),
            ..self
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
