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
    owner: String,
    line: usize,
    open_line: usize,
    close_line: usize,
    members: Vec<ImplMember>,
    refusal: Option<FixRefusal>,
}

impl ImplLayout {
    pub fn new(
        owner: String,
        line: usize,
        open_line: usize,
        close_line: usize,
        members: Vec<ImplMember>,
        refusal: Option<FixRefusal>,
    ) -> Self {
        Self {
            owner,
            line,
            open_line,
            close_line,
            members,
            refusal,
        }
    }

    pub fn close_line(&self) -> usize {
        self.close_line
    }

    // The first position at which the members as written differ from the
    // members in order.
    pub fn first_misplaced(&self) -> Option<usize> {
        self.ordered()
            .iter()
            .zip(&self.members)
            .position(|(belongs, written)| belongs.first_line() != written.first_line())
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn members(&self) -> &[ImplMember] {
        &self.members
    }

    pub fn open_line(&self) -> usize {
        self.open_line
    }

    pub fn ordered(&self) -> Vec<&ImplMember> {
        let mut ordered: Vec<&ImplMember> = self.members.iter().collect();
        ordered.sort_by(|left, right| left.order(right));
        ordered
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn refusal(&self) -> Option<FixRefusal> {
        self.refusal
    }
}
