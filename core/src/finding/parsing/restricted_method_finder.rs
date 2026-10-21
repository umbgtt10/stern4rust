// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::ImplItem;
use syn::ImplItemFn;
use syn::Item;
use syn::ItemImpl;
use syn::VisRestricted;
use syn::Visibility;

use crate::finding::model::restricted_method::RestrictedMethod;
use crate::finding::parsing::impl_blocks::ImplBlocks;
use crate::finding::parsing::item_naming::ItemNaming;
use crate::finding::parsing::public_entry_point_finder::PublicEntryPointFinder;

// The methods of a file's inherent impls written `pub(crate)`, `pub(super)`,
// `pub(self)` or `pub(in ...)`, inline modules included.
//
// Only inherent impls: a trait impl's methods take the trait's visibility and
// carry none of their own, however a parser reads them, and a free function is
// not a method. Each is named the way a call names it, so the rule can ask
// which other files call it.
pub struct RestrictedMethodFinder;

impl RestrictedMethodFinder {
    pub fn find(items: &[Item]) -> Vec<RestrictedMethod> {
        ImplBlocks::inherent(items)
            .into_iter()
            .flat_map(Self::in_block)
            .collect()
    }

    fn in_block(block: &ItemImpl) -> Vec<RestrictedMethod> {
        let owner = ItemNaming::implemented_type(&block.self_ty);
        block
            .items
            .iter()
            .filter_map(|inner| match inner {
                ImplItem::Fn(method) => Self::restricted(&owner, method),
                _ => None,
            })
            .collect()
    }

    fn restricted(owner: &str, method: &ImplItemFn) -> Option<RestrictedMethod> {
        let Visibility::Restricted(restriction) = &method.vis else {
            return None;
        };
        Some(RestrictedMethod {
            call: PublicEntryPointFinder::of_signature(&method.sig),
            line: method.sig.ident.span().start().line,
            visibility: Self::written(restriction),
            owner: owner.to_string(),
        })
    }

    // As written, without the spaces a token stream puts in: `pub(in crate::store)`.
    fn written(restriction: &VisRestricted) -> String {
        let path = restriction
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<String>>()
            .join("::");
        let scope = if restriction.in_token.is_some() {
            format!("in {path}")
        } else {
            path
        };
        format!("pub({scope})")
    }
}
