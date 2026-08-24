// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;

// Reads the expected header off disk.
//
// Trailing blank lines are dropped so the file can end with a newline -- every
// editor adds one -- without the rule then demanding a blank line at the top of
// every source file. The same normalisation SourceFile applies is applied here,
// so a header file saved with CRLF or a byte order mark still matches sources
// that were not.
pub struct HeaderSource;

impl HeaderSource {
    pub fn read(path: &Path) -> Result<Vec<String>> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read header file {}", path.display()))?;
        Ok(Self::parse(&contents))
    }

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
}
