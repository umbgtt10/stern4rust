// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::offence::Offence;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// Every .rs file opens with the repository's header.
//
// The expected text is data rather than a constant, because it is not the same
// twice: this repository is MIT, the etheram repositories are Apache 2.0, the
// year moves, and another codebase will have something else entirely. A rule
// that hardcoded one header would be right for exactly one repository.
//
// Only the first divergence is reported. A file whose header is missing
// altogether would otherwise produce one offence per header line, burying every
// other file in the report behind it.
pub struct HeaderRule {
    expected: Vec<String>,
}

impl HeaderRule {
    pub fn new(expected: Vec<String>) -> Self {
        Self { expected }
    }

    pub fn expected(&self) -> &[String] {
        &self.expected
    }

    fn missing_entirely(&self, file: &SourceFile) -> Offence {
        Offence::new(
            file.relative_path(),
            1,
            self.name(),
            "file is empty, so it carries no header".to_string(),
        )
        .with_expected(&self.expected.join(
            "
",
        ))
    }

    fn ends_early(&self, file: &SourceFile) -> Offence {
        Offence::new(
            file.relative_path(),
            file.lines().len(),
            self.name(),
            format!(
                "file has {} lines but the header is {}",
                file.lines().len(),
                self.expected.len()
            ),
        )
        .with_expected(&self.expected.join(
            "
",
        ))
    }

    fn line_differs(&self, file: &SourceFile, index: usize) -> Offence {
        Offence::new(
            file.relative_path(),
            index + 1,
            self.name(),
            format!(
                "expected {:?} but found {:?}",
                self.expected[index],
                file.lines()[index]
            ),
        )
        .with_expected(&self.expected.join(
            "
",
        ))
    }
}

impl Rule for HeaderRule {
    fn name(&self) -> &'static str {
        "header"
    }

    // The overlap is compared before the length, so a file whose very first line
    // is wrong is reported at line 1 rather than at its end. Checking length
    // first would tell a file with no header at all that it was "too short",
    // which is true and useless -- the actionable fact is that line 1 is not the
    // header.
    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if self.expected.is_empty() {
            return Vec::new();
        }
        if file.is_empty() {
            return vec![self.missing_entirely(file)];
        }
        let overlap = self.expected.len().min(file.lines().len());
        if let Some(index) =
            (0..overlap).find(|index| file.lines()[*index] != self.expected[*index])
        {
            return vec![self.line_differs(file, index)];
        }
        if file.lines().len() < self.expected.len() {
            return vec![self.ends_early(file)];
        }
        Vec::new()
    }
}
