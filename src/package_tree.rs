// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use crate::source_file::SourceFile;

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
    directories: BTreeMap<PathBuf, Vec<PathBuf>>,
}

impl PackageTree {
    pub const REGISTRY_NAMES: [&'static str; 4] = ["all_tests.rs", "lib.rs", "main.rs", "mod.rs"];

    pub fn of(files: &[SourceFile]) -> Self {
        let mut directories: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
        for file in files {
            let path = PathBuf::from(file.relative_path().replace('\\', "/"));
            let parent = path.parent().unwrap_or(Path::new("")).to_path_buf();
            directories.entry(parent).or_default().push(path);
        }
        Self { directories }
    }

    pub fn directories(&self) -> Vec<&Path> {
        self.directories.keys().map(PathBuf::as_path).collect()
    }

    // The registries of one directory, in REGISTRY_NAMES order, so the offence
    // lands on `lib.rs` rather than `main.rs` when a package has both.
    pub fn registries_in(&self, directory: &Path) -> Vec<&Path> {
        let mut found: Vec<&Path> = self
            .files_in(directory)
            .into_iter()
            .filter(|path| Self::is_registry(path))
            .collect();
        found.sort_by_key(|path| Self::registry_rank(path));
        found
    }

    // What the registries of this directory have to declare for everything
    // beside them to be compiled: each sibling module file, and each subfolder
    // that is itself a module.
    pub fn expected_modules_in(&self, directory: &Path) -> Vec<String> {
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

    fn files_in(&self, directory: &Path) -> Vec<&Path> {
        self.directories
            .get(directory)
            .map(|paths| paths.iter().map(PathBuf::as_path).collect())
            .unwrap_or_default()
    }

    // A subfolder is a module only if it has a registry of its own. One without
    // is tests-layout's finding, and reporting it here as undeclared would
    // instruct the reader to declare a folder that cannot be declared yet.
    fn submodules_of(&self, directory: &Path) -> Vec<String> {
        self.directories
            .keys()
            .filter(|candidate| candidate.parent() == Some(directory))
            .filter(|candidate| !self.registries_in(candidate).is_empty())
            .filter_map(|candidate| Self::directory_name(candidate))
            .collect()
    }

    fn directory_name(path: &Path) -> Option<String> {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
    }

    fn is_registry(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| Self::REGISTRY_NAMES.contains(&name))
    }

    fn module_name(path: &Path) -> Option<String> {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(str::to_string)
    }

    fn registry_rank(path: &Path) -> usize {
        path.file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| Self::REGISTRY_NAMES.iter().position(|known| *known == name))
            .unwrap_or(usize::MAX)
    }
}
