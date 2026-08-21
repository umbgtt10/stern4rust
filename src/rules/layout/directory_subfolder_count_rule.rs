// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::Path;

use crate::finding::model::package_tree::PackageTree;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A directory holds a number of subfolders a reader can hold in their head.
//
// The counterweight to `directory-file-count`. That rule creates folders; this
// one stops the creating from being the answer to everything, because a
// directory with twenty subfolders is exactly as unreadable as one with a
// hundred files and looks tidier while being worse.
//
// It is checked at every level rather than only at the root, so pushing the
// sprawl one directory down does not escape it.
//
// Measured across eight repositories at the time it was written it finds
// **nothing**: the deepest tree is two levels and no directory has more than one
// subfolder. That is stated plainly because a rule that has never fired has not
// yet earned the reader's trust, and this one is a guard against a shape the
// family has not reached rather than a description of a problem it has.
pub struct DirectorySubfolderCountRule {
    limit: usize,
}

impl DirectorySubfolderCountRule {
    pub const DEFAULT_LIMIT: usize = 5;

    pub fn new(limit: usize) -> Self {
        Self { limit }
    }

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
                "{subject} holds {found} subfolders, more than the {} a directory may hold",
                self.limit
            ),
            format!(
                "group the subfolders of {subject} so that no directory holds more than {}",
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

impl Default for DirectorySubfolderCountRule {
    fn default() -> Self {
        Self::new(Self::DEFAULT_LIMIT)
    }
}

impl Rule for DirectorySubfolderCountRule {
    fn name(&self) -> &'static str {
        "directory-subfolder-count"
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        let tree = PackageTree::of(files);
        tree.directories()
            .iter()
            .filter_map(|directory| {
                let found = tree.subdirectories_of(directory).len();
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
            "A directory holds a number of subfolders a reader can hold in their head.",
            "src/  -- 9 subfolders",
            "src/  -- 4 subfolders, each grouping the rest",
        )
    }
}
