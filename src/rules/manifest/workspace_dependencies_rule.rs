// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::manifest_dependency::ManifestDependency;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A workspace declares its dependencies once, in the root, and every member
// takes them from there.
//
// Three requirements were asked for: the root holds every reference, each member
// uses `.workspace`, and no member declares one of its own. **They are the same
// requirement**, and only the middle one needs code.
//
// A member that writes `foo = { workspace = true }` for a `foo` the root does
// not declare does not compile -- `cargo` rejects it outright. So requiring the
// root to hold every reference costs nothing here, and "no new dependencies in a
// member" is what a member using nothing but `.workspace` already means. One
// syntactic check delivers all three, which is the split
// [R009], [R014] and [R016] each found before it.
//
// The declaration is read from the TOML rather than from `cargo metadata`,
// because the question is *how it was written*. Resolution erases exactly that
// distinction: a dependency taken from the workspace and one spelled out in the
// member are identical once resolved.
//
// A package that is not a workspace has no root to centralise into, so the rule
// says nothing rather than reporting something wrong -- the same silence
// `tests-layout` keeps about a package with no tests tree.
pub struct WorkspaceDependenciesRule {
    declared: Option<Vec<ManifestDependency>>,
}

impl WorkspaceDependenciesRule {
    pub fn new(declared: Option<Vec<ManifestDependency>>) -> Self {
        Self { declared }
    }

    // Reported against the member manifest at line 1: the edit is one line in
    // that file, plus one in the root.
    fn offence(&self, dependency: &ManifestDependency) -> Offence {
        let name = &dependency.name;
        Offence::new(
            &dependency.manifest,
            1,
            self.name(),
            format!(
                "{} declares `{name}` in [{}] rather than taking it from the workspace",
                dependency.manifest, dependency.section
            ),
            format!(
                "add `{name}` to [workspace.dependencies] in the root manifest, and write \
                 `{name} = {{ workspace = true }}` here"
            ),
        )
        .with_subject(name)
        .with_expected(&format!("{name} = {{ workspace = true }}"))
    }
}

impl Rule for WorkspaceDependenciesRule {
    fn name(&self) -> &'static str {
        "workspace-dependencies"
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    // A fact about the workspace, and about files the walker never reads: the
    // manifests are gathered once by `ManifestResolver` rather than found here.
    fn check_workspace(&self, _files: &[SourceFile]) -> Vec<Offence> {
        self.declared
            .iter()
            .flatten()
            .filter(|dependency| !dependency.takes_from_workspace)
            .map(|dependency| self.offence(dependency))
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
            "A workspace declares its dependencies once, in the root, and every member takes them from there.",
            "serde = { version = \"1\" }   -- in a member manifest",
            "serde.workspace = true",
        )
    }
}
