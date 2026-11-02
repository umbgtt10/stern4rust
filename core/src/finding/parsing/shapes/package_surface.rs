// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use syn::ImplItem;
use syn::Item;
use syn::ItemImpl;
use syn::TraitItem;
use syn::parse_file;

use crate::finding::parsing::impl_blocks::ImplBlocks;
use crate::finding::parsing::shapes::type_shape::TypeShape;
use crate::finding::parsing::shapes::type_shapes::TypeShapes;
use crate::finding::parsing::source_items::SourceItems;
use crate::source_file::SourceFile;

// What a package offers its tests. A record is a Record of
// `records-and-objects` that also writes no trait method by hand: a derived
// trait is the compiler's, but a hand-written `fmt` or `from_str` is code a
// test can pin. Behaviour is every other method the package writes, and its
// free functions.
pub struct PackageSurface {
    records: BTreeSet<String>,
    objects: BTreeSet<String>,
    behaviour_methods: BTreeSet<String>,
    free_functions: BTreeSet<String>,
}

impl PackageSurface {
    pub fn of(files: &[SourceFile]) -> Self {
        let parsed: Vec<Vec<Item>> = files
            .iter()
            .filter_map(|file| parse_file(&file.contents()).ok())
            .map(|syntax| syntax.items)
            .collect();
        let live: Vec<&Item> = parsed
            .iter()
            .flat_map(|items| SourceItems::live(items))
            .collect();
        let crafted = Self::crafted_types(&live);
        let shapes = TypeShapes::of(files);
        let records: BTreeSet<String> = shapes
            .types()
            .iter()
            .filter(|shaped| {
                shaped.shape() == TypeShape::Record && !crafted.contains(shaped.name())
            })
            .map(|shaped| shaped.name().to_string())
            .collect();
        let objects: BTreeSet<String> = shapes
            .types()
            .iter()
            .map(|shaped| shaped.name().to_string())
            .filter(|name| !records.contains(name))
            .collect();
        let behaviour_methods = Self::behaviour_methods(&live, &objects);
        let free_functions = live
            .iter()
            .filter_map(|item| match item {
                Item::Fn(function) => Some(function.sig.ident.to_string()),
                _ => None,
            })
            .collect();
        Self {
            records,
            objects,
            behaviour_methods,
            free_functions,
        }
    }

    pub fn has_records(&self) -> bool {
        !self.records.is_empty()
    }

    pub fn is_behaviour_method(&self, name: &str) -> bool {
        self.behaviour_methods.contains(name)
    }

    pub fn is_free_function(&self, name: &str) -> bool {
        self.free_functions.contains(name)
    }

    pub fn is_object(&self, name: &str) -> bool {
        self.objects.contains(name)
    }

    pub fn is_record(&self, name: &str) -> bool {
        self.records.contains(name)
    }

    fn behaviour_methods(live: &[&Item], objects: &BTreeSet<String>) -> BTreeSet<String> {
        live.iter()
            .flat_map(|item| match item {
                Item::Impl(block) if block.trait_.is_some() => Self::method_names(block),
                Item::Impl(block) => match ImplBlocks::target(&block.self_ty) {
                    Some(target) if objects.contains(&target) => Self::method_names(block),
                    _ => Vec::new(),
                },
                Item::Trait(declared) => declared
                    .items
                    .iter()
                    .filter_map(|member| match member {
                        TraitItem::Fn(method) if method.default.is_some() => {
                            Some(method.sig.ident.to_string())
                        }
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            })
            .collect()
    }

    fn crafted_types(live: &[&Item]) -> BTreeSet<String> {
        live.iter()
            .filter_map(|item| match item {
                Item::Impl(block)
                    if block.trait_.is_some() && !Self::method_names(block).is_empty() =>
                {
                    ImplBlocks::target(&block.self_ty)
                }
                _ => None,
            })
            .collect()
    }

    fn method_names(block: &ItemImpl) -> Vec<String> {
        block
            .items
            .iter()
            .filter_map(|member| match member {
                ImplItem::Fn(method) => Some(method.sig.ident.to_string()),
                _ => None,
            })
            .collect()
    }
}
