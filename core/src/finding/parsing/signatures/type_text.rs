// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use quote::ToTokens;
use syn::Type;

// A type as it is written, for a reader: lifetimes dropped, and the token
// stream's spacing replaced by a space only between two words, after a
// separator and around a `+`, so `& 'a mut Vec < u8 >` reads `&mut Vec<u8>` and
// `dyn Error + Send` keeps its spaces. Two types written alike read alike.
pub struct TypeText;

impl TypeText {
    pub fn of(ty: &Type) -> String {
        Self::tightened(&Self::without_lifetimes(&ty.to_token_stream().to_string()))
    }

    fn is_word(character: char) -> bool {
        character.is_alphanumeric() || character == '_'
    }

    fn tightened(text: &str) -> String {
        text.split_whitespace().fold(String::new(), |joined, word| {
            let after = joined.chars().last();
            let spaced = after.is_some_and(|last| last == ',' || last == ';' || last == '+')
                || word == "+"
                || (after.is_some_and(Self::is_word) && word.starts_with(Self::is_word));
            if spaced {
                format!("{joined} {word}")
            } else {
                format!("{joined}{word}")
            }
        })
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
