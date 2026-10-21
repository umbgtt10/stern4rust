// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What a member of an inherent impl is, in the order the kinds are listed:
// constants and types, constructors, `pub` functions, then the rest -- private
// functions, and restricted ones where public-or-private is not yet adopted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemberKind {
    Head,
    Constructor,
    Public,
    Other,
}
