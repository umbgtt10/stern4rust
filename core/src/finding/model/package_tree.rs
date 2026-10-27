// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::source_file::SourceFile;
use crate::source_path::SourcePath;

// The walked files arranged as the directories they sit in.
//
// A registry declares the files beside it and the folders directly beneath it,
// so the question "is this file reached" can only be asked of a directory as a
// whole. This is that shape: every directory the run saw, the files in it, and
// which of those files is a registry.
//
// `main.rs` counts as a registry alongside `lib.rs`. It is an entry point
// rather than an index and legitimately holds code, but it may still declare
// modules -- and a file declared only from `main.rs` is reached. Treating the
// registries of a directory as one set is what keeps that from being reported
// as an orphan.
pub struct PackageTree {
    directories: BTreeMap<SourcePath, Vec<SourcePath>>,
}

impl PackageTree {
    pub const REGISTRY_NAMES: [&'static str; 4] = ["all_tests.rs", "lib.rs", "main.rs", "mod.rs"];

    // Every ancestor is a key too, even when it holds no file of its own.
    // `src/` whose sources all live one level down is still a directory, and a
    // rule counting what it contains has to be able to ask about it -- without
    // this, a tree of nothing but subfolders was invisible to the walk.
    pub fn of(files: &[SourceFile]) -> Self {
        let mut directories: BTreeMap<SourcePath, Vec<SourcePath>> = BTreeMap::new();
        for file in files {
            let path = SourcePath::new(file.relative_path().replace('\\', "/"));
            let parent = path.parent().unwrap_or_default();
            directories.entry(parent.clone()).or_default().push(path);
            let mut ancestor = parent;
            while let Some(above) = ancestor.parent() {
                directories.entry(above.clone()).or_default();
                ancestor = above;
            }
        }
        Self { directories }
    }

    pub fn directories(&self) -> Vec<&SourcePath> {
        self.directories.keys().collect()
    }

    // What the registries of this directory have to declare for everything
    // beside them to be compiled: each sibling module file, and each subfolder
    // that is itself a module.
    pub fn expected_modules_in(&self, directory: &SourcePath) -> Vec<String> {
        let mut expected: Vec<String> = self
            .files_in(directory)
            .into_iter()
            .filter(|path| !Self::is_registry(path))
            .filter_map(Self::module_name)
            .collect();
        expected.extend(self.submodules_of(directory));
        expected.sort();
        expected
    }

    pub fn files_in(&self, directory: &SourcePath) -> Vec<&SourcePath> {
        self.directories
            .get(directory)
            .map(|paths| paths.iter().collect())
            .unwrap_or_default()
    }

    // The registries of one directory, in REGISTRY_NAMES order, so the offence
    // lands on `lib.rs` rather than `main.rs` when a package has both.
    pub fn registries_in(&self, directory: &SourcePath) -> Vec<&SourcePath> {
        let mut found: Vec<&SourcePath> = self
            .files_in(directory)
            .into_iter()
            .filter(|path| Self::is_registry(path))
            .collect();
        found.sort_by_key(|path| Self::registry_rank(path));
        found
    }

    // Every directory the walk saw directly beneath this one. Directories
    // holding no `.rs` file are not keys and so are not counted: the tool
    // cannot see them, and a folder of documentation is nobody's module.
    pub fn subdirectories_of(&self, directory: &SourcePath) -> Vec<&SourcePath> {
        self.directories
            .keys()
            .filter(|candidate| candidate.parent().as_ref() == Some(directory))
            .collect()
    }

    fn directory_name(path: &SourcePath) -> Option<String> {
        path.file_name().map(str::to_string)
    }

    fn is_registry(path: &SourcePath) -> bool {
        path.file_name()
            .is_some_and(|name| Self::REGISTRY_NAMES.contains(&name))
    }

    fn module_name(path: &SourcePath) -> Option<String> {
        path.file_stem().map(str::to_string)
    }

    fn registry_rank(path: &SourcePath) -> usize {
        path.file_name()
            .and_then(|name| Self::REGISTRY_NAMES.iter().position(|known| *known == name))
            .unwrap_or(usize::MAX)
    }

    // A subfolder is a module only if it has a registry of its own. One without
    // is tests-layout's finding, and reporting it here as undeclared would
    // instruct the reader to declare a folder that cannot be declared yet.
    fn submodules_of(&self, directory: &SourcePath) -> Vec<String> {
        self.directories
            .keys()
            .filter(|candidate| candidate.parent().as_ref() == Some(directory))
            .filter(|candidate| !self.registries_in(candidate).is_empty())
            .filter_map(Self::directory_name)
            .collect()
    }
}
