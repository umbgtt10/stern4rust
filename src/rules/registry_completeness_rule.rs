// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;
use std::path::Path;

use crate::module_declaration_finder::ModuleDeclarationFinder;
use crate::offence::Offence;
use crate::package_tree::PackageTree;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A registry declares every file beside it, so nothing in the tree goes
// uncompiled.
//
// This closes the half of the registry question the other rules leave open.
// `tests-layout` and `module-registry` both check that a registry exists and
// holds only declarations; neither checks that the declarations are complete. A
// `mod.rs` that is present, valid, and simply fails to mention `alpha_tests`
// leaves `alpha_tests.rs` uncompiled -- and a test that is never compiled cannot
// fail.
//
// Only one direction needs a rule, which is worth stating because it is not
// obvious. `pub mod missing;` with no `missing.rs` is a **compile error**, so
// rustc already reports it, loudly and better than this could. An orphan
// `.rs` file that no registry declares produces no error and no warning at all.
// Silence is the whole failure, so silence is all this rule looks for.
pub struct RegistryCompletenessRule;

impl RegistryCompletenessRule {
    pub fn new() -> Self {
        Self
    }

    fn undeclared_in(
        &self,
        tree: &PackageTree,
        files: &[SourceFile],
        directory: &Path,
    ) -> Vec<Offence> {
        let registries = tree.registries_in(directory);
        let Some(primary) = registries.first() else {
            return Vec::new();
        };
        let Some(declared) = Self::declared_by(files, &registries) else {
            return Vec::new();
        };
        tree.expected_modules_in(directory)
            .into_iter()
            .filter(|name| !declared.contains(name))
            .map(|name| self.offence(primary, &name))
            .collect()
    }

    // None when any registry of the directory cannot be read or parsed. Treating
    // an unreadable registry as declaring nothing would report every file beside
    // it as an orphan -- a page of wrong answers caused by one real one, which
    // readable-source already reports.
    fn declared_by(files: &[SourceFile], registries: &[&Path]) -> Option<BTreeSet<String>> {
        let mut declared = BTreeSet::new();
        for path in registries {
            let file = Self::file_at(files, path)?;
            declared.extend(ModuleDeclarationFinder::find(file)?);
        }
        Some(declared)
    }

    fn file_at<'a>(files: &'a [SourceFile], path: &Path) -> Option<&'a SourceFile> {
        let wanted = path.to_string_lossy().replace('\\', "/");
        files
            .iter()
            .find(|file| file.relative_path().replace('\\', "/") == wanted)
    }

    // Reported against the registry rather than the orphan. The orphan is not
    // wrong -- it is a perfectly good file -- and the edit that fixes it is one
    // line in the registry.
    fn offence(&self, registry: &Path, name: &str) -> Offence {
        let registry = registry.to_string_lossy().replace('\\', "/");
        Offence::new(
            &registry,
            1,
            self.name(),
            format!("`{name}` is not declared here, so its file is never compiled"),
            format!("add `pub mod {name};` to {registry}"),
        )
        .with_subject(name)
    }
}

impl Default for RegistryCompletenessRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for RegistryCompletenessRule {
    fn name(&self) -> &'static str {
        "registry-completeness"
    }

    // A fact about a directory rather than about a file: the file that proves
    // the offence is the one that is missing a line, and the file that suffers
    // is a different one entirely.
    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        let tree = PackageTree::of(files);
        tree.directories()
            .iter()
            .flat_map(|directory| self.undeclared_in(&tree, files, directory))
            .collect()
    }
}
