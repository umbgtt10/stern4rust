// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::leading_constants_rewriter::LeadingConstantsRewriter;
use crate::ordered_impl_functions_rewriter::OrderedImplFunctionsRewriter;
use crate::source_file::SourceFile;
use crate::source_rewriter::SourceRewriter;
use crate::test_file_rewriter::TestFileRewriter;

// The fixers --fix runs, in turn: each one whose rule the package applies, each
// handed what the one before left. Two of them can repair one source file --
// its constants and its impls -- and composing is what keeps the second from
// working on, and writing back, the file as it was before the first.
pub struct RewriterRegistry {
    rewriters: Vec<Box<dyn SourceRewriter>>,
}

impl RewriterRegistry {
    pub fn all() -> Self {
        Self::new(vec![
            Box::new(TestFileRewriter::new()),
            Box::new(LeadingConstantsRewriter::new()),
            Box::new(OrderedImplFunctionsRewriter::new()),
        ])
    }

    pub fn new(rewriters: Vec<Box<dyn SourceRewriter>>) -> Self {
        Self { rewriters }
    }

    // None when no fixer the package applies changes the file.
    pub fn rewrite(&self, file: &SourceFile, applied: &[&'static str]) -> Option<String> {
        self.rewriters
            .iter()
            .filter(|rewriter| applied.contains(&rewriter.rule()))
            .fold(None, |rewritten: Option<String>, rewriter| {
                let current = rewritten
                    .as_deref()
                    .map(|contents| SourceFile::new(file.relative_path(), contents));
                rewriter
                    .rewrite(current.as_ref().unwrap_or(file))
                    .or(rewritten)
            })
    }
}
