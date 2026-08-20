// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::registry_policy::RegistryPolicy;
use crate::finding::parsing::registry_parser::RegistryParser;
use crate::reporting::offence::Offence;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A lib.rs or mod.rs outside tests/ is a list of the modules beneath it and
// nothing else: the header, the crate's inner attributes, `extern crate alloc;`
// and `pub mod` declarations.
//
// The file that names a crate's shape should be readable in one glance. A `use`
// here is a re-export shim wearing a registry's clothes -- the thing this
// repository's own standards forbid outright, caught where it most often
// appears. A `fn` here is code in the one file nobody opens expecting code. An
// inline `mod name { ... }` is a module that no longer has a file to be found
// in, hidden inside the index that was supposed to lead to it.
//
// Inner attributes never reach the item list: syn keeps `#![no_std]` on the
// file rather than among its items, so a no_std crate root passes without this
// rule needing to know which attributes exist.
//
// tests/ is left to tests-layout, which asks a different question of the same
// filenames and gives a different answer about a private `mod`.
pub struct ModuleRegistryRule;

impl ModuleRegistryRule {
    pub const TESTS_ROOT: &'static str = "tests/";

    pub fn new() -> Self {
        Self
    }

    fn applies_to(file: &SourceFile) -> bool {
        !file.relative_path().starts_with(Self::TESTS_ROOT) && Self::is_registry(file)
    }

    fn is_registry(file: &SourceFile) -> bool {
        matches!(
            file.relative_path().rsplit('/').next(),
            Some("lib.rs") | Some("mod.rs")
        )
    }
}

impl Default for ModuleRegistryRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ModuleRegistryRule {
    fn name(&self) -> &'static str {
        "module-registry"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !Self::applies_to(file) {
            return Vec::new();
        }
        RegistryParser::strays(file, RegistryPolicy::source())
            .unwrap_or_default()
            .into_iter()
            .map(|stray| {
                Offence::new(
                    file.relative_path(),
                    stray.line,
                    self.name(),
                    format!(
                        "{} does not belong in a module registry, which holds the \
                         header, inner attributes, `extern crate alloc;` and pub mod \
                         declarations only",
                        stray.label
                    ),
                    format!("move {} into a module of its own", stray.label),
                )
                .with_subject(&stray.label)
            })
            .collect()
    }

    fn check_workspace(&self, _files: &[SourceFile]) -> Vec<Offence> {
        Vec::new()
    }

    fn requirement(&self) -> Option<&'static str> {
        None
    }

    fn is_configured(&self) -> bool {
        true
    }
}
