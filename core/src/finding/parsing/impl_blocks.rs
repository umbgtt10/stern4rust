// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use quote::ToTokens;
use syn::Item;
use syn::ItemImpl;
use syn::Type;

use crate::finding::parsing::identifiers::Identifiers;
use crate::finding::parsing::inline_module::InlineModule;

// The impl blocks of a file: the types they are for, in source order, and the
// identifiers their bodies use.
//
// The types are a file's subject when it declares none of its own -- a file
// continuing another file's impl is about that type. The identifiers are what
// tells a free function the impl calls, which is its helper, from one it never
// reaches. Both descend into inline modules.
pub struct ImplBlocks {
    targets: Vec<String>,
    uses: BTreeSet<String>,
}

impl ImplBlocks {
    pub fn of(items: &[Item]) -> Self {
        let blocks = Self::collect(items);
        let mut targets: Vec<String> = Vec::new();
        for target in blocks
            .iter()
            .filter_map(|block| Self::target(&block.self_ty))
        {
            if !targets.contains(&target) {
                targets.push(target);
            }
        }
        Self {
            targets,
            uses: blocks
                .iter()
                .flat_map(|block| Identifiers::of(block.to_token_stream()))
                .collect(),
        }
    }

    // The impl blocks a type owns rather than a trait: no trait impl, inline
    // modules included, in source order.
    pub fn inherent(items: &[Item]) -> Vec<&ItemImpl> {
        Self::collect(items)
            .into_iter()
            .filter(|block| block.trait_.is_none())
            .collect()
    }

    pub fn names(&self, identifier: &str) -> bool {
        self.uses.contains(identifier)
    }

    pub fn target(self_ty: &Type) -> Option<String> {
        match self_ty {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string()),
            _ => None,
        }
    }

    pub fn targets(&self) -> &[String] {
        &self.targets
    }

    fn as_impl(item: &Item) -> Option<&ItemImpl> {
        match item {
            Item::Impl(block) => Some(block),
            _ => None,
        }
    }

    fn collect(items: &[Item]) -> Vec<&ItemImpl> {
        InlineModule::flatten(items)
            .into_iter()
            .filter_map(Self::as_impl)
            .collect()
    }
}
