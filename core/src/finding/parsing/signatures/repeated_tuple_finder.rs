// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use quote::ToTokens;
use syn::Type;
use syn::TypeTuple;

use crate::finding::model::repeated_tuple::RepeatedTuple;
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

    fn is_word(character: char) -> bool {
        character.is_alphanumeric() || character == '_'
    }

    fn repeated(tuple: &TypeTuple) -> Option<RepeatedTuple> {
        let elements: Vec<String> = tuple.elems.iter().map(Self::text).collect();
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

    // Tokens rejoined the way the type is written: a space only between two words
    // and after a separator, so `& mut Vec < u8 >` reads `&mut Vec<u8>`.
    fn tightened(text: &str) -> String {
        text.split_whitespace().fold(String::new(), |joined, word| {
            let after = joined.chars().last();
            let spaced = after.is_some_and(|last| last == ',' || last == ';')
                || (after.is_some_and(Self::is_word) && word.starts_with(Self::is_word));
            if spaced {
                format!("{joined} {word}")
            } else {
                format!("{joined}{word}")
            }
        })
    }

    fn text(ty: &Type) -> String {
        Self::tightened(&Self::without_lifetimes(&ty.to_token_stream().to_string()))
    }

    fn without_lifetimes(text: &str) -> String {
        let mut kept = String::new();
        let mut characters = text.chars().peekable();
        while let Some(character) = characters.next() {
            if character == '\'' {
                while characters.next_if(|next| Self::is_word(*next)).is_some() {}
                continue;
            }
            kept.push(character);
        }
        kept
    }
}
