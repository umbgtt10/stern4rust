// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::offence::Offence;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// Every .rs file can be read and parsed.
//
// This rule exists because silence is indistinguishable from success. Every
// other rule that parses gives up quietly on source it cannot read, trusting
// rustc to say so more clearly -- which is right for a file somebody is
// actively editing and wrong for a file nobody is looking at. A corrupted file
// disappears from the report entirely and the package looks cleaner than it is.
//
// Not hypothetical: during development a test file became a run of NUL bytes
// and the tool reported one fewer offence than the tree contained, with nothing
// to indicate anything had been skipped.
pub struct ReadableSourceRule;

impl ReadableSourceRule {
    // Shared with SourceReader, which reports the same rule for a file that
    // could not be read at all -- there is no SourceFile to hand a rule in that
    // case, but it is the same finding about the same tree.
    pub const NAME: &'static str = "readable-source";

    pub fn new() -> Self {
        Self
    }

    // A parse error's span can be the call site rather than a real position, so
    // line 0 is reported as line 1 instead of as a line no editor can go to.
    fn line_of(error: &syn::Error) -> usize {
        error.span().start().line.max(1)
    }
}

impl Default for ReadableSourceRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ReadableSourceRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        match syn::parse_file(&file.contents()) {
            Ok(_) => Vec::new(),
            Err(error) => vec![
                Offence::new(
                    file.relative_path(),
                    Self::line_of(&error),
                    self.name(),
                    format!("file does not parse as Rust: {error}"),
                    "correct the syntax error rustc reports for this file, or restore \
                     the file if it is corrupted"
                        .to_string(),
                )
                .with_subject(file.relative_path()),
            ],
        }
    }
}
