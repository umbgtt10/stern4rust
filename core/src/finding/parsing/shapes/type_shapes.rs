// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::ImplItem;
use syn::Item;
use syn::ItemImpl;
use syn::parse_file;
use syn::spanned::Spanned;

use crate::finding::parsing::impl_blocks::ImplBlocks;
use crate::finding::parsing::shapes::method_classifier::MethodClassifier;
use crate::finding::parsing::shapes::shaped_type::ShapedType;
use crate::finding::parsing::unit_test_finder::UnitTestFinder;
use crate::source_file::SourceFile;

pub struct TypeShapes {
    types: Vec<ShapedType>,
}

impl TypeShapes {
    pub fn of(files: &[SourceFile]) -> Self {
        let parsed: Vec<(String, Vec<Item>)> = files
            .iter()
            .filter_map(|file| {
                parse_file(&file.contents())
                    .ok()
                    .map(|syntax| (file.relative_path().to_string(), syntax.items))
            })
            .collect();
        let borrowed: Vec<(&str, &[Item])> = parsed
            .iter()
            .map(|(path, items)| (path.as_str(), items.as_slice()))
            .collect();
        Self::shaped(&borrowed)
    }

    fn shaped(parsed: &[(&str, &[Item])]) -> Self {
        let declared: Vec<ShapedType> = parsed
            .iter()
            .flat_map(|(path, items)| Self::declared(path, items))
            .collect();
        let types = parsed
            .iter()
            .flat_map(|(_, items)| Self::impls(items))
            .fold(declared, |types, block| {
                let target = ImplBlocks::target(&block.self_ty);
                types
                    .into_iter()
                    .map(|shaped| match &target {
                        Some(name) if name == shaped.name() => Self::applied(shaped, block),
                        _ => shaped,
                    })
                    .collect()
            });
        Self { types }
    }

    pub fn in_items(path: &str, items: &[Item]) -> Self {
        Self::shaped(&[(path, items)])
    }

    pub fn get(&self, name: &str) -> Option<&ShapedType> {
        self.types.iter().find(|shaped| shaped.name() == name)
    }

    pub fn types(&self) -> &[ShapedType] {
        &self.types
    }

    fn applied(shaped: ShapedType, block: &ItemImpl) -> ShapedType {
        if let Some((_, path, _)) = &block.trait_ {
            let name = path
                .segments
                .last()
                .map(|segment| segment.ident.to_string())
                .unwrap_or_default();
            return shaped.with_trait(&name);
        }
        block.items.iter().fold(shaped, |shaped, item| match item {
            ImplItem::Fn(method) => {
                let shape = MethodClassifier::shape(method, shaped.name());
                shaped.with_method(&method.sig.ident.to_string(), shape)
            }
            _ => shaped,
        })
    }

    fn declared(path: &str, items: &[Item]) -> Vec<ShapedType> {
        Self::live(items)
            .into_iter()
            .flat_map(|item| match item {
                Item::Struct(declared) => vec![ShapedType::new(
                    &declared.ident.to_string(),
                    path,
                    declared.span().start().line,
                )],
                Item::Enum(declared) => vec![ShapedType::new(
                    &declared.ident.to_string(),
                    path,
                    declared.span().start().line,
                )],
                _ => Vec::new(),
            })
            .collect()
    }

    fn impls(items: &[Item]) -> Vec<&ItemImpl> {
        Self::live(items)
            .into_iter()
            .filter_map(|item| match item {
                Item::Impl(block) => Some(block),
                _ => None,
            })
            .collect()
    }

    fn is_test_only(item: &Item) -> bool {
        let attrs = match item {
            Item::Mod(module) => &module.attrs,
            Item::Impl(block) => &block.attrs,
            Item::Struct(declared) => &declared.attrs,
            Item::Enum(declared) => &declared.attrs,
            _ => return false,
        };
        attrs.iter().any(UnitTestFinder::is_cfg_test)
    }

    fn live(items: &[Item]) -> Vec<&Item> {
        items
            .iter()
            .filter(|item| !Self::is_test_only(item))
            .flat_map(|item| match item {
                Item::Mod(module) => module
                    .content
                    .as_ref()
                    .map(|(_, inner)| Self::live(inner))
                    .unwrap_or_default(),
                _ => vec![item],
            })
            .collect()
    }
}
