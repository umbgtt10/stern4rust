// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::vec::Vec;

use syn::ItemFn;
use syn::parse_file;

use crate::finding::parsing::unit_test_finder::UnitTestFinder;
use crate::source_file::SourceFile;

// The tests a rule over test files judges: the top-level functions carrying a
// test attribute, in a file under tests/ that is no registry -- `all_tests.rs`
// and a `mod.rs` list test files rather than hold tests.
pub struct TestFunctions;

impl TestFunctions {
    pub const REGISTRIES: [&'static str; 2] = ["all_tests.rs", "mod.rs"];
    pub const TESTS_ROOT: &'static str = "tests/";

    // What each test of the file was found to break, in the order the tests
    // stand.
    pub fn judged<T, I>(file: &SourceFile, judge: impl Fn(&ItemFn) -> I) -> Vec<T>
    where
        I: IntoIterator<Item = T>,
    {
        Self::of(file).iter().flat_map(judge).collect()
    }

    pub fn of(file: &SourceFile) -> Vec<ItemFn> {
        if !file.is_under(Self::TESTS_ROOT, &Self::REGISTRIES) {
            return Vec::new();
        }
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        UnitTestFinder::test_functions(&syntax.items)
            .into_iter()
            .cloned()
            .collect()
    }
}
