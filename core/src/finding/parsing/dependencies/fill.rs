// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;

use syn::Expr;

// One field of a type a method body fills, with the value it puts there and the
// line that value starts on: a struct literal's field, a tuple constructor's
// position by its index, or the field of `self.field = ...`.
pub struct Fill {
    pub field: String,
    pub value: Expr,
    pub line: usize,
}
