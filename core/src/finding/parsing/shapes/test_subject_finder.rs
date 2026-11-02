// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use syn::Block;
use syn::ImplItem;
use syn::ImplItemFn;
use syn::Item;
use syn::ItemFn;
use syn::parse_file;

use crate::finding::parsing::shapes::package_surface::PackageSurface;
use crate::finding::parsing::shapes::test_body_visitor::TestBodyVisitor;
use crate::finding::parsing::shapes::test_subject::TestSubject;
use crate::finding::parsing::unit_test_finder::UnitTestFinder;
use crate::source_file::SourceFile;

pub struct TestSubjectFinder<'a> {
    surface: &'a PackageSurface,
}

impl<'a> TestSubjectFinder<'a> {
    pub fn new(surface: &'a PackageSurface) -> Self {
        Self { surface }
    }

    pub fn find(&self, file: &SourceFile) -> Vec<TestSubject> {
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        let functions = Self::functions(&syntax.items);
        let helpers: BTreeMap<String, &Block> = functions
            .iter()
            .filter(|function| !Self::is_test(function))
            .map(|function| (function.sig.ident.to_string(), function.block.as_ref()))
            .collect();
        let local_methods: BTreeMap<String, &Block> = Self::methods(&syntax.items)
            .into_iter()
            .map(|method| (method.sig.ident.to_string(), &method.block))
            .collect();
        functions
            .iter()
            .filter(|function| Self::is_test(function))
            .map(|test| TestBodyVisitor::new(self.surface, &helpers, &local_methods).examine(test))
            .collect()
    }

    fn functions(items: &[Item]) -> Vec<&ItemFn> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Fn(function) => vec![function],
                Item::Mod(module) => module
                    .content
                    .as_ref()
                    .map(|(_, inner)| Self::functions(inner))
                    .unwrap_or_default(),
                _ => Vec::new(),
            })
            .collect()
    }

    fn is_test(function: &ItemFn) -> bool {
        function.attrs.iter().any(UnitTestFinder::is_test)
    }

    fn methods(items: &[Item]) -> Vec<&ImplItemFn> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Impl(block) => block
                    .items
                    .iter()
                    .filter_map(|member| match member {
                        ImplItem::Fn(method) => Some(method),
                        _ => None,
                    })
                    .collect(),
                Item::Mod(module) => module
                    .content
                    .as_ref()
                    .map(|(_, inner)| Self::methods(inner))
                    .unwrap_or_default(),
                _ => Vec::new(),
            })
            .collect()
    }
}
