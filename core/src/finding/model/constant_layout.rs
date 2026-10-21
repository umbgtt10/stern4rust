// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::fix_refusal::FixRefusal;
use crate::finding::model::misplaced_constant::MisplacedConstant;

// A file's constants declared after its first item that is neither an import nor
// a constant -- the leader -- and where they would go: before
// `insertion_line`, the line after the item that ends the constants group, or
// the leader's own first line when nothing precedes it. `follows_constant` says
// whether that group already holds a constant, so a moved one joins it with no
// blank line between.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantLayout {
    pub leader: String,
    pub insertion_line: usize,
    pub follows_constant: bool,
    pub misplaced: Vec<MisplacedConstant>,
    pub refusal: Option<FixRefusal>,
}

impl ConstantLayout {
    // Whether a line leaves its place: it belongs to one of the misplaced
    // constants' blocks.
    pub fn moves(&self, line: usize) -> bool {
        self.misplaced.iter().any(|constant| constant.holds(line))
    }
}
