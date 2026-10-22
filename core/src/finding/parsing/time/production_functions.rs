// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Attribute;
use syn::Block;
use syn::Ident;
use syn::ImplItem;
use syn::Item;
use syn::ItemImpl;

use crate::finding::parsing::inline_module::InlineModule;
use crate::finding::parsing::time::port_traits::PortTraits;
use crate::finding::parsing::unit_test_finder::UnitTestFinder;

// The functions of a source file that are production code and not a port's adapter: free functions, inherent
// methods and the methods of a standard trait's impl, through inline modules. Left out: a test, anything under
// `#[cfg(test)]`, the methods of an impl of a port -- a trait the package or a dependency declares, the adapter
// a test replaces (`PortTraits` says which) --, and a trait's own default bodies, which a test's implementation
// overrides.
pub struct ProductionFunctions;

impl ProductionFunctions {
    pub fn of<'a>(items: &'a [Item], ports: &PortTraits) -> Vec<(&'a Ident, &'a Block)> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Fn(function) if !Self::is_test(&function.attrs) => {
                    vec![(&function.sig.ident, function.block.as_ref())]
                }
                Item::Impl(block) => Self::in_impl(block, ports),
                Item::Mod(module) if !Self::is_test(&module.attrs) => InlineModule::items(module)
                    .map(|inner| Self::of(inner, ports))
                    .unwrap_or_default(),
                _ => Vec::new(),
            })
            .collect()
    }

    fn in_impl<'a>(block: &'a ItemImpl, ports: &PortTraits) -> Vec<(&'a Ident, &'a Block)> {
        if Self::is_port(block, ports) || Self::is_test(&block.attrs) {
            return Vec::new();
        }
        block
            .items
            .iter()
            .filter_map(|inner| match inner {
                ImplItem::Fn(method) if !Self::is_test(&method.attrs) => {
                    Some((&method.sig.ident, &method.block))
                }
                _ => None,
            })
            .collect()
    }

    fn is_port(block: &ItemImpl, ports: &PortTraits) -> bool {
        block
            .trait_
            .as_ref()
            .is_some_and(|(_, path, _)| ports.is_port(path))
    }

    fn is_test(attrs: &[Attribute]) -> bool {
        attrs
            .iter()
            .any(|attr| UnitTestFinder::is_test(attr) || UnitTestFinder::is_cfg_test(attr))
    }
}
