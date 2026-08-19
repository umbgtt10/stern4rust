// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every call a file makes, as a name and a count of arguments.
//
// Macro arguments are scanned as well as parsed expressions, and that is not
// optional: a Rust test puts its assertion in `assert!` or `assert_eq!`, whose
// contents never become syntax. Missing them would report tested code as
// untested, which is the one answer this rule must never give.

use stern4rust::finding::call_site_finder::CallSiteFinder;
use stern4rust::source_file::SourceFile;

fn find(body: &str) -> Vec<String> {
    CallSiteFinder::find(&SourceFile::new("tests/a_tests.rs", body))
        .expect("parses")
        .iter()
        .map(|entry| entry.signature())
        .collect()
}

// The call most tests care about lives inside a macro.
#[test]
fn find_of_a_call_inside_an_assert_records_it() {
    // Arrange & Act
    let found = find("fn t() { assert!(outcome.is_stale()); }\n");

    // Assert
    assert!(found.contains(&"is_stale/0".to_string()), "got {found:?}");
}

#[test]
fn find_of_a_file_that_does_not_parse_returns_nothing() {
    // Arrange & Act
    let found = CallSiteFinder::find(&SourceFile::new("tests/a_tests.rs", "fn broken( {\n"));

    // Assert
    assert!(found.is_none());
}

#[test]
fn find_of_a_free_call_records_its_arity() {
    // Arrange & Act
    let found = find("fn t() { run(1, 2); }\n");

    // Assert
    assert!(found.contains(&"run/2".to_string()), "got {found:?}");
}

#[test]
fn find_of_a_method_call_does_not_count_the_receiver() {
    // Arrange & Act
    let found = find("fn t() { printer.with_fixed(3); }\n");

    // Assert
    assert!(found.contains(&"with_fixed/1".to_string()), "got {found:?}");
}

#[test]
fn find_of_a_nullary_call_records_zero_arity() {
    // Arrange & Act
    let found = find("fn t() { Args::default(); }\n");

    // Assert
    assert!(found.contains(&"default/0".to_string()), "got {found:?}");
}

#[test]
fn find_of_a_path_call_takes_the_last_segment() {
    // Arrange & Act
    let found = find("fn t() { Finder::find(1); }\n");

    // Assert
    assert!(found.contains(&"find/1".to_string()), "got {found:?}");
}
