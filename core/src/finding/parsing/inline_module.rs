// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::iter::once;

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
    // Every item, an inline module and then what it holds, in source order. A
    // walker after one kind of item filters this rather than writing the
    // descent again: ImplBlocks and PureTraitsRule each carried it.
    pub fn flatten(items: &[Item]) -> Vec<&Item> {
        items
            .iter()
            .flat_map(|item| {
                let nested = match item {
                    Item::Mod(module) => Self::items(module).map(Self::flatten).unwrap_or_default(),
                    _ => Vec::new(),
                };
                once(item).chain(nested)
            })
            .collect()
    }

    // None for `mod a;`: its items live in another file, which the walker
    // reaches when it reads that file.
    pub fn items(module: &ItemMod) -> Option<&[Item]> {
        module.content.as_ref().map(|(_, items)| items.as_slice())
    }
}
