// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::ItemMod;

// The items an inline module holds.
//
// Every walker that descends into `mod a { ... }` needs this one line, and five
// of them -- `ImplementedTypeFinder`, `PureTraitsRule`, `DeclaredByNameRule`,
// `TestFileNamePostfixRule` and `ReturnedMutationsRule` -- each carried a private
// copy until dry4rust's gate counted the fifth. What each walker does with the
// items stays with it.
pub struct InlineModule;

impl InlineModule {
    // None for `mod a;`: its items live in another file, which the walker
    // reaches when it reads that file.
    pub fn items(module: &ItemMod) -> Option<&[Item]> {
        module.content.as_ref().map(|(_, items)| items.as_slice())
    }
}
