// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::public_entry_point::PublicEntryPoint;

// A method in an inherent impl whose visibility is neither `pub` nor private:
// how a call names it, the line naming it, the form it was written in --
// `pub(crate)`, `pub(super)`, `pub(self)`, `pub(in ...)` -- and the type it
// belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestrictedMethod {
    pub call: PublicEntryPoint,
    pub line: usize,
    pub visibility: String,
    pub owner: String,
}

impl RestrictedMethod {
    pub const PRIVATE_SPELLED_LONG: &'static str = "pub(self)";

    // `pub(self)` restricts a method to its own module, which is what `fn`
    // already means.
    pub fn is_private_already(&self) -> bool {
        self.visibility == Self::PRIVATE_SPELLED_LONG
    }
}
