// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::Path;

use crate::finding::model::package_tree::PackageTree;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A directory holds a number of files a reader can hold in their head.
//
// This is the one rule whose limit is a matter of taste rather than a fact, and
// it is the reason the limit is configuration rather than a constant. Twenty is
// where a directory stops being a list and starts being a wall; somebody else's
// twenty is thirty.
//
// It is also the rule most in tension with the rest of this tool. One struct per
// file, one implemented type per file, and one test file per source file all
// manufacture files by design -- so the limit has to be generous enough that the
// conventions producing the files are not themselves the offence. A limit that
// punished its own standards would be worked around rather than kept.
//
// Registries do not count. A `mod.rs`, `lib.rs` or `all_tests.rs` is an index of
// the directory rather than something in it, and counting the list against the
// length of the list makes no sense. `main.rs` does count: it is an entry point
// holding real code, which is why this list is shorter than the one
// `PackageTree` uses for deciding what may declare a module.
pub struct DirectoryFileCountRule {
    limit: usize,
}

impl DirectoryFileCountRule {
    pub const DEFAULT_LIMIT: usize = 20;
    pub const INDEXES: [&'static str; 3] = ["all_tests.rs", "lib.rs", "mod.rs"];

    pub fn new(limit: usize) -> Self {
        Self { limit }
    }

    fn counted_in(tree: &PackageTree, directory: &Path) -> usize {
        tree.files_in(directory)
            .into_iter()
            .filter(|path| !Self::is_index(path))
            .count()
    }

    fn is_index(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| Self::INDEXES.contains(&name))
    }

    // Reported against the directory's own index where it has one, because that
    // is the file a split has to edit anyway. A directory with no index is named
    // by its path.
    fn offence(&self, tree: &PackageTree, directory: &Path, found: usize) -> Offence {
        let subject = Self::shown(directory);
        let carrier = tree
            .registries_in(directory)
            .first()
            .map(|path| Self::shown(path))
            .unwrap_or_else(|| subject.clone());
        Offence::new(
            &carrier,
            1,
            self.name(),
            format!(
                "{subject} holds {found} files, more than the {} a directory may hold",
                self.limit
            ),
            format!(
                "group the files of {subject} into subfolders of at most {} each, each with its \
                 own mod.rs declared by this index",
                self.limit
            ),
        )
        .with_subject(&subject)
    }

    fn shown(path: &Path) -> String {
        let shown = path.to_string_lossy().replace('\\', "/");
        if shown.is_empty() || shown == "." {
            return "the package root".to_string();
        }
        shown
    }
}

impl Default for DirectoryFileCountRule {
    fn default() -> Self {
        Self::new(Self::DEFAULT_LIMIT)
    }
}

impl Rule for DirectoryFileCountRule {
    fn name(&self) -> &'static str {
        "directory-file-count"
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    // A fact about a directory, so it cannot be answered a file at a time: no
    // single file is the one too many.
    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        let tree = PackageTree::of(files);
        tree.directories()
            .iter()
            .filter_map(|directory| {
                let found = Self::counted_in(&tree, directory);
                (found > self.limit).then(|| self.offence(&tree, directory, found))
            })
            .collect()
    }

    fn requirement(&self) -> Option<&'static str> {
        None
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn explanation(&self) -> RuleExplanation {
        RuleExplanation::new(
            self.name(),
            "A directory holds a number of files a reader can hold in their head.",
            "src/parsing/  -- 24 files",
            "src/parsing/  -- 12 files\nsrc/parsing/naming/  -- 12 files",
        )
    }
}
