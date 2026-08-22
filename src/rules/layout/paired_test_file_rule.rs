// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A `<X>_tests.rs` names the source file it exercises, and that file exists.
//
// This is the other half of the mirrored pairing, and the half nothing checked.
// twin4rust starts at a source file and looks for its test, so it reports a
// source file with no tests. Nothing started at a test file and asked whether
// the source it is named after still exists -- and a test file outlives the
// module it was named for silently, because it still compiles, still runs, and
// still passes.
//
// The failure is a reader's, not the compiler's: somebody looking for the tests
// of `retention_window.rs` finds `retention_window_proptest_tests.rs` and never
// learns that `retention_tests.rs` holds seven more.
//
// `_proptest_tests.rs` is exempt. It is a second suite for a module it does not
// name, so its stem resolves to a file nobody ever meant to write. The pairing
// question cannot be asked of it, and asking anyway produced three wrong answers
// out of seven when this was measured.
//
// The rule assumes the package is mirrored. A harness crate -- one whose `src/`
// is apparatus and whose `tests/` are scenarios named after behaviours rather
// than files -- is not, and every one of its test files would be reported.
// `--skip paired-test-file` is the answer there, which is what rule selection is
// for.
pub struct PairedTestFileRule;

impl PairedTestFileRule {
    pub const PROPTEST_POSTFIX: &'static str = "_proptest_tests.rs";
    pub const REGISTRY: &'static str = "all_tests.rs";
    pub const SOURCE_ROOT: &'static str = "src/";
    pub const TESTS_POSTFIX: &'static str = "_tests.rs";
    pub const TESTS_ROOT: &'static str = "tests/";

    pub fn new() -> Self {
        Self
    }

    fn is_test_file(path: &str) -> bool {
        path.starts_with(Self::TESTS_ROOT)
            && path.ends_with(Self::TESTS_POSTFIX)
            && !path.ends_with(Self::PROPTEST_POSTFIX)
            && Self::file_name(path) != Self::REGISTRY
    }

    fn file_name(path: &str) -> &str {
        path.rsplit('/').next().unwrap_or(path)
    }

    // tests/a/b_tests.rs pairs with src/a/b.rs. By path rather than by name, so
    // a test file in the wrong directory is as unpaired as one whose source is
    // gone -- both leave a reader looking in the wrong place.
    fn source_of(path: &str) -> String {
        let without_root = path.strip_prefix(Self::TESTS_ROOT).unwrap_or(path);
        let stem = without_root
            .strip_suffix(Self::TESTS_POSTFIX)
            .unwrap_or(without_root);
        format!("{}{stem}.rs", Self::SOURCE_ROOT)
    }

    fn present(files: &[SourceFile]) -> BTreeSet<&str> {
        files
            .iter()
            .map(SourceFile::relative_path)
            .filter(|path| path.starts_with(Self::SOURCE_ROOT))
            .collect()
    }

    // The correction does not say "create the missing file". Measured across a
    // real tree, every unpaired file tested something real under a name that had
    // drifted, so the file to create is never the answer -- the name is what is
    // wrong, or the tests have outlived their subject.
    fn offence(&self, path: &str, expected: &str) -> Offence {
        Offence::new(
            path,
            1,
            self.name(),
            format!("{path} is named for {expected}, which does not exist"),
            "rename it after the source file it exercises, or delete it if that file is gone"
                .to_string(),
        )
        .with_subject(path)
        .with_expected(expected)
    }
}

impl Default for PairedTestFileRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PairedTestFileRule {
    fn name(&self) -> &'static str {
        "paired-test-file"
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    // A fact about the tree: the file that proves the offence is the source file
    // that is not there, so there is nothing for check() to be handed.
    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        let present = Self::present(files);
        files
            .iter()
            .map(SourceFile::relative_path)
            .filter(|path| Self::is_test_file(path))
            .filter_map(|path| {
                let expected = Self::source_of(path);
                (!present.contains(expected.as_str())).then(|| self.offence(path, &expected))
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
            "A <X>_tests.rs names the source file it exercises, and that file exists.",
            "tests/widget_tests.rs   -- with no src/widget.rs",
            "tests/widget_tests.rs   -- beside src/widget.rs",
        )
    }
}
