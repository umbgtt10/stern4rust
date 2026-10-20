// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::ImplItem;
use syn::Item;
use syn::ItemImpl;
use syn::ItemTrait;
use syn::Signature;
use syn::TraitItem;

use crate::finding::parsing::inline_module::InlineModule;

// The signatures a file's author can change: free functions, inherent methods
// and trait declarations, inline modules included.
//
// A trait impl is left out. Its signatures are the trait's decision -- `Display::fmt`
// takes `&mut Formatter` because `Display` says so -- and a rule judging a
// signature reports a trait of the author's own at its declaration, the one place
// changing it is both possible and enough.
pub struct AuthoredSignatures;

impl AuthoredSignatures {
    pub fn of(items: &[Item]) -> Vec<&Signature> {
        items.iter().flat_map(Self::in_item).collect()
    }

    fn in_impl(block: &ItemImpl) -> Vec<&Signature> {
        if block.trait_.is_some() {
            return Vec::new();
        }
        block
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(method) => Some(&method.sig),
                _ => None,
            })
            .collect()
    }

    fn in_item(item: &Item) -> Vec<&Signature> {
        match item {
            Item::Fn(function) => vec![&function.sig],
            Item::Impl(block) => Self::in_impl(block),
            Item::Trait(declaration) => Self::in_trait(declaration),
            Item::Mod(module) => InlineModule::items(module)
                .map(Self::of)
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    fn in_trait(declaration: &ItemTrait) -> Vec<&Signature> {
        declaration
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Fn(method) => Some(&method.sig),
                _ => None,
            })
            .collect()
    }
}
