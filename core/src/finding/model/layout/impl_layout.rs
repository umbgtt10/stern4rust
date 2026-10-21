// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::layout::fix_refusal::FixRefusal;
use crate::finding::model::layout::impl_member::ImplMember;

// One inherent impl block: the type it is for, the line naming it, the lines of
// its braces, its members as written, and why --fix could not reorder it, if it
// could not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplLayout {
    pub owner: String,
    pub line: usize,
    pub open_line: usize,
    pub close_line: usize,
    pub members: Vec<ImplMember>,
    pub refusal: Option<FixRefusal>,
}

impl ImplLayout {
    // The first position at which the members as written differ from the
    // members in order.
    pub fn first_misplaced(&self) -> Option<usize> {
        self.ordered()
            .iter()
            .zip(&self.members)
            .position(|(belongs, written)| belongs.first_line != written.first_line)
    }

    pub fn ordered(&self) -> Vec<&ImplMember> {
        let mut ordered: Vec<&ImplMember> = self.members.iter().collect();
        ordered.sort_by(|left, right| left.order(right));
        ordered
    }
}
