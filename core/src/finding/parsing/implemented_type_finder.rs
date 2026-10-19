// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use syn::Item;
use syn::Type;
use syn::parse_file;

use crate::finding::model::implemented_type::ImplementedType;
use crate::finding::parsing::declared_item_finder::DeclaredItemFinder;
use crate::finding::parsing::inline_module::InlineModule;
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
        Some(Self::among(&syntax.items))
    }

    // For a caller that has parsed the file already and asks more of it than
    // this.
    //
    // The two halves are gathered separately rather than in one pass with two
    // accumulators handed down. Each answers one question and returns it, which
    // is what lets the recursion be an expression instead of a side effect. The
    // declarations are DeclaredItemFinder's, which named-subject reads too.
    //
    // Both descend into inline modules: a nested type with behaviour is still a
    // second subject in the same file.
    pub fn among(items: &[Item]) -> Vec<ImplementedType> {
        let implemented = Self::implemented(items);
        DeclaredItemFinder::types(items)
            .into_iter()
            .filter(|candidate| implemented.contains(&candidate.name))
            .map(|declared| ImplementedType::new(&declared.name, declared.line))
            .collect()
    }

    fn implemented(items: &[Item]) -> BTreeSet<String> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Impl(inner) => Self::implemented_name(&inner.self_ty)
                    .into_iter()
                    .collect::<BTreeSet<String>>(),
                Item::Mod(module) => InlineModule::items(module)
                    .map(Self::implemented)
                    .unwrap_or_default(),
                _ => BTreeSet::new(),
            })
            .collect()
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
