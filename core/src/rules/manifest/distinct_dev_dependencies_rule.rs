// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::manifest_dependency::ManifestDependency;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A dev-dependency adds to the dependencies.
//
// The tests of a package already have every crate its `[dependencies]` names. A
// `[dev-dependencies]` entry naming one of them again, written the same way and
// asking for nothing more, is a line that does nothing -- and nothing else says
// so: cargo accepts it, and cargo-shear looks for unused dependencies, not
// repeated ones. Found in three of the family's cores, this crate's among them.
//
// A repeat that adds something is the only way cargo has of giving the tests
// it, so it is left alone: a feature production must not ship, the default
// features a dependency turns off, a crate optional there. So is a pair written
// unlike -- one taken from the workspace, one spelled out, two version
// requirements -- whose difference this manifest alone cannot weigh. What
// `ManifestDependency::repeats` decides.
//
// Read from the TOML, in every package, workspace or not: the question is about
// one manifest, and unlike `workspace-dependencies` it has an answer outside a
// workspace.
pub struct DistinctDevDependenciesRule {
    declared: Vec<ManifestDependency>,
}

impl DistinctDevDependenciesRule {
    pub fn new(declared: Vec<ManifestDependency>) -> Self {
        Self { declared }
    }

    // Reported against the manifest at line 1, as `workspace-dependencies` is:
    // the fix is one line deleted there.
    fn offence(&self, dev: &ManifestDependency, normal: &ManifestDependency) -> Offence {
        let name = dev.name();
        Offence::new(
            dev.manifest(),
            1,
            self.name(),
            format!(
                "{} declares `{name}` in [{}] as well as in [{}], and adds nothing to it",
                dev.manifest(),
                dev.table(),
                normal.table()
            ),
            format!(
                "remove `{name}` from [{}]: the tests already have it from [{}]",
                dev.table(),
                normal.table()
            ),
        )
        .with_subject(name)
    }

    // The dependency an entry repeats, if it is a dev-dependency adding nothing
    // to one.
    fn repeated(&self, dev: &ManifestDependency) -> Option<&ManifestDependency> {
        self.declared.iter().find(|normal| dev.repeats(normal))
    }
}

impl Rule for DistinctDevDependenciesRule {
    fn name(&self) -> &'static str {
        "distinct-dev-dependencies"
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    // A fact about the manifest, a file the walker never reads: its entries are
    // gathered by `ManifestResolver` and handed to each package for its own.
    fn check_workspace(&self, _files: &[SourceFile]) -> Vec<Offence> {
        self.declared
            .iter()
            .filter_map(|dev| self.repeated(dev).map(|normal| self.offence(dev, normal)))
            .collect()
    }

    fn requirement(&self) -> Option<&'static str> {
        None
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn explanation(&self) -> RuleExplanation {
        RuleExplanation {
            name: self.name(),
            summary: "A [dev-dependencies] entry gives the tests something [dependencies] does not: a crate, a feature, or a crate that is optional there.",
            breaks: "[dependencies]\nclap = { workspace = true }\n\n[dev-dependencies]\nclap = { workspace = true }",
            instead: "[dependencies]\nclap = { workspace = true }",
        }
    }
}
