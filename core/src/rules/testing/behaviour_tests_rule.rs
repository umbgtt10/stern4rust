// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::finding::parsing::shapes::package_surface::PackageSurface;
use crate::finding::parsing::shapes::test_subject::TestSubject;
use crate::finding::parsing::shapes::test_subject_finder::TestSubjectFinder;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A test that builds a record and calls nothing of its package but the record's
// accessors and derived traits tests the compiler: that a derived `Default`
// gives zeros, that a clone equals its original, that a constructor keeps its
// arguments. Its check belongs in the test of the code that uses the record.
pub struct BehaviourTestsRule;

impl BehaviourTestsRule {
    pub const NAME: &'static str = "behaviour-tests";
    pub const SOURCE_ROOT: &'static str = "src/";
    pub const TESTS_ROOT: &'static str = "tests/";

    pub fn new() -> Self {
        Self
    }

    fn offence(file: &str, subject: &TestSubject) -> Offence {
        let records: Vec<String> = subject
            .records
            .iter()
            .map(|record| format!("`{record}`"))
            .collect();
        let named = records.join(", ");
        Offence::new(
            file,
            subject.line,
            Self::NAME,
            format!(
                "`{}` tests {named} alone: it calls nothing of its package but a record's accessors and derived traits",
                subject.name
            ),
            format!("test the behaviour that uses {named}, and check there what this test checks"),
        )
        .with_subject(&subject.name)
    }
}

impl Default for BehaviourTestsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for BehaviourTestsRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    // A fact about the package: the record is declared in one file and the test
    // that calls nothing else lives in another.
    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        let sources: Vec<SourceFile> = files
            .iter()
            .filter(|file| file.relative_path().starts_with(Self::SOURCE_ROOT))
            .cloned()
            .collect();
        let surface = PackageSurface::of(&sources);
        if !surface.has_records() {
            return Vec::new();
        }
        let finder = TestSubjectFinder::new(&surface);
        files
            .iter()
            .filter(|file| file.relative_path().starts_with(Self::TESTS_ROOT))
            .flat_map(|file| {
                finder
                    .find(file)
                    .into_iter()
                    .filter(|subject| !subject.records.is_empty() && !subject.exercises_behaviour)
                    .map(|subject| Self::offence(file.relative_path(), &subject))
                    .collect::<Vec<Offence>>()
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
        RuleExplanation {
            name: self.name(),
            summary: "A test exercises behaviour: one that calls nothing but a record's accessors and derived traits tests the compiler.",
            breaks: "#[test]\nfn default_counts_nothing() {\n    assert_eq!(Summary::default(), Summary { count: 0 });\n}",
            instead: "#[test]\nfn summarise_of_no_receipts_counts_nothing() {\n    assert_eq!(summarise(&[]), Summary { count: 0 });\n}",
        }
    }
}
