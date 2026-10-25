// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Expr;

// A call from a method body to another associated function of its own type --
// `Self::with_deps(...)` -- and what it hands over, in order.
pub struct Delegation {
    pub target: String,
    pub arguments: Vec<Expr>,
}
