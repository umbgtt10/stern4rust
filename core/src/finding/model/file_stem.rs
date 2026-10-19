// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::Path;

use crate::finding::model::type_name::TypeName;

// A source file's name, as the words it is written in, held against a type it
// might be named after.
//
// The stem names a type when it is the type's name, underscores and case aside,
// or the type's last words. The words in front are what a type's crate or module
// usually supplies -- `loopback/src/driver.rs` holding `LoopbackDriver` -- and
// the last words are what the type is: a `UdpEiListener` is a listener. Only the
// front may be left out, because a stem with a word the type lacks claims more
// than the type says, and the match is on whole words, so `dex.rs` does not name
// `Index`.
pub struct FileStem {
    words: Vec<String>,
}

impl FileStem {
    pub fn of(relative_path: &str) -> Self {
        let stem = Path::new(relative_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default();
        Self {
            words: stem
                .split('_')
                .filter(|word| !word.is_empty())
                .map(str::to_lowercase)
                .collect(),
        }
    }

    pub fn names(&self, type_name: &str) -> bool {
        let name = TypeName::new(type_name);
        !self.words.is_empty()
            && (self.words.concat() == name.words().concat() || name.words().ends_with(&self.words))
    }
}
