// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use syn::Item;
use syn::Type;
use syn::spanned::Spanned;

use crate::implemented_type::ImplementedType;
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
        let syntax = syn::parse_file(&file.contents()).ok()?;
        let mut declared = Vec::new();
        let mut implemented = BTreeSet::new();
        Self::walk(&syntax.items, &mut declared, &mut implemented);
        Some(
            declared
                .into_iter()
                .filter(|(name, _)| implemented.contains(name))
                .map(|(name, line)| ImplementedType::new(&name, line))
                .collect(),
        )
    }

    // Descends into inline modules: a nested type with behaviour is still a
    // second subject in the same file.
    fn walk(
        items: &[Item],
        declared: &mut Vec<(String, usize)>,
        implemented: &mut BTreeSet<String>,
    ) {
        for item in items {
            match item {
                Item::Struct(inner) => {
                    declared.push((inner.ident.to_string(), item.span().start().line));
                }
                Item::Enum(inner) => {
                    declared.push((inner.ident.to_string(), item.span().start().line));
                }
                Item::Impl(inner) => {
                    if let Some(name) = Self::implemented_name(&inner.self_ty) {
                        implemented.insert(name);
                    }
                }
                Item::Mod(module) => {
                    if let Some((_, inner)) = &module.content {
                        Self::walk(inner, declared, implemented);
                    }
                }
                _ => {}
            }
        }
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
