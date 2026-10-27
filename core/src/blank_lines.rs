// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Lines with the blank ones at one end dropped. Where moved lines meet the lines
// that stay, a fixer drops the blanks on both sides and puts back exactly one,
// so a gap never grows or vanishes with a move.
use alloc::vec::Vec;

pub struct BlankLines;

impl BlankLines {
    pub fn without_leading<T: AsRef<str> + Clone>(lines: &[T]) -> Vec<T> {
        lines
            .iter()
            .skip_while(|line| line.as_ref().trim().is_empty())
            .cloned()
            .collect()
    }

    pub fn without_trailing<T: AsRef<str> + Clone>(lines: &[T]) -> Vec<T> {
        let end = lines
            .iter()
            .rposition(|line| !line.as_ref().trim().is_empty())
            .map_or(0, |last| last + 1);
        lines[..end].to_vec()
    }
}
