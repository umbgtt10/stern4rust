// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::spanned::Spanned;

use crate::finding::model::declared_item::DeclaredItem;
use crate::finding::parsing::inline_module::InlineModule;

// The types and the traits a file declares, in the order it declares them.
//
// Types are structs and enums. A type alias names another type rather than
// declaring one, so it is not here. Traits are found apart from types because
// named-subject ranks them ahead: a trait declares behaviour, a data type does
// not.
//
// One walk serves both, handed what to pick out, and it descends into inline
// modules: wrapping a declaration in `mod detail { ... }` keeps it in the file.
pub struct DeclaredItemFinder;

impl DeclaredItemFinder {
    pub fn traits(items: &[Item]) -> Vec<DeclaredItem> {
        Self::collect(items, Self::trait_name)
    }

    pub fn types(items: &[Item]) -> Vec<DeclaredItem> {
        Self::collect(items, Self::type_name)
    }

    fn collect(items: &[Item], name_of: fn(&Item) -> Option<String>) -> Vec<DeclaredItem> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Mod(module) => InlineModule::items(module)
                    .map(|inner| Self::collect(inner, name_of))
                    .unwrap_or_default(),
                _ => name_of(item)
                    .map(|name| DeclaredItem::new(&name, item.span().start().line))
                    .into_iter()
                    .collect(),
            })
            .collect()
    }

    fn trait_name(item: &Item) -> Option<String> {
        match item {
            Item::Trait(inner) => Some(inner.ident.to_string()),
            _ => None,
        }
    }

    fn type_name(item: &Item) -> Option<String> {
        match item {
            Item::Struct(inner) => Some(inner.ident.to_string()),
            Item::Enum(inner) => Some(inner.ident.to_string()),
            _ => None,
        }
    }
}
