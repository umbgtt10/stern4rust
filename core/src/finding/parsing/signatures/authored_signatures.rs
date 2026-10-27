// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::vec;
use alloc::vec::Vec;

use syn::ImplItem;
use syn::Item;
use syn::ItemImpl;
use syn::ItemTrait;
use syn::ReturnType;
use syn::Signature;
use syn::TraitItem;
use syn::Type;
use syn::Visibility;

use crate::finding::parsing::inline_module::InlineModule;

// The signatures a file's author can change: free functions, inherent methods
// and trait declarations, inline modules included -- every one, or the public
// ones a caller outside the crate can reach: a `pub fn`, a `pub` method of an
// inherent impl, and every method of a `pub` trait.
//
// A trait impl is left out. Its signatures are the trait's decision -- `Display::fmt`
// takes `&mut Formatter` because `Display` says so -- and a rule judging a
// signature reports a trait of the author's own at its declaration, the one place
// changing it is both possible and enough.
pub struct AuthoredSignatures;

impl AuthoredSignatures {
    pub fn of(items: &[Item]) -> Vec<&Signature> {
        Self::walk(items, Self::any)
    }

    // The type a function returns, None for `()` left unwritten -- what the
    // rules that judge a return type look at.
    pub fn output(signature: &Signature) -> Option<&Type> {
        match &signature.output {
            ReturnType::Type(_, output) => Some(output),
            ReturnType::Default => None,
        }
    }

    pub fn public(items: &[Item]) -> Vec<&Signature> {
        Self::walk(items, Self::is_public)
    }

    fn any(_visibility: &Visibility) -> bool {
        true
    }

    fn in_impl(block: &ItemImpl, keep: fn(&Visibility) -> bool) -> Vec<&Signature> {
        if block.trait_.is_some() {
            return Vec::new();
        }
        block
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(method) if keep(&method.vis) => Some(&method.sig),
                _ => None,
            })
            .collect()
    }

    fn in_item(item: &Item, keep: fn(&Visibility) -> bool) -> Vec<&Signature> {
        match item {
            Item::Fn(function) if keep(&function.vis) => vec![&function.sig],
            Item::Impl(block) => Self::in_impl(block, keep),
            Item::Trait(declaration) if keep(&declaration.vis) => Self::in_trait(declaration),
            Item::Mod(module) => InlineModule::items(module)
                .map(|inner| Self::walk(inner, keep))
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

    fn is_public(visibility: &Visibility) -> bool {
        matches!(visibility, Visibility::Public(_))
    }

    fn walk(items: &[Item], keep: fn(&Visibility) -> bool) -> Vec<&Signature> {
        items
            .iter()
            .flat_map(|item| Self::in_item(item, keep))
            .collect()
    }
}
