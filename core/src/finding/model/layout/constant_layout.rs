// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::layout::fix_refusal::FixRefusal;
use crate::finding::model::layout::misplaced_constant::MisplacedConstant;

// A file's constants declared after its first item that is neither an import nor
// a constant -- the leader -- and where they would go: before
// `insertion_line`, the line after the item that ends the constants group, or
// the leader's own first line when nothing precedes it. `follows_constant` says
// whether that group already holds a constant, so a moved one joins it with no
// blank line between.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantLayout {
    leader: String,
    insertion_line: usize,
    follows_constant: bool,
    misplaced: Vec<MisplacedConstant>,
    refusal: Option<FixRefusal>,
}

impl ConstantLayout {
    pub fn new(
        leader: String,
        insertion_line: usize,
        follows_constant: bool,
        misplaced: Vec<MisplacedConstant>,
        refusal: Option<FixRefusal>,
    ) -> Self {
        Self {
            leader,
            insertion_line,
            follows_constant,
            misplaced,
            refusal,
        }
    }

    pub fn follows_constant(&self) -> bool {
        self.follows_constant
    }

    pub fn insertion_line(&self) -> usize {
        self.insertion_line
    }

    pub fn leader(&self) -> &str {
        &self.leader
    }

    pub fn misplaced(&self) -> &[MisplacedConstant] {
        &self.misplaced
    }

    // Whether a line leaves its place: it belongs to one of the misplaced
    // constants' blocks.
    pub fn moves(&self, line: usize) -> bool {
        self.misplaced.iter().any(|constant| constant.holds(line))
    }

    pub fn refusal(&self) -> Option<FixRefusal> {
        self.refusal
    }
}
