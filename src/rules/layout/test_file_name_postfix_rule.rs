// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Attribute;
use syn::Item;
use syn::ItemMod;
use syn::parse_file;

use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A file that holds tests is named for it: `<X>_tests.rs`.
//
// The name is what pairs a test file with the source file it exercises. That
// pairing is the whole basis of the mirrored layout -- `src/foo.rs` answering to
// `tests/foo_tests.rs` -- and a file holding tests under any other name is
// outside it: nothing points at it from the source side, and no tool checking
// the pair can see that it is the test file somebody wrote.
//
// One direction only. Holding a test obliges the name; a `_tests.rs` file
// holding none is a different question, and a separate rule if it ever earns
// one.
//
// The two exemptions do real work rather than softening the rule.
//
// `src/` is exempt because a `#[test]` there is already `test-free-source`'s
// offence, and the correction here would be **wrong**: renaming `src/foo.rs` to
// `src/foo_tests.rs` leaves the test exactly where it does not belong. That file
// has to move, not be renamed.
//
// Registries are exempt for the same reason from the other side. A `#[test]` in
// an `all_tests.rs` or a `mod.rs` is already `tests-layout`'s offence, and
// `mod.rs` cannot be renamed at all -- the correction would be impossible to
// follow.
pub struct TestFileNamePostfixRule;

impl TestFileNamePostfixRule {
    pub const POSTFIX: &'static str = "_tests.rs";
    pub const REGISTRIES: [&'static str; 2] = ["all_tests.rs", "mod.rs"];
    pub const TESTS_ROOT: &'static str = "tests/";

    pub fn new() -> Self {
        Self
    }

    fn applies_to(file: &SourceFile) -> bool {
        let path = file.relative_path();
        path.starts_with(Self::TESTS_ROOT)
            && !path
                .rsplit('/')
                .next()
                .is_some_and(|name| Self::REGISTRIES.contains(&name))
    }

    // Descends into inline modules: a test does not stop being a test for
    // sitting one level down, and the file still holds it.
    fn tests_in(items: &[Item]) -> usize {
        items
            .iter()
            .map(|item| match item {
                Item::Fn(function) if Self::is_test(&function.attrs) => 1,
                Item::Mod(module) => Self::inside(module).map(Self::tests_in).unwrap_or_default(),
                _ => 0,
            })
            .sum()
    }

    fn inside(module: &ItemMod) -> Option<&[Item]> {
        module.content.as_ref().map(|(_, items)| items.as_slice())
    }

    // The last segment is what says so, which is what lets `#[tokio::test]`
    // count without naming any runtime here.
    fn is_test(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "test")
        })
    }

    fn suggested_name(relative_path: &str) -> String {
        let stem = relative_path
            .strip_suffix(".rs")
            .unwrap_or(relative_path)
            .to_string();
        format!("{stem}{}", Self::POSTFIX)
    }

    // Reported at line 1: the offence is the file's name, not any one test in
    // it. The count is named as the evidence for calling it a test file.
    fn offence(&self, file: &SourceFile, found: usize) -> Offence {
        let path = file.relative_path();
        let suggested = Self::suggested_name(path);
        Offence::new(
            path,
            1,
            self.name(),
            format!(
                "{path} holds {found} test(s) but its name does not end in `{}`, so nothing \
                 pairs it with the source file it exercises",
                Self::POSTFIX
            ),
            format!("rename it `{suggested}`"),
        )
        .with_subject(path)
        .with_expected(&suggested)
    }
}

impl Default for TestFileNamePostfixRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for TestFileNamePostfixRule {
    fn name(&self) -> &'static str {
        "test-file-name-postfix"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !Self::applies_to(file) || file.relative_path().ends_with(Self::POSTFIX) {
            return Vec::new();
        }
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        match Self::tests_in(&syntax.items) {
            0 => Vec::new(),
            found => vec![self.offence(file, found)],
        }
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

    fn explanation(&self) -> RuleExplanation {
        RuleExplanation::new(
            self.name(),
            "A file that holds tests is named for it: <X>_tests.rs.",
            "tests/widget_spec.rs",
            "tests/widget_tests.rs",
        )
    }
}
