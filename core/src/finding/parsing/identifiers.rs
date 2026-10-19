// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use proc_macro2::TokenTree;

// The identifiers a stretch of source uses, however deeply its groups nest.
//
// Read off the tokens rather than the syntax tree, because the question is only
// whether a name appears, and every expression, type and pattern a function can
// hold would otherwise need a visit of its own. A literal is not an identifier,
// so a string spelling a type's name does not name it.
pub struct Identifiers;

impl Identifiers {
    pub fn of(tokens: TokenStream) -> BTreeSet<String> {
        tokens
            .into_iter()
            .flat_map(|tree| match tree {
                TokenTree::Ident(identifier) => BTreeSet::from([identifier.to_string()]),
                TokenTree::Group(group) => Self::of(group.stream()),
                _ => BTreeSet::new(),
            })
            .collect()
    }
}
