// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::unit_test_finder::UnitTestFinder;
use crate::reporting::offence::Offence;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// Tests live in tests/, and the production source tree carries none of them.
//
// A unit test inside src/ is a test nobody can find from the outside. It does
// not appear in the mirrored test file twin4rust checks for, it is not declared
// from all_tests.rs, and it is compiled under a configuration the shipped build
// never uses -- so the file reads as covered while the coverage lives somewhere
// nothing else in the toolchain looks.
//
// `#[cfg_attr(test, ...)]` is the same door under a different name: a type
// carrying a derive only under test is a type that means one thing to the tests
// and another to the shipped build. Only the test-gated spelling is forbidden --
// `#[cfg_attr(feature = "serde", ...)]` is ordinary library work, and so is
// `#[cfg(feature = "...")]`, because both gate on something the shipped build
// can also select.
pub struct TestFreeSourceRule;

impl TestFreeSourceRule {
    pub const ROOT: &'static str = "tests/";

    pub fn new() -> Self {
        Self
    }

    // tests/ is exempt, and not as a concession. A #[test] under tests/ is the
    // entire point of tests/, and a rule that reported it would report every
    // test in the workspace.
    fn applies_to(file: &SourceFile) -> bool {
        !file.relative_path().starts_with(Self::ROOT)
    }
}

impl Default for TestFreeSourceRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for TestFreeSourceRule {
    fn name(&self) -> &'static str {
        "test-free-source"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !Self::applies_to(file) {
            return Vec::new();
        }
        UnitTestFinder::sites(file)
            .unwrap_or_default()
            .into_iter()
            .map(|site| {
                Offence::new(
                    file.relative_path(),
                    site.line,
                    self.name(),
                    format!("{} does not belong in the source tree", site.label),
                    site.correction.clone(),
                )
                .with_subject(&site.label)
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
