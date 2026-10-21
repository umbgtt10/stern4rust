// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::parse_file;

use crate::source_file::SourceFile;

// The module names a registry declares.
//
// `pub` is not required here, though `module-registry` requires it in `src/`
// and this rule is about a different question: a private `mod name;` compiles
// that file just as well, and being compiled is the whole concern. Demanding
// `pub` in this rule would report a file that is reached as though it were not.
//
// An inline `mod name { ... }` is not counted. It declares no file, so it
// cannot be what reaches one.
pub struct ModuleDeclarationFinder;

impl ModuleDeclarationFinder {
    // None means the file does not parse. readable-source reports that, and
    // guessing at the declarations of a file nobody can parse would be worse
    // than saying nothing.
    pub fn find(file: &SourceFile) -> Option<Vec<String>> {
        let syntax = parse_file(&file.contents()).ok()?;
        Some(
            syntax
                .items
                .iter()
                .filter_map(|item| match item {
                    Item::Mod(module) if module.content.is_none() => Some(module.ident.to_string()),
                    _ => None,
                })
                .collect(),
        )
    }
}
