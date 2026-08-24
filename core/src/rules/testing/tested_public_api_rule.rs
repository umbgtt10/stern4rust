// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use crate::finding::model::public_entry_point::PublicEntryPoint;
use crate::finding::parsing::call_site_finder::CallSiteFinder;
use crate::finding::parsing::public_entry_point_finder::PublicEntryPointFinder;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// Every public entry point is called by at least one test.
//
// This is the question `test-naming` gave up on, asked from the other end. That
// rule tried to prove a test tests what its name claims, and every version of it
// -- body, helpers, mirrored source -- eventually accused correct code. Starting
// from the declared entry points instead needs no guess about intent: a
// `pub fn` either appears at a call site under `tests/` or it does not.
//
// It also sidesteps derives entirely, which is what defeated the mirrored-source
// approach. `Default::default` and a `ValueEnum`'s `from_str` are not `pub fn`
// declarations, so they never enter the count and can never be reported.
//
// Matched on **name and arity**. Types and parameter order are not checked and
// cannot be: at a call site `check(3, &paths)` offers two arguments and nothing
// that says whether they fit `usize` and `&[&str]`. That is type inference, and
// this tool reads syntax. Arity is free and separates `new()` from `new(a, b)`,
// which is most of what a bare name confuses.
//
// The consequence to keep in mind is that the rule **under-reports**: two
// entry points sharing a name and an arity are indistinguishable, so a test
// calling one marks both. It errs toward silence rather than toward accusing
// tested code, which is the same direction every other rule here leans.
pub struct TestedPublicApiRule;

impl TestedPublicApiRule {
    pub const SOURCE_ROOT: &'static str = "src/";
    pub const TESTS_ROOT: &'static str = "tests/";

    pub fn new() -> Self {
        Self
    }

    fn declared_in(files: &[SourceFile]) -> Vec<(String, PublicEntryPoint)> {
        files
            .iter()
            .filter(|file| file.relative_path().starts_with(Self::SOURCE_ROOT))
            .flat_map(|file| {
                PublicEntryPointFinder::find(file)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|entry| (file.relative_path().to_string(), entry))
            })
            .collect()
    }

    fn called_in(files: &[SourceFile]) -> BTreeSet<PublicEntryPoint> {
        files
            .iter()
            .filter(|file| file.relative_path().starts_with(Self::TESTS_ROOT))
            .filter_map(CallSiteFinder::find)
            .flatten()
            .collect()
    }

    fn offence(&self, file: &str, entry: &PublicEntryPoint) -> Offence {
        Offence::new(
            file,
            1,
            self.name(),
            format!(
                "`{}` is public but no test calls it with {} argument(s)",
                entry.name, entry.arity
            ),
            format!(
                "call `{}` from a test, or stop exposing it if nothing outside needs it",
                entry.name
            ),
        )
        .with_subject(&entry.signature())
    }
}

impl Default for TestedPublicApiRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for TestedPublicApiRule {
    fn name(&self) -> &'static str {
        "tested-public-api"
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    // A fact about the whole package: the entry point is declared in one file
    // and the call that would exercise it lives in another.
    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        let called = Self::called_in(files);
        Self::declared_in(files)
            .iter()
            .filter(|(_, entry)| !called.contains(entry))
            .map(|(file, entry)| self.offence(file, entry))
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
            "Every public entry point is called by at least one test.",
            "pub fn commit(&self) -> bool   -- no test calls it",
            "#[test]\nfn commit_without_a_quorum_returns_false() {\n    assert!(!store.commit());\n}",
        )
    }
}
