// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::finding::parsing::modules::module_declaration_finder::ModuleDeclarationFinder;
use crate::source_file::SourceFile;

// The files of a package's library target: `src/lib.rs` and every file its
// module declarations reach, as `name.rs` or `name/mod.rs`, followed down.
//
// What a binary holds is not among them -- `src/main.rs`, `src/bin/`, and the
// modules only they declare -- and neither is a package with no `src/lib.rs`. A
// declaration inside an inline module is not followed.
pub struct LibraryTarget {
    files: BTreeSet<String>,
}

impl LibraryTarget {
    pub const ROOT: &'static str = "src/lib.rs";

    pub fn of(files: &[SourceFile]) -> Self {
        let by_path: BTreeMap<&str, &SourceFile> = files
            .iter()
            .map(|file| (file.relative_path(), file))
            .collect();
        let mut reached = BTreeSet::new();
        let mut pending = vec![String::from(Self::ROOT)];
        while let Some(path) = pending.pop() {
            let Some(file) = by_path.get(path.as_str()) else {
                continue;
            };
            if !reached.insert(path.clone()) {
                continue;
            }
            let directory = ModuleDeclarationFinder::directory_of(&path);
            for name in ModuleDeclarationFinder::find(file).unwrap_or_default() {
                pending.push(format!("{directory}{name}.rs"));
                pending.push(format!("{directory}{name}/mod.rs"));
            }
        }
        Self { files: reached }
    }

    pub fn contains(&self, path: &str) -> bool {
        self.files.contains(path)
    }
}
