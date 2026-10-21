// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Type;
use syn::TypeTuple;

use crate::finding::model::repeated_tuple::RepeatedTuple;
use crate::finding::parsing::signatures::type_text::TypeText;
use crate::finding::parsing::signatures::type_walk::TypeWalk;

// The first tuple in a type that holds two elements of one type, wherever it
// sits: at the top, inside a generic, behind a reference. The outer tuple is found
// before anything it holds.
//
// Types are compared as written, with lifetimes and needless whitespace dropped,
// so `&'a str` and `&'static str` are one type. Nothing is resolved: an alias is
// its own name, and an alias that names what a value means is the cure for the
// ambiguity this looks for, not a way around it.
pub struct RepeatedTupleFinder;

impl RepeatedTupleFinder {
    pub fn first_in(ty: &Type) -> Option<RepeatedTuple> {
        let own = match ty {
            Type::Tuple(tuple) => Self::repeated(tuple),
            _ => None,
        };
        own.or_else(|| TypeWalk::children(ty).into_iter().find_map(Self::first_in))
    }

    fn repeated(tuple: &TypeTuple) -> Option<RepeatedTuple> {
        let elements: Vec<String> = tuple.elems.iter().map(TypeText::of).collect();
        let element = elements
            .iter()
            .enumerate()
            .find(|(at, element)| elements[..*at].contains(element))
            .map(|(_, element)| element.clone())?;
        Some(RepeatedTuple::new(
            format!("({})", elements.join(", ")),
            element,
        ))
    }
}
