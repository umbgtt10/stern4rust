// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A type's name as the words it is written in, lower-cased.
//
// A capital starts a word, except inside a run of capitals: an acronym written
// in capitals is one word, ended by the capital that starts the next one, so
// `HTTPServer` is `http` and `server` rather than one word per letter. A digit
// stays with the word it follows, and an underscore is a break of its own.
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub struct TypeName {
    words: Vec<String>,
}

impl TypeName {
    pub fn new(name: &str) -> Self {
        Self {
            words: name.split('_').flat_map(Self::split).collect(),
        }
    }

    // The file this type would live in on its own, so a correction names a path
    // rather than describing a convention.
    pub fn file_name(&self) -> String {
        format!("{}.rs", self.words.join("_"))
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }

    fn split(part: &str) -> Vec<String> {
        let characters: Vec<char> = part.chars().collect();
        let starts: Vec<usize> = (0..characters.len())
            .filter(|index| Self::starts_word(&characters, *index))
            .collect();
        let ends = starts.iter().copied().skip(1).chain([characters.len()]);
        starts
            .iter()
            .copied()
            .zip(ends)
            .map(|(start, end)| {
                characters[start..end]
                    .iter()
                    .collect::<String>()
                    .to_lowercase()
            })
            .collect()
    }

    fn starts_word(characters: &[char], index: usize) -> bool {
        let Some(previous) = index.checked_sub(1).map(|before| characters[before]) else {
            return true;
        };
        let next_is_lower = characters
            .get(index + 1)
            .is_some_and(|next| next.is_lowercase());
        characters[index].is_uppercase() && (!previous.is_uppercase() || next_is_lower)
    }
}
