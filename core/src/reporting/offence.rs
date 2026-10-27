// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::string::ToString;
use core::fmt::Debug;

use crate::reporting::offence_order::OffenceOrder;

// One thing wrong with one file. Every rule reports in this currency, so the
// report is a single table rather than one section per rule, and a new rule
// costs nothing in the printer.
//
// `subject` and `expected` are what make the JSON report worth consuming rather
// than only reading. The description is a sentence for a person; the subject is
// the thing the offence is about, and `expected` is the correct text where the
// rule knows it. A rule opts into both, so a rule with nothing precise to add
// says nothing rather than repeating its own prose in another field.
// `correction` is required rather than optional, and that is the point. A rule
// that can say what is wrong can say what to do about it, and making the field
// optional would let a future rule quietly omit the half of the report that is
// worth acting on -- the exact shape of silent gap this tool exists to catch.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Offence {
    file: String,
    line: usize,
    rule: &'static str,
    description: String,
    correction: String,
    subject: Option<String>,
    expected: Option<String>,
}

impl Offence {
    pub fn new(
        file: &str,
        line: usize,
        rule: &'static str,
        description: String,
        correction: String,
    ) -> Self {
        Self {
            file: file.to_string(),
            line,
            rule,
            description,
            correction,
            subject: None,
            expected: None,
        }
    }

    pub fn correction(&self) -> &str {
        &self.correction
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn expected(&self) -> Option<&str> {
        self.expected.as_deref()
    }

    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn rule(&self) -> &'static str {
        self.rule
    }

    // Offences are found in whatever order the rules happen to run, which puts
    // every tree-wide one after every per-file one. Grouping by file and then by
    // line is what lets a reader -- or a tool consuming the report -- work
    // through one file at a time instead of jumping between them.
    pub fn sort_key(&self) -> OffenceOrder<'_> {
        OffenceOrder {
            file: &self.file,
            line: self.line,
            rule: self.rule,
        }
    }

    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    pub fn with_expected(self, expected: &str) -> Self {
        Self {
            expected: Some(expected.to_string()),
            ..self
        }
    }

    pub fn with_file(self, file: &str) -> Self {
        Self {
            file: file.to_string(),
            ..self
        }
    }

    pub fn with_subject(self, subject: &str) -> Self {
        Self {
            subject: Some(subject.to_string()),
            ..self
        }
    }
}
