// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Attribute;
use syn::Item;
use syn::parse_file;

use crate::finding::parsing::modules::module_declaration_finder::ModuleDeclarationFinder;
use crate::source_file::SourceFile;

// The files of a package a `#[cfg]` selects: a module whose `mod name;`
// declaration carries one, every file inside it, and a file opening with
// `#![cfg(...)]`.
//
// That is how a crate builds a different file for each platform -- the gate sits
// on the declaration in the registry above, and the file it selects carries
// nothing to say so. Each gated module is kept as its stem, `src/hardware` for
// `#[cfg(...)] pub mod hardware;` in `src/lib.rs`, which selects `hardware.rs`
// and everything under `hardware/`. Declarations inside inline modules are not
// followed, and `cfg_attr` gates nothing: it sets an attribute under a
// condition, and the module is built either way.
pub struct GatedModules {
    stems: Vec<String>,
}

impl GatedModules {
    pub fn of(files: &[SourceFile]) -> Self {
        Self {
            stems: files.iter().flat_map(Self::stems_in).collect(),
        }
    }

    pub fn gates(&self, path: &str) -> bool {
        self.stems.iter().any(|stem| {
            path.strip_prefix(stem.as_str())
                .is_some_and(|rest| rest == ".rs" || rest.starts_with('/'))
        })
    }

    pub fn is_cfg(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| attr.path().is_ident("cfg"))
    }

    fn own_stem(path: &str) -> String {
        path.strip_suffix("/mod.rs")
            .or_else(|| path.strip_suffix(".rs"))
            .unwrap_or(path)
            .to_string()
    }

    fn stems_in(file: &SourceFile) -> Vec<String> {
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        let path = file.relative_path();
        let directory = ModuleDeclarationFinder::directory_of(path);
        let mut stems: Vec<String> = syntax
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Mod(module) if module.content.is_none() && Self::is_cfg(&module.attrs) => {
                    Some(format!("{directory}{}", module.ident))
                }
                _ => None,
            })
            .collect();
        if Self::is_cfg(&syntax.attrs) {
            stems.push(Self::own_stem(path));
        }
        stems
    }
}
