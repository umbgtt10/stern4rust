// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::run_failure::RunFailure;
use crate::source_path::SourcePath;
use crate::traits::source_tree::SourceTree;

// Reads the expected header from the tree.
//
// Trailing blank lines are dropped so the file can end with a newline -- every
// editor adds one -- without the rule then demanding a blank line at the top of
// every source file. The same normalisation SourceFile applies is applied here,
// so a header file saved with CRLF or a byte order mark still matches sources
// that were not.
pub struct HeaderSource;

impl HeaderSource {
    pub fn parse(contents: &str) -> Vec<String> {
        let contents = contents.strip_prefix('\u{feff}').unwrap_or(contents);
        let mut lines: Vec<String> = contents
            .split('\n')
            .map(|line| line.strip_suffix('\r').unwrap_or(line).to_string())
            .collect();
        while lines.last().is_some_and(|line| line.trim().is_empty()) {
            lines.pop();
        }
        lines
    }

    pub fn read(
        source_tree: &dyn SourceTree,
        path: &SourcePath,
    ) -> Result<Vec<String>, RunFailure> {
        let contents = source_tree
            .read_source(path)
            .map_err(RunFailure::HeaderUnreadable)?;
        Ok(Self::parse(&contents))
    }
}
