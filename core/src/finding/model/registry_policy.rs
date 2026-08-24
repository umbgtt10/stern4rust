// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::Visibility;

// What counts as a declaration, which is not the same question in both trees.
//
// Under tests/, being compiled is the whole concern: a private `mod name;`
// compiles that file as well as a public one, so it is a declaration and
// tests-layout accepts it. Under src/, the module tree is the crate's shape,
// and a registry that hides part of it behind a private `mod` is describing
// something other than what the crate exports.
//
// `extern crate alloc;` is the one non-mod item a source registry may hold. A
// no_std crate has to say it somewhere and the crate root is where it belongs.
// No other extern crate has that excuse, and in 2018-and-later Rust the form is
// otherwise a relic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryPolicy {
    requires_pub: bool,
    allows_extern_alloc: bool,
}

impl RegistryPolicy {
    pub const ALLOC: &'static str = "alloc";

    pub fn tests() -> Self {
        Self {
            requires_pub: false,
            allows_extern_alloc: false,
        }
    }

    pub fn source() -> Self {
        Self {
            requires_pub: true,
            allows_extern_alloc: true,
        }
    }

    pub fn is_declaration(&self, item: &Item) -> bool {
        match item {
            Item::Mod(module) => module.content.is_none() && self.visibility_allows(&module.vis),
            Item::ExternCrate(external) => {
                self.allows_extern_alloc && external.ident == Self::ALLOC
            }
            _ => false,
        }
    }

    fn visibility_allows(&self, visibility: &Visibility) -> bool {
        !self.requires_pub || matches!(visibility, Visibility::Public(_))
    }
}
