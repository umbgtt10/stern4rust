// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use syn::Item;
use syn::ItemMod;
use syn::Type;
use syn::parse_file;
use syn::spanned::Spanned;

use crate::finding::implemented_type::ImplementedType;
use crate::source_file::SourceFile;

// The types a file both declares and gives behaviour to.
//
// Two halves have to meet. The type must be declared here, so an
// `impl Display for SomeoneElsesType` does not make this file that type's home.
// And it must carry at least one impl block, so a file may hold as many plain
// data declarations as its subject needs -- a handful of payload structs with
// no behaviour is one subject, not five.
//
// A trait impl counts as much as an inherent one. Both are behaviour, and a
// reader looking for what a type does opens the file either way.
pub struct ImplementedTypeFinder;

impl ImplementedTypeFinder {
    // None means the file does not parse. readable-source reports that.
    pub fn find(file: &SourceFile) -> Option<Vec<ImplementedType>> {
        let syntax = parse_file(&file.contents()).ok()?;
        let implemented = Self::implemented(&syntax.items);
        Some(
            Self::declared(&syntax.items)
                .into_iter()
                .filter(|candidate| implemented.contains(&candidate.name))
                .collect(),
        )
    }

    // The two halves are gathered separately rather than in one pass with two
    // accumulators handed down. Each answers one question and returns it, which
    // is what lets the recursion be an expression instead of a side effect.
    //
    // Both descend into inline modules: a nested type with behaviour is still a
    // second subject in the same file.
    fn declared(items: &[Item]) -> Vec<ImplementedType> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Struct(inner) => vec![Self::at(&inner.ident.to_string(), item)],
                Item::Enum(inner) => vec![Self::at(&inner.ident.to_string(), item)],
                Item::Mod(module) => Self::inside(module).map(Self::declared).unwrap_or_default(),
                _ => Vec::new(),
            })
            .collect()
    }

    fn implemented(items: &[Item]) -> BTreeSet<String> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Impl(inner) => Self::implemented_name(&inner.self_ty)
                    .into_iter()
                    .collect::<BTreeSet<String>>(),
                Item::Mod(module) => Self::inside(module)
                    .map(Self::implemented)
                    .unwrap_or_default(),
                _ => BTreeSet::new(),
            })
            .collect()
    }

    fn at(name: &str, item: &Item) -> ImplementedType {
        ImplementedType::new(name, item.span().start().line)
    }

    fn inside(module: &ItemMod) -> Option<&[Item]> {
        module.content.as_ref().map(|(_, items)| items.as_slice())
    }

    fn implemented_name(target: &Type) -> Option<String> {
        match target {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string()),
            _ => None,
        }
    }
}
