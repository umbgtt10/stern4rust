// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use syn::Item;
use syn::parse_file;

use crate::finding::parsing::declared_item_finder::DeclaredItemFinder;
use crate::finding::parsing::gated_modules::GatedModules;
use crate::finding::parsing::impl_blocks::ImplBlocks;
use crate::finding::parsing::item_naming::ItemNaming;
use crate::source_file::SourceFile;

// Where each type of a package is declared, and which files hold its inherent
// impls.
//
// A type is its name, the last segment of the path an impl is for, the way
// single-implemented-type reads it: `impl<T> Store<T>`, `impl Store<u8>` and
// `impl crate::store::Store` are all `Store`. Nothing is resolved. Trait impls
// are not counted -- a seam's implementation sits where the house pattern puts
// it -- and impls inside inline modules are, being in the file.
//
// A type is spread when its inherent impls sit in two or more files. A name
// declared in two or more files is read as alternatives instead -- platform
// types a `#[cfg]` chooses between, or unrelated types sharing a name -- and is
// spread only when an impl sits in a file declaring none of them. A file whose
// impls of a type a `#[cfg]` selects counts towards a split by platform.
pub struct TypeHomes {
    declared: BTreeMap<String, BTreeMap<String, usize>>,
    implemented: BTreeMap<String, BTreeMap<String, usize>>,
    gated: BTreeMap<String, BTreeSet<String>>,
}

impl TypeHomes {
    pub fn of(root: &str, files: &[SourceFile]) -> Self {
        let modules = GatedModules::of(files);
        let mut homes = Self {
            declared: BTreeMap::new(),
            implemented: BTreeMap::new(),
            gated: BTreeMap::new(),
        };
        for file in files {
            let path = file.relative_path();
            if !path.starts_with(root) {
                continue;
            }
            if let Ok(syntax) = parse_file(&file.contents()) {
                homes.record(path, &syntax.items, modules.gates(path));
            }
        }
        homes
    }

    // Where an offence about the type belongs: its declaration, or with none in
    // the package, its first impl.
    pub fn anchor_file(&self, name: &str) -> String {
        self.anchor(name)
            .map(|(file, _)| file.clone())
            .unwrap_or_default()
    }

    pub fn anchor_line(&self, name: &str) -> usize {
        self.anchor(name).map_or(1, |(_, line)| *line)
    }

    pub fn declaring_files(&self, name: &str) -> Vec<String> {
        Self::files(&self.declared, name)
    }

    pub fn implementing_files(&self, name: &str) -> Vec<String> {
        Self::files(&self.implemented, name)
    }

    pub fn outside_declarations(&self, name: &str) -> Vec<String> {
        let declaring = self.declaring_files(name);
        self.implementing_files(name)
            .into_iter()
            .filter(|file| !declaring.contains(file))
            .collect()
    }

    pub fn split_by_platform(&self, name: &str) -> bool {
        self.gated.get(name).is_some_and(|files| files.len() >= 2)
    }

    pub fn spread(&self) -> Vec<String> {
        self.implemented
            .keys()
            .filter(|name| self.is_spread(name))
            .cloned()
            .collect()
    }

    fn anchor(&self, name: &str) -> Option<(&String, &usize)> {
        self.declared
            .get(name)
            .and_then(|files| files.iter().next())
            .or_else(|| {
                self.implemented
                    .get(name)
                    .and_then(|files| files.iter().next())
            })
    }

    fn files(homes: &BTreeMap<String, BTreeMap<String, usize>>, name: &str) -> Vec<String> {
        homes
            .get(name)
            .map(|files| files.keys().cloned().collect())
            .unwrap_or_default()
    }

    fn is_spread(&self, name: &str) -> bool {
        match self.declaring_files(name).len() {
            0 | 1 => self.implementing_files(name).len() >= 2,
            _ => !self.outside_declarations(name).is_empty(),
        }
    }

    fn record(&mut self, path: &str, items: &[Item], module_gated: bool) {
        for declared in DeclaredItemFinder::types(items) {
            self.declared
                .entry(declared.name)
                .or_default()
                .entry(path.to_string())
                .or_insert(declared.line);
        }
        for block in ImplBlocks::inherent(items) {
            let name = ItemNaming::implemented_type(&block.self_ty);
            if name.is_empty() {
                continue;
            }
            if module_gated || GatedModules::is_cfg(&block.attrs) {
                self.gated
                    .entry(name.clone())
                    .or_default()
                    .insert(path.to_string());
            }
            self.implemented
                .entry(name)
                .or_default()
                .entry(path.to_string())
                .or_insert(block.impl_token.span.start().line);
        }
    }
}
