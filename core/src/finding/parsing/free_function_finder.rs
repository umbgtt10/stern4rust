// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use quote::ToTokens;
use syn::Attribute;
use syn::Ident;
use syn::Item;
use syn::ItemFn;
use syn::ItemMod;
use syn::Path;
use syn::visit;
use syn::visit::Visit;

use crate::finding::model::free_function::FreeFunction;
use crate::finding::parsing::identifiers::Identifiers;

// The functions of a file that belong to no impl block.
//
// Module level, inside an inline module and nested in another function's body
// all count -- a function declared inside a method is still free, only harder
// to see -- which is why this is a visit rather than a walk of the items.
//
// Left out are the functions something other than the author requires to be
// free: an entry point or handler a framework attribute names, an exported
// symbol, an extern function, and `main` in an entry file. So is anything under
// `#[cfg(test)]`, which test-free-source reports.
pub struct FreeFunctionFinder {
    entry_file: bool,
    found: Vec<FreeFunction>,
}

impl FreeFunctionFinder {
    // Matched on the attribute path's last segment, so
    // `#[embassy_executor::task]` and `#[cortex_m_rt::exception]` are both
    // recognised. Inside `#[unsafe(...)]`, as edition 2024 writes `no_mangle`,
    // the inner path is the one that counts.
    pub const REQUIRED_FREE: [&'static str; 13] = [
        "alloc_error_handler",
        "entry",
        "exception",
        "export_name",
        "interrupt",
        "main",
        "no_mangle",
        "panic_handler",
        "pre_init",
        "proc_macro",
        "proc_macro_attribute",
        "proc_macro_derive",
        "task",
    ];

    pub fn among(items: &[Item], entry_file: bool) -> Vec<FreeFunction> {
        let mut finder = Self {
            entry_file,
            found: Vec::new(),
        };
        for item in items {
            finder.visit_item(item);
        }
        finder.found
    }

    fn is_required_free(&self, function: &ItemFn) -> bool {
        function.sig.abi.is_some()
            || (self.entry_file && function.sig.ident == "main")
            || function.attrs.iter().any(Self::requires_free)
    }

    fn is_test_only(attributes: &[Attribute]) -> bool {
        attributes.iter().any(|attribute| {
            attribute.path().is_ident("cfg")
                && attribute
                    .parse_args::<Ident>()
                    .is_ok_and(|argument| argument == "test")
        })
    }

    fn last_segment(path: &Path) -> String {
        path.segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default()
    }

    fn requires_free(attribute: &Attribute) -> bool {
        let named = Self::last_segment(attribute.path());
        let effective = if named == "unsafe" {
            attribute
                .parse_args::<Path>()
                .map(|inner| Self::last_segment(&inner))
                .unwrap_or_default()
        } else {
            named
        };
        Self::REQUIRED_FREE.contains(&effective.as_str())
    }
}

impl<'ast> Visit<'ast> for FreeFunctionFinder {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        if Self::is_test_only(&function.attrs) {
            return;
        }
        if !self.is_required_free(function) {
            self.found.push(FreeFunction::new(
                function.sig.ident.to_string(),
                function.sig.ident.span().start().line,
                Identifiers::of(function.to_token_stream()),
            ));
        }
        visit::visit_item_fn(self, function);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !Self::is_test_only(&module.attrs) {
            visit::visit_item_mod(self, module);
        }
    }
}
