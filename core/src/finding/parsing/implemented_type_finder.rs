// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use syn::Item;
use syn::parse_file;

use crate::finding::model::implemented_type::ImplementedType;
use crate::finding::parsing::declared_item_finder::DeclaredItemFinder;
use crate::finding::parsing::shapes::type_shape::TypeShape;
use crate::finding::parsing::shapes::type_shapes::TypeShapes;
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
    // For a caller that has parsed the file already and asks more of it than
    // this.
    //
    // The two halves are gathered separately: the declarations are
    // DeclaredItemFinder's, which named-subject reads too, and the impl blocks'
    // types are ImplBlocks', which owned-functions reads. Both descend into
    // inline modules: a nested type with behaviour is still a second subject in
    // the same file.
    pub fn among(items: &[Item]) -> Vec<ImplementedType> {
        let shapes = TypeShapes::in_items("", items);
        DeclaredItemFinder::types(items)
            .into_iter()
            .filter(|candidate| {
                shapes
                    .get(candidate.name())
                    .is_some_and(|shaped| shaped.shape() != TypeShape::Record)
            })
            .map(|declared| ImplementedType::new(declared.name(), declared.line()))
            .collect()
    }

    // None means the file does not parse. readable-source reports that.
    pub fn find(file: &SourceFile) -> Option<Vec<ImplementedType>> {
        let syntax = parse_file(&file.contents()).ok()?;
        Some(Self::among(&syntax.items))
    }
}
