// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A type a file both declares and gives behaviour to -- one of the file's
// subjects.
//
// The line is the declaration's, not the impl block's, because the declaration
// is what a reader moves when the file turns out to have two subjects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementedType {
    pub name: String,
    pub line: usize,
}

impl ImplementedType {
    pub fn new(name: &str, line: usize) -> Self {
        Self {
            name: name.to_string(),
            line,
        }
    }

    // The file this type would live in on its own: PascalCase to snake_case, so
    // the correction names a path rather than describing a convention.
    pub fn suggested_file(&self) -> String {
        let mut file = String::new();
        for (index, character) in self.name.chars().enumerate() {
            if character.is_uppercase() && index > 0 {
                file.push('_');
            }
            file.extend(character.to_lowercase());
        }
        file.push_str(".rs");
        file
    }
}
