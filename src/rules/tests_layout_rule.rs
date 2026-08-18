// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use crate::offence::Offence;
use crate::registry_parser::RegistryParser;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A tests folder is reached through exactly one door -- `tests/all_tests.rs` --
// and a `mod.rs` in every subfolder below it.
//
// Miss one and the files beneath it are not compiled at all. They still exist,
// still look like tests, and are still counted by anyone reading the directory,
// but nothing runs them. That is the failure this rule exists for, and it is
// silent by construction: a test that is never compiled cannot fail.
//
// Both registry files hold nothing but the header and `pub mod` declarations.
// Anything else in them is logic living in the one file a reader scans expecting
// a list.
pub struct TestsLayoutRule;

impl TestsLayoutRule {
    pub const ROOT: &'static str = "tests/";

    pub fn new() -> Self {
        Self
    }

    fn in_tests(files: &[SourceFile]) -> Vec<&SourceFile> {
        files
            .iter()
            .filter(|file| file.relative_path().starts_with(Self::ROOT))
            .collect()
    }

    fn missing_door(&self, present: &[&SourceFile]) -> Vec<Offence> {
        if present
            .iter()
            .any(|file| file.relative_path() == "tests/all_tests.rs")
        {
            return Vec::new();
        }
        vec![Offence::new(
            "tests/all_tests.rs",
            1,
            self.name(),
            "a tests folder is present but has no all_tests.rs, so nothing in it \
             is compiled"
                .to_string(),
        )]
    }

    // A second one below the top is not a door; it is a file with a misleading
    // name that no `pub mod` will ever point at.
    fn stray_doors(&self, present: &[&SourceFile]) -> Vec<Offence> {
        present
            .iter()
            .filter(|file| {
                file.relative_path().ends_with("/all_tests.rs")
                    && file.relative_path() != "tests/all_tests.rs"
            })
            .map(|file| {
                Offence::new(
                    file.relative_path(),
                    1,
                    self.name(),
                    "only tests/all_tests.rs is a registry; this one is never \
                     reached"
                        .to_string(),
                )
            })
            .collect()
    }

    fn missing_mod_files(&self, present: &[&SourceFile]) -> Vec<Offence> {
        let existing: BTreeSet<&str> = present.iter().map(|file| file.relative_path()).collect();
        Self::subfolders(present)
            .into_iter()
            .map(|folder| format!("{folder}/mod.rs"))
            .filter(|expected| !existing.contains(expected.as_str()))
            .map(|expected| {
                Offence::new(
                    &expected,
                    1,
                    self.name(),
                    "a tests subfolder has no mod.rs, so nothing in it is compiled".to_string(),
                )
            })
            .collect()
    }

    // Every folder on the way down, not only the ones holding a file. An
    // intermediate folder is a folder too, and a missing mod.rs there hides
    // everything beneath it just as completely.
    fn subfolders(present: &[&SourceFile]) -> BTreeSet<String> {
        let mut folders = BTreeSet::new();
        for file in present {
            let mut parts: Vec<&str> = file.relative_path().split('/').collect();
            parts.pop();
            for depth in 2..=parts.len() {
                folders.insert(parts[..depth].join("/"));
            }
        }
        folders
    }

    fn registry_contents(&self, present: &[&SourceFile]) -> Vec<Offence> {
        present
            .iter()
            .filter(|file| Self::is_registry(file))
            .flat_map(|file| self.declarations_only(file))
            .collect()
    }

    fn is_registry(file: &SourceFile) -> bool {
        let path = file.relative_path();
        path == "tests/all_tests.rs" || path.ends_with("/mod.rs")
    }

    // Each stray reported at its own line, named. "Something in this file is not
    // a declaration" is true of the whole file and actionable nowhere in it.
    fn declarations_only(&self, file: &SourceFile) -> Vec<Offence> {
        RegistryParser::strays(file)
            .unwrap_or_default()
            .into_iter()
            .map(|stray| {
                Offence::new(
                    file.relative_path(),
                    stray.line,
                    self.name(),
                    format!(
                        "{} does not belong in a registry, which holds the header \
                         and pub mod declarations only",
                        stray.label
                    ),
                )
            })
            .collect()
    }
}

impl Default for TestsLayoutRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for TestsLayoutRule {
    fn name(&self) -> &'static str {
        "tests-layout"
    }

    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        let present = Self::in_tests(files);
        if present.is_empty() {
            return Vec::new();
        }
        let mut offences = self.missing_door(&present);
        offences.extend(self.stray_doors(&present));
        offences.extend(self.missing_mod_files(&present));
        offences.extend(self.registry_contents(&present));
        offences
    }
}
