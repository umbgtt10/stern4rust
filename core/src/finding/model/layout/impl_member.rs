// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::cmp::Ordering;

use crate::finding::model::layout::member_kind::MemberKind;

// One member of an inherent impl block: its name, a raw identifier without its
// `r#`, its kind, how many parameters it takes, and the lines it moves with --
// its attributes, doc comments and the comment directly above it included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplMember {
    pub name: String,
    pub kind: MemberKind,
    pub parameters: usize,
    pub first_line: usize,
    pub last_line: usize,
}

impl ImplMember {
    // Where it belongs against another: by kind, then constructors by how many
    // parameters they take, then functions by name, byte by byte. Constants and
    // types keep the order written, and so do two functions of one name --
    // `cfg`-gated twins -- since a stable sort leaves equals where they were.
    pub fn order(&self, other: &Self) -> Ordering {
        self.kind.cmp(&other.kind).then_with(|| match self.kind {
            MemberKind::Head => Ordering::Equal,
            MemberKind::Constructor => self
                .parameters
                .cmp(&other.parameters)
                .then_with(|| self.name.cmp(&other.name)),
            MemberKind::Public | MemberKind::Other => self.name.cmp(&other.name),
        })
    }
}
