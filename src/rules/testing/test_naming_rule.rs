// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Attribute;
use syn::Item;
use syn::ItemFn;
use syn::parse_file;

use crate::reporting::offence::Offence;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A test is named `<method>_<conditions>_<result>`.
//
// This rule reads the name and nothing else, which is a deliberate retreat from
// where it started. Earlier versions tried to verify that the name's leading
// part was the method actually under test -- by looking for it in the body, then
// through the test file's helpers transitively, then against the mirrored source
// file. All three were measured across the workspace and all three produced
// confident wrong answers on correct code: tests of derived operators (`a < b`
// calls no named function), tests of derived methods (`from_str` on a
// `#[derive(ValueEnum)]` enum is not a `fn` anywhere), and names reachable only
// through whatever a wide setup helper happened to touch.
//
// The question those versions were reaching for -- is this thing actually
// tested -- is answered from the other end by `tested-public-api`, which starts
// from the declared entry points instead of guessing at intent. This rule keeps
// the part that can be checked without ever being wrong: a name with fewer than
// three parts cannot carry a method, a condition and a result.
pub struct TestNamingRule;

impl TestNamingRule {
    pub const MINIMUM_PARTS: usize = 3;
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

    fn is_test(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "test")
        })
    }

    fn offence(&self, file: &SourceFile, function: &ItemFn) -> Option<Offence> {
        let name = function.sig.ident.to_string();
        if name.split('_').count() >= Self::MINIMUM_PARTS {
            return None;
        }
        Some(self.shape_offence(file, &name, function.sig.ident.span().start().line))
    }

    fn shape_offence(&self, file: &SourceFile, name: &str, line: usize) -> Offence {
        Offence::new(
            file.relative_path(),
            line,
            self.name(),
            format!(
                "`{name}` has fewer than {} parts, so it cannot say what it calls, under what \
                 conditions, and with what result",
                Self::MINIMUM_PARTS
            ),
            format!(
                "rename it `<method>_<conditions>_<result>`, starting with the method {name} calls"
            ),
        )
        .with_subject(name)
    }
}

impl Default for TestNamingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for TestNamingRule {
    fn name(&self) -> &'static str {
        "test-naming"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !Self::applies_to(file) {
            return Vec::new();
        }
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        syntax
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Fn(function) if Self::is_test(&function.attrs) => Some(function),
                _ => None,
            })
            .filter_map(|function| self.offence(file, function))
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
